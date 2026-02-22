use crate::consts::{CHO_TO_JONG_MAP, KERN_JAMO_MAP};
use crate::error::{ConfigError, Error, FontError};
use crate::file::{delete_font_dir, get_evolution_dir, get_evolution_str, get_font_dir};
use crate::structs::{
    Args, Config, ConfigMutationRule, EvolutionConfigFile, KerningMutationAxes,
    KerningMutationRule, MutationNumber,
};
use crate::{compile, compile_selected_hangul_composites, make_woff2, KerningMap};
use ahash::HashSet;
use chrono::Utc;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JamoKind {
    Cho,
    Jung,
    Jong,
}

#[derive(Debug, Clone)]
pub enum EvolutionCompileMode {
    Full,
    Fast { content: String },
}

#[derive(Debug, Clone)]
pub struct EvolutionCandidate {
    pub generation: u64,
    pub config: Config,
    pub kerning_data: KerningMap,
}

#[derive(Debug, Clone)]
pub struct EvolutionEngine {
    evolution_name: String,
    evolution_config: EvolutionConfigFile,
    generation: u64,
    seed: u64,
    rng: XorShift64,
}

#[derive(Debug, Clone, Serialize)]
struct EvolutionCheckpointMeta {
    version: u32,
    evolution_name: String,
    generation: u64,
    seed: u64,
    created_at_utc: String,
}

#[derive(Debug, Clone)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_usize(&mut self, upper_bound: usize) -> usize {
        if upper_bound <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper_bound
    }

    fn next_unit_f64(&mut self) -> f64 {
        // Keep values strictly inside (0, 1) so Box-Muller stays stable.
        ((self.next_u64() as f64) + 1.0) / ((u64::MAX as f64) + 2.0)
    }

    fn next_standard_normal(&mut self) -> f64 {
        let u1 = self.next_unit_f64();
        let u2 = self.next_unit_f64();
        let r = (-2.0_f64 * u1.ln()).sqrt();
        let theta = std::f64::consts::TAU * u2;
        r * theta.cos()
    }
}

impl EvolutionEngine {
    pub fn load_from_file(evolution_name: &str, seed: Option<u64>) -> Result<Self, Error> {
        let evolution_data = get_evolution_str(evolution_name);
        if evolution_data.is_empty() {
            return Err(Error::Config(ConfigError {
                msg: format!(
                    "Evolution config '{}' does not exist or is empty",
                    evolution_name
                ),
            }));
        }
        let evolution_config: EvolutionConfigFile = match json5::from_str(&evolution_data) {
            Ok(v) => v,
            Err(e) => {
                return Err(Error::Config(ConfigError {
                    msg: format!(
                        "Error parsing evolution config '{}': {:?}",
                        evolution_name, e
                    ),
                }));
            }
        };
        if let Err(msg) = evolution_config.validate() {
            return Err(Error::Config(ConfigError {
                msg: format!("Invalid evolution config '{}': {}", evolution_name, msg),
            }));
        }
        let seed = seed.unwrap_or_else(default_seed);
        Ok(Self {
            evolution_name: evolution_name.to_string(),
            evolution_config,
            generation: 0,
            seed,
            rng: XorShift64::new(seed),
        })
    }

    pub fn evolution_name(&self) -> &str {
        &self.evolution_name
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn mutate_once(
        &mut self,
        base_config: &Config,
        base_kerning_data: &KerningMap,
    ) -> Result<EvolutionCandidate, Error> {
        let mut config = base_config.clone();
        let mut kerning_data = base_kerning_data.clone();
        self.apply_config_mutation(&mut config)?;
        self.apply_kerning_mutation(&mut kerning_data)?;
        self.generation += 1;
        Ok(EvolutionCandidate {
            generation: self.generation,
            config,
            kerning_data,
        })
    }

    pub fn generate_font_for_mutation(
        &self,
        candidate: &EvolutionCandidate,
        glyph_set: &str,
        compile_mode: EvolutionCompileMode,
        check_collision: bool,
        target_fontname: &str,
    ) -> Result<(), Error> {
        let args = build_args_from_config(
            candidate.config.clone(),
            candidate.kerning_data.clone(),
            target_fontname.to_string(),
        )?;
        let font_dir = get_font_dir(&args.target_fontname);
        if font_dir.exists() {
            delete_font_dir(&args.target_fontname)?;
        }
        match compile_mode {
            EvolutionCompileMode::Full => {
                compile(&args, glyph_set, check_collision)?;
            }
            EvolutionCompileMode::Fast { content } => {
                let codepoints = collect_hangul_syllable_codepoints(&content);
                compile_selected_hangul_composites(&args, glyph_set, &codepoints, check_collision)?;
            }
        }
        make_woff2(&args)?;
        Ok(())
    }

    pub fn save_checkpoint_last(&self, candidate: &EvolutionCandidate) -> Result<(), Error> {
        let checkpoint_dir = get_checkpoint_dir(&self.evolution_name);
        if let Err(e) = std::fs::create_dir_all(&checkpoint_dir) {
            return Err(Error::Font(FontError {
                msg: format!(
                    "Cannot create evolution checkpoint directory {:?}: {:?}",
                    checkpoint_dir, e
                ),
            }));
        }
        let config_path = checkpoint_dir.join("last.config.json5");
        let kerning_path = checkpoint_dir.join("last.kerning.txt");
        let meta_path = checkpoint_dir.join("last.meta.json5");

        let config_data = match serde_json::to_string_pretty(&candidate.config) {
            Ok(v) => v,
            Err(e) => {
                return Err(Error::Config(ConfigError {
                    msg: format!("Failed to serialize checkpoint config: {:?}", e),
                }));
            }
        };
        write_atomic_string(&config_path, &config_data)?;

        let kerning_data = serialize_kerning_checkpoint(&candidate.kerning_data)?;
        write_atomic_string(&kerning_path, &kerning_data)?;

        let meta = EvolutionCheckpointMeta {
            version: 1,
            evolution_name: self.evolution_name.clone(),
            generation: candidate.generation,
            seed: self.seed,
            created_at_utc: Utc::now().to_rfc3339(),
        };
        let meta_data = match serde_json::to_string_pretty(&meta) {
            Ok(v) => v,
            Err(e) => {
                return Err(Error::Config(ConfigError {
                    msg: format!("Failed to serialize checkpoint metadata: {:?}", e),
                }));
            }
        };
        write_atomic_string(&meta_path, &meta_data)?;
        Ok(())
    }

    fn apply_config_mutation(&mut self, config: &mut Config) -> Result<(), Error> {
        let rules = match &self.evolution_config.config {
            Some(v) => v.clone(),
            None => return Ok(()),
        };
        let mut keys: Vec<String> = rules.keys().cloned().collect();
        keys.sort();
        for key in keys {
            let rule = match rules.get(&key) {
                Some(v) => v,
                None => continue,
            };
            match rule {
                ConfigMutationRule::String { mutation_string } => {
                    if mutation_string.options.is_empty() {
                        return Err(Error::Config(ConfigError {
                            msg: format!("config.{}.mutation_string.options cannot be empty", key),
                        }));
                    }
                    let center_idx = self.get_config_string_center_index(
                        config,
                        &key,
                        &mutation_string.options,
                    )?;
                    let idx =
                        self.sample_centered_index(center_idx, mutation_string.options.len() - 1);
                    self.set_config_string(config, &key, mutation_string.options[idx].clone())?;
                }
                ConfigMutationRule::Float { mutation_number } => {
                    let center = self.get_config_float_center(config, &key, mutation_number)?;
                    let mut value = self.sample_number(mutation_number, center)?;
                    if value < mutation_number.min {
                        value = mutation_number.min;
                    } else if value > mutation_number.max {
                        value = mutation_number.max;
                    }
                    self.set_config_float(config, &key, value)?;
                }
            }
        }
        Ok(())
    }

    fn set_config_string(
        &self,
        config: &mut Config,
        key: &str,
        value: String,
    ) -> Result<(), Error> {
        match key {
            "source" => {
                let next = value.trim().to_string();
                config.source = if next.is_empty() { None } else { Some(next) };
            }
            "cho_type" => config.cho_type = value,
            "jung_type" => config.jung_type = value,
            "jong_type" => config.jong_type = value,
            _ => {
                return Err(Error::Config(ConfigError {
                    msg: format!(
                        "Unsupported string config key '{}' in evolution config",
                        key
                    ),
                }));
            }
        }
        Ok(())
    }

    fn get_config_string_center_index(
        &self,
        config: &Config,
        key: &str,
        options: &[String],
    ) -> Result<usize, Error> {
        let current = match key {
            "source" => config.source.clone().unwrap_or_default(),
            "cho_type" => config.cho_type.clone(),
            "jung_type" => config.jung_type.clone(),
            "jong_type" => config.jong_type.clone(),
            _ => {
                return Err(Error::Config(ConfigError {
                    msg: format!(
                        "Unsupported string config key '{}' in evolution config",
                        key
                    ),
                }));
            }
        };
        Ok(options.iter().position(|opt| opt == &current).unwrap_or(0))
    }

    fn get_config_float_center(
        &self,
        config: &Config,
        key: &str,
        mutation_number: &MutationNumber,
    ) -> Result<f32, Error> {
        let value = match key {
            "cho_h_ratio" => config.cho_h_ratio.unwrap_or(0.0),
            "jung_w_ratio" => config.jung_w_ratio.unwrap_or(1.0),
            "jong_w_ratio" => config.jong_w_ratio.unwrap_or(1.0),
            "jung_h_ratio" => config.jung_h_ratio.unwrap_or(1.0),
            "jong_h_ratio" => config.jong_h_ratio.unwrap_or(1.0),
            "char_gap" => config.char_gap.unwrap_or(0) as f32,
            "cho_cho_gap" => config.cho_cho_gap.unwrap_or(0) as f32,
            "jung_jung_gap" => config.jung_jung_gap.unwrap_or(0) as f32,
            "jong_jong_gap" => config.jong_jong_gap.unwrap_or(0) as f32,
            "cho_jung_gap" => config.cho_jung_gap.unwrap_or(0) as f32,
            "jung_jong_gap" => config.jung_jong_gap.unwrap_or(0) as f32,
            "x_sw" => config.x_sw.unwrap_or(0.2),
            "y_sw" => config.y_sw.unwrap_or(0.2),
            "text_size" => config.text_size.unwrap_or(16) as f32,
            "underdot_y" => config.underdot_y.unwrap_or(-300) as f32,
            "underdot_r_ratio" => config.underdot_r_ratio.unwrap_or(0.5),
            "upperdot_y" => config.upperdot_y.unwrap_or(1800) as f32,
            "upperdot_r_ratio" => config.upperdot_r_ratio.unwrap_or(0.5),
            "glyph_width" => config.glyph_width.unwrap_or(800) as f32,
            "cap_height" => config.cap_height.unwrap_or(1800) as f32,
            "x_height" => config.x_height.unwrap_or(1500) as f32,
            "baseline" => config.baseline.unwrap_or(0) as f32,
            "min_gap" => config.min_gap.unwrap_or(200) as f32,
            "space_width" => config
                .space_width
                .map(|v| v as f32)
                .unwrap_or(mutation_number.min),
            "space_width_ratio" => config.space_width_ratio.unwrap_or(2.0),
            _ => {
                return Err(Error::Config(ConfigError {
                    msg: format!("Unsupported float config key '{}' in evolution config", key),
                }));
            }
        };
        Ok(value)
    }

    fn set_config_float(&self, config: &mut Config, key: &str, value: f32) -> Result<(), Error> {
        match key {
            "cho_h_ratio" => config.cho_h_ratio = Some(value),
            "jung_w_ratio" => config.jung_w_ratio = Some(value),
            "jong_w_ratio" => config.jong_w_ratio = Some(value),
            "jung_h_ratio" => config.jung_h_ratio = Some(value),
            "jong_h_ratio" => config.jong_h_ratio = Some(value),
            "char_gap" => config.char_gap = Some(Self::f32_to_u16(key, value)?),
            "cho_cho_gap" => config.cho_cho_gap = Some(Self::f32_to_u16(key, value)?),
            "jung_jung_gap" => config.jung_jung_gap = Some(Self::f32_to_u16(key, value)?),
            "jong_jong_gap" => config.jong_jong_gap = Some(Self::f32_to_u16(key, value)?),
            "cho_jung_gap" => config.cho_jung_gap = Some(Self::f32_to_u16(key, value)?),
            "jung_jong_gap" => config.jung_jong_gap = Some(Self::f32_to_u16(key, value)?),
            "x_sw" => config.x_sw = Some(value),
            "y_sw" => config.y_sw = Some(value),
            "text_size" => config.text_size = Some(Self::f32_to_u16(key, value)?),
            "underdot_y" => config.underdot_y = Some(Self::f32_to_i16(key, value)?),
            "underdot_r_ratio" => config.underdot_r_ratio = Some(value),
            "upperdot_y" => config.upperdot_y = Some(Self::f32_to_i16(key, value)?),
            "upperdot_r_ratio" => config.upperdot_r_ratio = Some(value),
            "glyph_width" => config.glyph_width = Some(Self::f32_to_i16(key, value)?),
            "cap_height" => config.cap_height = Some(Self::f32_to_i16(key, value)?),
            "x_height" => config.x_height = Some(Self::f32_to_i16(key, value)?),
            "baseline" => config.baseline = Some(Self::f32_to_i16(key, value)?),
            "min_gap" => config.min_gap = Some(Self::f32_to_i16(key, value)?),
            "space_width" => config.space_width = Some(Self::f32_to_u16(key, value)?),
            "space_width_ratio" => config.space_width_ratio = Some(value),
            _ => {
                return Err(Error::Config(ConfigError {
                    msg: format!("Unsupported float config key '{}' in evolution config", key),
                }));
            }
        }
        Ok(())
    }

    fn f32_to_u16(key: &str, value: f32) -> Result<u16, Error> {
        if !value.is_finite() {
            return Err(Error::Config(ConfigError {
                msg: format!(
                    "Non-finite numeric value for evolution config key '{}'",
                    key
                ),
            }));
        }
        let rounded = value.round();
        if rounded < 0.0 || rounded > u16::MAX as f32 {
            return Err(Error::Config(ConfigError {
                msg: format!(
                    "Numeric value {} out of range for u16 evolution config key '{}'",
                    value, key
                ),
            }));
        }
        Ok(rounded as u16)
    }

    fn f32_to_i16(key: &str, value: f32) -> Result<i16, Error> {
        if !value.is_finite() {
            return Err(Error::Config(ConfigError {
                msg: format!(
                    "Non-finite numeric value for evolution config key '{}'",
                    key
                ),
            }));
        }
        let rounded = value.round();
        if rounded < i16::MIN as f32 || rounded > i16::MAX as f32 {
            return Err(Error::Config(ConfigError {
                msg: format!(
                    "Numeric value {} out of range for i16 evolution config key '{}'",
                    value, key
                ),
            }));
        }
        Ok(rounded as i16)
    }

    fn apply_kerning_mutation(&mut self, kerning_data: &mut KerningMap) -> Result<(), Error> {
        let axes = match &self.evolution_config.kerning {
            Some(v) => v.clone(),
            None => return Ok(()),
        };
        let mut pairs: Vec<(u16, u16)> = kerning_data.keys().copied().collect();
        pairs.sort_unstable();
        for pair in pairs {
            let prev_kind = match classify_jamo(pair.0) {
                Some(v) => v,
                None => continue,
            };
            let next_kind = match classify_jamo(pair.1) {
                Some(v) => v,
                None => continue,
            };
            let rule = match get_kerning_rule(&axes, prev_kind, next_kind) {
                Some(v) if v.active => v,
                _ => continue,
            };
            if !self.passes_kerning_filter(rule, prev_kind, next_kind, pair)? {
                continue;
            }
            if let Some(v) = kerning_data.get_mut(&pair) {
                let center = *v;
                let sampled = self.sample_number(&rule.mutation_number, center)?;
                *v = sampled;
            }
        }
        Ok(())
    }

    fn passes_kerning_filter(
        &self,
        rule: &KerningMutationRule,
        prev_kind: JamoKind,
        next_kind: JamoKind,
        pair: (u16, u16),
    ) -> Result<bool, Error> {
        let include_pairs = parse_pair_filters(&rule.include_pairs, prev_kind, next_kind)?;
        if !include_pairs.is_empty() && !include_pairs.contains(&pair) {
            return Ok(false);
        }
        let include_prev = parse_char_filters(&rule.include_prev, prev_kind)?;
        if !include_prev.is_empty() && !include_prev.contains(&pair.0) {
            return Ok(false);
        }
        let include_next = parse_char_filters(&rule.include_next, next_kind)?;
        if !include_next.is_empty() && !include_next.contains(&pair.1) {
            return Ok(false);
        }
        let exclude_pairs = parse_pair_filters(&rule.exclude_pairs, prev_kind, next_kind)?;
        if exclude_pairs.contains(&pair) {
            return Ok(false);
        }
        let exclude_prev = parse_char_filters(&rule.exclude_prev, prev_kind)?;
        if exclude_prev.contains(&pair.0) {
            return Ok(false);
        }
        let exclude_next = parse_char_filters(&rule.exclude_next, next_kind)?;
        if exclude_next.contains(&pair.1) {
            return Ok(false);
        }
        Ok(true)
    }

    fn sample_centered_value(&mut self, center: f32, min: f32, max: f32) -> f32 {
        let sigma = (max - min).abs() / 6.0;
        if !sigma.is_finite() || sigma == 0.0 {
            return center;
        }
        let z = self.rng.next_standard_normal() as f32;
        let mut new_value = center + z * sigma;
        if new_value > max {
            new_value = max * 2.0 - new_value;
        }
        if new_value < min {
            new_value = min * 2.0 - new_value;
        }
        new_value
    }

    fn sample_centered_index(&mut self, center: usize, max_index: usize) -> usize {
        if max_index == 0 {
            return 0;
        }
        let sampled = self.sample_centered_value(center as f32, 0.0, max_index as f32);
        let mut idx = sampled.round();
        if !idx.is_finite() {
            idx = center as f32;
        }
        if idx < 0.0 {
            idx = 0.0;
        }
        if idx > max_index as f32 {
            idx = max_index as f32;
        }
        idx as usize
    }

    fn sample_number(&mut self, m: &MutationNumber, center: f32) -> Result<f32, Error> {
        if m.step <= 0.0
            || m.min > m.max
            || !m.step.is_finite()
            || !m.min.is_finite()
            || !m.max.is_finite()
            || !center.is_finite()
        {
            return Err(Error::Config(ConfigError {
                msg: "Invalid numeric mutation range in evolution config".to_string(),
            }));
        }
        let sampled = self.sample_centered_value(center, m.min, m.max);
        let snapped_steps = ((sampled - m.min) / m.step).round();
        let mut value = m.min + snapped_steps * m.step;
        if !value.is_finite() {
            value = center;
        }
        if value < m.min {
            value = m.min;
        }
        if value > m.max {
            value = m.max;
        }
        Ok(value)
    }
}

fn get_checkpoint_dir(evolution_name: &str) -> PathBuf {
    let mut p = get_evolution_dir();
    p.push("checkpoints");
    p.push(evolution_name);
    p
}

fn get_kerning_rule(
    axes: &KerningMutationAxes,
    prev_kind: JamoKind,
    next_kind: JamoKind,
) -> Option<&KerningMutationRule> {
    let targets = match prev_kind {
        JamoKind::Cho => axes.cho.as_ref()?,
        JamoKind::Jung => axes.jung.as_ref()?,
        JamoKind::Jong => axes.jong.as_ref()?,
    };
    match next_kind {
        JamoKind::Cho => targets.cho.as_ref(),
        JamoKind::Jung => targets.jung.as_ref(),
        JamoKind::Jong => targets.jong.as_ref(),
    }
}

fn classify_jamo(codepoint: u16) -> Option<JamoKind> {
    if (0x1100..=0x115f).contains(&codepoint) {
        return Some(JamoKind::Cho);
    }
    if codepoint == 0x119e || (0x1160..=0x11a7).contains(&codepoint) {
        return Some(JamoKind::Jung);
    }
    if codepoint == 0x3181 || (0x11a8..=0x11ff).contains(&codepoint) {
        return Some(JamoKind::Jong);
    }
    None
}

fn parse_char_filters(tokens: &[String], kind: JamoKind) -> Result<HashSet<u16>, Error> {
    let mut out: HashSet<u16> = HashSet::default();
    for token in tokens {
        let codepoint = parse_filter_char(token, kind)?;
        out.insert(codepoint);
    }
    Ok(out)
}

fn parse_pair_filters(
    pairs: &[[String; 2]],
    prev_kind: JamoKind,
    next_kind: JamoKind,
) -> Result<HashSet<(u16, u16)>, Error> {
    let mut out: HashSet<(u16, u16)> = HashSet::default();
    for pair in pairs {
        let prev = parse_filter_char(&pair[0], prev_kind)?;
        let next = parse_filter_char(&pair[1], next_kind)?;
        out.insert((prev, next));
    }
    Ok(out)
}

fn parse_filter_char(token: &str, kind: JamoKind) -> Result<u16, Error> {
    let mut chars = token.chars();
    let c = match chars.next() {
        Some(v) => v,
        None => {
            return Err(Error::Config(ConfigError {
                msg: "Empty char token in evolution kerning filter".to_string(),
            }));
        }
    };
    if chars.next().is_some() {
        return Err(Error::Config(ConfigError {
            msg: format!(
                "Evolution kerning filter token '{}' must be a single character",
                token
            ),
        }));
    }
    let cp_u32 = c as u32;
    if cp_u32 > u16::MAX as u32 {
        return Err(Error::Config(ConfigError {
            msg: format!(
                "Evolution kerning filter token '{}' is outside BMP and unsupported",
                token
            ),
        }));
    }
    let mut codepoint = cp_u32 as u16;
    codepoint = *KERN_JAMO_MAP.get(&codepoint).unwrap_or(&codepoint);
    if kind == JamoKind::Jong {
        codepoint = to_jong_codepoint(codepoint)?;
    }
    Ok(codepoint)
}

fn to_jong_codepoint(codepoint: u16) -> Result<u16, Error> {
    let mut jong = CHO_TO_JONG_MAP
        .get(&codepoint)
        .unwrap_or(&codepoint)
        .to_owned();
    if jong == 0x11bc {
        jong = 0x3181;
    }
    if jong == 0x3181 || (0x11a8..=0x11c2).contains(&jong) {
        return Ok(jong);
    }
    Err(Error::Config(ConfigError {
        msg: format!(
            "Cannot map kerning filter codepoint U+{:04X} to jong",
            codepoint
        ),
    }))
}

fn build_args_from_config(
    mut config: Config,
    kerning_data: KerningMap,
    target_fontname: String,
) -> Result<Args, Error> {
    let source_filename: Option<String> = match &config.source {
        Some(v) => {
            let p = PathBuf::from(v);
            if !p.exists() {
                return Err(Error::Config(ConfigError {
                    msg: format!("{} does not exist.", v),
                }));
            }
            match p.canonicalize() {
                Ok(canonical) => Some(canonical.to_string_lossy().to_string()),
                Err(e) => {
                    return Err(Error::Config(ConfigError {
                        msg: format!("Cannot canonicalize source '{}': {:?}", v, e),
                    }));
                }
            }
        }
        None => None,
    };
    if config.cho_h_ratio.is_none() {
        config.cho_h_ratio = Some(0.0);
    }
    if config.jung_w_ratio.is_none() {
        config.jung_w_ratio = Some(1.0);
    }
    if config.jong_w_ratio.is_none() {
        config.jong_w_ratio = Some(1.0);
    }
    if config.jung_h_ratio.is_none() {
        config.jung_h_ratio = Some(1.0);
    }
    if config.jong_h_ratio.is_none() {
        config.jong_h_ratio = Some(1.0);
    }
    if config.char_gap.is_none() {
        config.char_gap = Some(0);
    }
    if config.cho_cho_gap.is_none() {
        config.cho_cho_gap = Some(0);
    }
    if config.jung_jung_gap.is_none() {
        config.jung_jung_gap = Some(0);
    }
    if config.jong_jong_gap.is_none() {
        config.jong_jong_gap = Some(0);
    }
    if config.cho_jung_gap.is_none() {
        config.cho_jung_gap = Some(0);
    }
    if config.jung_jong_gap.is_none() {
        config.jung_jong_gap = Some(0);
    }
    if config.text_size.is_none() {
        config.text_size = Some(16);
    }
    if config.underdot_y.is_none() {
        config.underdot_y = Some(-300);
    }
    if config.underdot_r_ratio.is_none() {
        config.underdot_r_ratio = Some(0.5);
    }
    if config.upperdot_y.is_none() {
        config.upperdot_y = Some(1800);
    }
    if config.upperdot_r_ratio.is_none() {
        config.upperdot_r_ratio = Some(0.5);
    }
    if config.glyph_width.is_none() {
        config.glyph_width = Some(800);
    }
    if config.baseline.is_none() {
        config.baseline = Some(0);
    }
    if config.x_height.is_none() {
        config.x_height = Some(1500);
    }
    if config.cap_height.is_none() {
        config.cap_height = Some(1800);
    }
    if config.min_gap.is_none() {
        config.min_gap = Some(200);
    }
    if config.x_sw.is_none() {
        config.x_sw = Some(0.2);
    }
    if config.y_sw.is_none() {
        config.y_sw = Some(0.2);
    }
    if config.space_width_ratio.is_none() {
        config.space_width_ratio = Some(2.0);
    }

    let glyph_width = config.glyph_width.unwrap();
    let x_sw = scale_ratio_to_i16("x_sw", config.x_sw.unwrap(), glyph_width)?;
    let y_sw = scale_ratio_to_i16("y_sw", config.y_sw.unwrap(), glyph_width)?;

    Ok(Args {
        source_filename,
        target_fontname,
        cho_type: parse_jamo_type_flags(&config.cho_type),
        jung_type: parse_jamo_type_flags(&config.jung_type),
        jong_type: parse_jamo_type_flags(&config.jong_type),
        cho_h_ratio: config.cho_h_ratio.unwrap(),
        jung_w_ratio: config.jung_w_ratio.unwrap(),
        jong_w_ratio: config.jong_w_ratio.unwrap(),
        jung_h_ratio: config.jung_h_ratio.unwrap(),
        jong_h_ratio: config.jong_h_ratio.unwrap(),
        char_gap: config.char_gap.unwrap(),
        cho_cho_gap: config.cho_cho_gap.unwrap(),
        jung_jung_gap: config.jung_jung_gap.unwrap(),
        jong_jong_gap: config.jong_jong_gap.unwrap(),
        cho_jung_gap: config.cho_jung_gap.unwrap(),
        jung_jong_gap: config.jung_jong_gap.unwrap(),
        x_sw,
        y_sw,
        sw: x_sw,
        text_size: config.text_size.unwrap(),
        underdot_y: config.underdot_y.unwrap(),
        underdot_r_ratio: config.underdot_r_ratio.unwrap(),
        upperdot_y: config.upperdot_y.unwrap(),
        upperdot_r_ratio: config.upperdot_r_ratio.unwrap(),
        baseline: config.baseline.unwrap(),
        x_height: config.x_height.unwrap(),
        cap_height: config.cap_height.unwrap(),
        glyph_width,
        min_gap: config.min_gap.unwrap(),
        kerning_data,
        space_width: config.space_width,
        space_width_ratio: config.space_width_ratio.unwrap(),
    })
}

fn scale_ratio_to_i16(config_key: &str, ratio: f32, glyph_width: i16) -> Result<i16, Error> {
    if !ratio.is_finite() {
        return Err(Error::Config(ConfigError {
            msg: format!("Config key '{}' must be a finite number", config_key),
        }));
    }
    let scaled = ratio * glyph_width as f32;
    if !scaled.is_finite() || scaled < i16::MIN as f32 || scaled > i16::MAX as f32 {
        return Err(Error::Config(ConfigError {
            msg: format!(
                "Config key '{}' produced out-of-range stroke width {} (glyph_width={})",
                config_key, scaled, glyph_width
            ),
        }));
    }
    Ok(scaled as i16)
}

fn parse_jamo_type_flags(type_value: &str) -> u8 {
    let mut flags: u8 = 0;
    for token in type_value
        .split(|c: char| c == ',' || c == '|' || c == '+' || c.is_whitespace())
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
    {
        match token.as_str() {
            "underdot" => flags |= crate::consts::UNDERDOT,
            "upperdot" => flags |= crate::consts::UPPERDOT,
            "underbar" => flags |= crate::consts::UNDERBAR,
            _ => {}
        }
    }
    flags
}

fn collect_hangul_syllable_codepoints(content: &str) -> Vec<u16> {
    let mut codepoints: HashSet<u16> = HashSet::default();
    for ch in content.chars() {
        let cp = ch as u32;
        if (0xAC00..=0xD7A3).contains(&cp) {
            codepoints.insert(cp as u16);
        }
    }
    let mut out: Vec<u16> = codepoints.into_iter().collect();
    out.sort_unstable();
    out
}

fn serialize_kerning_checkpoint(kerning_data: &KerningMap) -> Result<String, Error> {
    let mut pairs: Vec<(u16, u16)> = kerning_data.keys().copied().collect();
    pairs.sort_unstable();
    let mut lines: Vec<String> = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let value = match kerning_data.get(&pair) {
            Some(v) => *v,
            None => continue,
        };
        if value == 0.0 {
            continue;
        }
        let prev = match char::from_u32(pair.0 as u32) {
            Some(v) => v,
            None => {
                return Err(Error::Kerning(crate::error::KerningError {
                    msg: format!(
                        "Cannot serialize kerning key U+{:04X} as a character",
                        pair.0
                    ),
                }));
            }
        };
        let next = match char::from_u32(pair.1 as u32) {
            Some(v) => v,
            None => {
                return Err(Error::Kerning(crate::error::KerningError {
                    msg: format!(
                        "Cannot serialize kerning key U+{:04X} as a character",
                        pair.1
                    ),
                }));
            }
        };
        lines.push(format!("{},{},,{}", prev, next, value));
    }
    Ok(lines.join("\n"))
}

fn write_atomic_string(path: &Path, content: &str) -> Result<(), Error> {
    let file_name = match path.file_name().and_then(|v| v.to_str()) {
        Some(v) => v,
        None => {
            return Err(Error::Font(FontError {
                msg: format!("Invalid checkpoint path {:?}", path),
            }));
        }
    };
    let tmp_path = path.with_file_name(format!("{}.tmp", file_name));
    if let Err(e) = std::fs::write(&tmp_path, content.as_bytes()) {
        return Err(Error::Font(FontError {
            msg: format!(
                "Failed to write temporary checkpoint {:?}: {:?}",
                tmp_path, e
            ),
        }));
    }
    if let Err(e) = std::fs::rename(&tmp_path, path) {
        return Err(Error::Font(FontError {
            msg: format!(
                "Failed to rename temporary checkpoint {:?} to {:?}: {:?}",
                tmp_path, path, e
            ),
        }));
    }
    Ok(())
}

fn default_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|v| v.as_nanos() as u64)
        .unwrap_or(0xA5A5_A5A5_A5A5_A5A5);
    nanos ^ 0xD1B54A32D192ED03
}
