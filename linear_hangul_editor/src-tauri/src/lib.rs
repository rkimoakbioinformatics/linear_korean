mod collision;
mod compose;
mod consts;
mod error;
mod evolution;
mod file;
mod font;
mod glyph;
mod settings;
mod structs;

use crate::compose::*;
use crate::consts::*;
use crate::error::*;
use crate::evolution::{EvolutionCandidate, EvolutionCompileMode, EvolutionEngine};
use crate::file::*;
use crate::font::*;
use crate::structs::*;
use ahash::{HashMap, HashSet};
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;

type KerningMap = HashMap<(u16, u16), f32>;

const EVOLUTION_PREVIEW_FONT_PREFIX: &str = "evolution_preview_";
const EVOLUTION_VARIATION_COUNT: usize = 8;
const EVOLUTION_MAX_COLLISION_ATTEMPTS: usize = 512;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EvolvePreviewItem {
    label: String,
    font_name: String,
    config_data: String,
    kerning_data: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EvolveResponse {
    session_id: u64,
    generation: u64,
    items: Vec<EvolvePreviewItem>,
}

#[derive(Debug, Clone)]
struct EvolveSeedState {
    config: Config,
    kerning_data: KerningMap,
}

#[derive(Debug, Clone)]
struct EvolveSessionState {
    evolution_name: String,
    config_name: String,
    kerning_name: String,
    root_seed: EvolveSeedState,
    current_seeds: Vec<EvolveSeedState>,
    generation: u64,
}

#[derive(Debug, Default)]
struct EvolveRuntimeState {
    next_session_id: u64,
    sessions: std::collections::HashMap<u64, EvolveSessionState>,
}

static EVOLVE_RUNTIME_STATE: OnceLock<Mutex<EvolveRuntimeState>> = OnceLock::new();

fn evolve_runtime_state() -> &'static Mutex<EvolveRuntimeState> {
    EVOLVE_RUNTIME_STATE.get_or_init(|| Mutex::new(EvolveRuntimeState::default()))
}

pub fn compile(args: &Args, glyph_set: &str, check_collision: bool) -> Result<(), Error> {
    compile_internal(args, glyph_set, None, check_collision)
}

pub fn compile_selected_hangul_composites(
    args: &Args,
    glyph_set: &str,
    target_codepoints: &[u16],
    check_collision: bool,
) -> Result<(), Error> {
    compile_internal(args, glyph_set, Some(target_codepoints), check_collision)
}

fn compile_internal(
    args: &Args,
    glyph_set: &str,
    target_codepoints: Option<&[u16]>,
    check_collision: bool,
) -> Result<(), Error> {
    let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        *CONFIG
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = args.clone();
        let font_bytes: Vec<u8>;
        if let Some(source_filename) = &args.source_filename {
            font_bytes = match std::fs::read(source_filename) {
                Ok(v) => v,
                Err(e) => {
                    return Err(Error::Font(FontError {
                        msg: format!("Failed to read source font '{}': {:?}", source_filename, e),
                    }));
                }
            };
        } else {
            font_bytes = Vec::new();
        }
        let (mut font_tables, builder) = get_font_tables_and_builder(&font_bytes, glyph_set)?;
        make_compatibility_jamos(&mut font_tables)?;
        if let Some(target_codepoints) = target_codepoints {
            generate_selected_hangul_composite_glyphs(
                &mut font_tables,
                target_codepoints,
                check_collision,
            )?;
        } else {
            generate_hangul_composite_glyphs(&mut font_tables, check_collision)?;
        }
        modify_maxp(&mut font_tables);
        modify_head_hhea(&mut font_tables)?;
        let font_data = build_font_data(&font_tables, builder);
        let p = get_font_ttf_p(&args.target_fontname);
        let dir = match p.parent() {
            Some(v) => v,
            None => {
                return Err(Error::Font(FontError {
                    msg: format!("Invalid target font path: {:?}", p),
                }));
            }
        };
        if !dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&dir) {
                return Err(Error::Font(FontError {
                    msg: format!("Failed to create output directory {:?}: {:?}", dir, e),
                }));
            }
        }
        if let Err(e) = std::fs::write(&p, font_data) {
            return Err(Error::Font(FontError {
                msg: format!("Failed to write compiled font {:?}: {:?}", p, e),
            }));
        }
        Ok(())
    }));
    match compile_result {
        Ok(result) => result,
        Err(panic_payload) => Err(Error::Font(FontError {
            msg: format!(
                "Unexpected panic while generating glyphs: {}",
                panic_payload_to_message(&panic_payload)
            ),
        })),
    }
}

fn panic_payload_to_message(panic_payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = panic_payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
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

fn collect_hangul_syllable_codepoints(content: &str) -> Vec<u16> {
    let mut codepoints: HashSet<u16> = HashSet::default();
    for ch in content.chars() {
        let codepoint = ch as u32;
        if (0xAC00..=0xD7A3).contains(&codepoint) {
            codepoints.insert(codepoint as u16);
        }
    }
    let mut codepoints: Vec<u16> = codepoints.into_iter().collect();
    codepoints.sort_unstable();
    codepoints
}

fn parse_jamo_type_flags(type_value: &str) -> u8 {
    let mut flags: u8 = 0;
    for token in type_value
        .split(|c: char| c == ',' || c == '|' || c == '+' || c.is_whitespace())
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
    {
        match token.as_str() {
            "underdot" => flags |= UNDERDOT,
            "upperdot" => flags |= UPPERDOT,
            "underbar" => flags |= UNDERBAR,
            _ => {}
        }
    }
    flags
}

pub fn make_woff2(args: &Args) -> Result<(), Error> {
    let ttf_p = get_font_ttf_p(&args.target_fontname);
    let woff2_p = get_font_woff2_p(&args.target_fontname);
    let ttf_data = match std::fs::read(&ttf_p) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Font(FontError {
                msg: format!("Failed to read {:?} before WOFF2 encoding: {:?}", ttf_p, e),
            }));
        }
    };
    let woff2_data = match ttf2woff2::encode(&ttf_data, ttf2woff2::BrotliQuality::default()) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Font(FontError {
                msg: format!("Failed to convert {:?} to WOFF2: {:?}", ttf_p, e),
            }));
        }
    };
    if let Err(e) = std::fs::write(&woff2_p, &woff2_data) {
        return Err(Error::Font(FontError {
            msg: format!("Failed to write WOFF2 {:?}: {:?}", woff2_p, e),
        }));
    }
    Ok(())
}

pub fn get_args(config: &mut Config, kerning_name: &str) -> Result<Args, Error> {
    let config_str = get_config_str(DEFAULT_NAME);
    if !config_str.is_empty() {
        let in_config: Config = match json5::from_str(&config_str) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Error parsing config '{}': {:?}", DEFAULT_NAME, e);
                return Err(Error::Config(ConfigError { msg }));
            }
        };
        if config.source.is_none() {
            config.source = in_config.source;
        }
        if config.cho_type.is_empty() {
            config.cho_type = in_config.cho_type;
        }
        if config.jung_type.is_empty() {
            config.jung_type = in_config.jung_type;
        }
        if config.jong_type.is_empty() {
            config.jong_type = in_config.jong_type;
        }
        if config.cho_h_ratio.is_none() {
            config.cho_h_ratio = in_config.cho_h_ratio;
        }
        if config.jung_h_ratio.is_none() {
            config.jung_h_ratio = in_config.jung_h_ratio;
        }
        if config.jong_h_ratio.is_none() {
            config.jong_h_ratio = in_config.jong_h_ratio;
        }
        if config.char_gap.is_none() {
            config.char_gap = in_config.char_gap;
        }
        if config.cho_cho_gap.is_none() {
            config.cho_cho_gap = in_config.cho_cho_gap;
        }
        if config.jung_jung_gap.is_none() {
            config.jung_jung_gap = in_config.jung_jung_gap;
        }
        if config.jong_jong_gap.is_none() {
            config.jong_jong_gap = in_config.jong_jong_gap;
        }
        if config.cho_jung_gap.is_none() {
            config.cho_jung_gap = in_config.cho_jung_gap;
        }
        if config.jung_jong_gap.is_none() {
            config.jung_jong_gap = in_config.jung_jong_gap;
        }
        if config.x_sw.is_none() {
            config.x_sw = in_config.x_sw;
        }
        if config.y_sw.is_none() {
            config.y_sw = in_config.y_sw;
        }
        if config.text_size.is_none() {
            config.text_size = in_config.text_size;
        }
        if config.underdot_y.is_none() {
            config.underdot_y = in_config.underdot_y;
        }
        if config.underdot_r_ratio.is_none() {
            config.underdot_r_ratio = in_config.underdot_r_ratio;
        }
        if config.upperdot_y.is_none() {
            config.upperdot_y = in_config.upperdot_y;
        }
        if config.upperdot_r_ratio.is_none() {
            config.upperdot_r_ratio = in_config.upperdot_r_ratio;
        }
        if config.glyph_width.is_none() {
            config.glyph_width = in_config.glyph_width;
        }
        if config.cap_height.is_none() {
            config.cap_height = in_config.cap_height;
        }
        if config.x_height.is_none() {
            config.x_height = in_config.x_height;
        }
        if config.baseline.is_none() {
            config.baseline = in_config.baseline;
        }
        if config.min_gap.is_none() {
            config.min_gap = in_config.min_gap;
        }
        if config.space_width.is_none() {
            config.space_width = in_config.space_width;
        }
    }
    let source_filename: Option<String> = match &config.source {
        Some(v) => {
            let p = PathBuf::from(&v);
            if !p.exists() {
                let msg = format!("{} does not exist.", v);
                return Err(Error::Config(ConfigError { msg }));
                //std::process::exit(1);
            }
            let source_p: PathBuf = p.canonicalize().unwrap();
            let source_filename = source_p.to_str().unwrap().to_string();
            Some(source_filename)
        }
        None => None,
    };
    let target_fontname = COMPILED_FONT_NAME.to_string();
    let cho_type: u8 = parse_jamo_type_flags(&config.cho_type);
    let jung_type: u8 = parse_jamo_type_flags(&config.jung_type);
    let jong_type: u8 = parse_jamo_type_flags(&config.jong_type);
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
    if config_str.is_empty() {
        let s = json5::to_string(&config).unwrap();
        let j: Config = serde_json::from_str(&s).unwrap();
        let s = serde_json::to_string_pretty(&j).unwrap();
        _save_config(&s, DEFAULT_NAME);
    }
    let cho_h_ratio: f32 = config.cho_h_ratio.unwrap();
    let jung_w_ratio: f32 = config.jung_w_ratio.unwrap();
    let jong_w_ratio: f32 = config.jong_w_ratio.unwrap();
    let jung_h_ratio: f32 = config.jung_h_ratio.unwrap();
    let jong_h_ratio: f32 = config.jong_h_ratio.unwrap();
    let char_gap: u16 = config.char_gap.unwrap();
    let cho_cho_gap: u16 = config.cho_cho_gap.unwrap();
    let jung_jung_gap: u16 = config.jung_jung_gap.unwrap();
    let jong_jong_gap: u16 = config.jong_jong_gap.unwrap();
    let cho_jung_gap: u16 = config.cho_jung_gap.unwrap();
    let jung_jong_gap: u16 = config.jung_jong_gap.unwrap();
    let text_size: u16 = config.text_size.unwrap();
    let underdot_y: i16 = config.underdot_y.unwrap();
    let underdot_r_ratio: f32 = config.underdot_r_ratio.unwrap();
    let upperdot_y: i16 = config.upperdot_y.unwrap();
    let upperdot_r_ratio: f32 = config.upperdot_r_ratio.unwrap();
    let glyph_width: i16 = config.glyph_width.unwrap();
    let baseline: i16 = config.baseline.unwrap();
    let x_height: i16 = config.x_height.unwrap();
    let cap_height: i16 = config.cap_height.unwrap();
    let min_gap: i16 = config.min_gap.unwrap();
    let x_sw = scale_ratio_to_i16("x_sw", config.x_sw.unwrap(), glyph_width)?;
    let y_sw = scale_ratio_to_i16("y_sw", config.y_sw.unwrap(), glyph_width)?;
    let sw: i16 = x_sw;
    let kerning_data: KerningMap = get_kerning_map(kerning_name)?;
    let space_width: Option<u16> = config.space_width;
    let space_width_ratio: f32 = config.space_width_ratio.unwrap();
    Ok(Args {
        source_filename,
        target_fontname,
        cho_type,
        jung_type,
        jong_type,
        jung_w_ratio,
        jong_w_ratio,
        cho_h_ratio,
        jung_h_ratio,
        jong_h_ratio,
        char_gap,
        cho_cho_gap,
        jung_jung_gap,
        jong_jong_gap,
        cho_jung_gap,
        jung_jong_gap,
        x_sw,
        y_sw,
        sw,
        text_size,
        underdot_y,
        underdot_r_ratio,
        upperdot_y,
        upperdot_r_ratio,
        glyph_width,
        baseline,
        x_height,
        cap_height,
        min_gap,
        kerning_data,
        space_width,
        space_width_ratio,
    })
}

fn parse_kerning_map_from_data(data: &str) -> Result<KerningMap, Error> {
    fn parse_kerning_char(line: &str, token: &str, column_name: &str) -> Result<u16, Error> {
        let mut chars = token.chars();
        let c = match chars.next() {
            Some(v) => v,
            None => {
                let msg = format!(
                    "Error parsing kerning data: empty {} in line '{}'",
                    column_name, line
                );
                return Err(Error::Font(FontError { msg }));
            }
        };
        if chars.next().is_some() {
            let msg = format!(
                "Error parsing kerning data: {} must be a single character in line '{}'",
                column_name, line
            );
            return Err(Error::Font(FontError { msg }));
        }
        let c = c as u16;
        let c = KERN_JAMO_MAP.get(&c).unwrap_or(&c);
        Ok(*c)
    }

    fn parse_kerning_column(line: &str, token: &str, column_idx: usize) -> Result<Vec<u16>, Error> {
        let mut v: Vec<u16> = Vec::new();
        for part in token.split('|') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let mut codepoint = parse_kerning_char(line, part, "glyph column")?;
            if column_idx == 2 {
                codepoint = to_jong_codepoint(line, codepoint)?;
            }
            v.push(codepoint);
        }
        Ok(v)
    }

    fn to_jong_codepoint(line: &str, cho_or_jong: u16) -> Result<u16, Error> {
        let mut jong = CHO_TO_JONG_MAP
            .get(&cho_or_jong)
            .unwrap_or(&cho_or_jong)
            .to_owned();
        if jong == 0x11bc {
            jong = 0x3181;
        }
        let is_jong = jong == 0x3181 || (0x11a8..=0x11c2).contains(&jong);
        if !is_jong {
            let msg = format!(
                "Error parsing kerning data: third column '{}' cannot be mapped to jong",
                line
            );
            return Err(Error::Font(FontError { msg }));
        }
        Ok(jong)
    }

    let mut m: KerningMap = HashMap::default();
    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(",").map(|v| v.trim()).collect();
        if parts.len() != 4 {
            let msg = format!(
                "Error parsing kerning data: expected 4 comma-delimited columns in '{}'",
                line
            );
            return Err(Error::Font(FontError { msg }));
        }
        let glyph_columns: Vec<Vec<u16>> = vec![
            parse_kerning_column(line, parts[0], 0)?,
            parse_kerning_column(line, parts[1], 1)?,
            parse_kerning_column(line, parts[2], 2)?,
        ];
        let kern: f32 = match parts[3].parse::<f32>() {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Error parsing kerning data: {}\n{:?}", line, e);
                return Err(Error::Font(FontError { msg }));
            }
        };
        let mut pair: Option<(u16, u16)> = None;
        for i in 0..glyph_columns.len() {
            if glyph_columns[i].len() > 1 {
                pair = Some((glyph_columns[i][0], glyph_columns[i][1]));
                break;
            }
            if i + 1 < glyph_columns.len()
                && !glyph_columns[i].is_empty()
                && !glyph_columns[i + 1].is_empty()
            {
                pair = Some((glyph_columns[i][0], glyph_columns[i + 1][0]));
                break;
            }
        }
        let (prev, next) = match pair {
            Some(v) => v,
            None => {
                let msg = format!(
                    "Error parsing kerning data: no kerning pair found in first 3 columns '{}'",
                    line
                );
                return Err(Error::Font(FontError { msg }));
            }
        };
        m.insert((prev, next), kern);
    }
    Ok(m)
}

fn get_kerning_map(kerning_name: &str) -> Result<KerningMap, Error> {
    let p = get_kerning_p(kerning_name);
    if !p.exists() {
        return Ok(HashMap::default());
    }
    let data = match std::fs::read_to_string(&p) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Error reading kerning file {:?}: {:?}", p, e);
            return Err(Error::Kerning(KerningError { msg }));
        }
    };
    parse_kerning_map_from_data(&data)
}

fn get_glyph_filename(glyph_set: &str, glyph_name: &str) -> Option<PathBuf> {
    let filename = format!("{}.lua", glyph_name);
    let mut p = get_glyph_set_dir(glyph_set);
    p.push(filename.clone());
    if p.exists() {
        return Some(p);
    } else {
        return None;
    }
}

//
// Toolset
//
#[tauri::command]
fn get_tool_set_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_tool_sets_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let ext = p.extension().unwrap().to_str().unwrap();
        if ext == "toolset" {
            let s = p.file_stem().unwrap().to_str().unwrap();
            v.push(s.to_string());
        }
    }
    v.sort();
    v
}

#[tauri::command]
fn get_tool_set_data(tool_set_name: String) -> Result<ToolSet, Error> {
    crate::file::get_tool_set_data(&tool_set_name)
}

#[tauri::command]
fn delete_tool_set(tool_set_name: String) {
    crate::file::delete_tool_set(&tool_set_name);
}

#[tauri::command]
fn save_tool_set(tool_set: ToolSet, tool_set_name: String) {
    if tool_set.config_name.is_empty()
        || tool_set.kerning_name.is_empty()
        || tool_set.glyph_set.is_empty()
    {
        return;
    }
    let p = get_tool_set_p(&tool_set_name);
    let mut f = std::fs::File::create(&p).unwrap();
    let s = json5::to_string(&tool_set).unwrap();
    f.write_all(s.as_bytes()).unwrap();
}

/*#[tauri::command]
fn copy_tool_set(tool_set_name: String, new_tool_set_name: String) {
    let old_p = get_tool_set_p(&tool_set_name);
    let new_p = get_tool_set_p(&new_tool_set_name);
    if !old_p.exists() {
        return;
    }
    if new_p.exists() {
        std::fs::remove_file(&new_p).unwrap();
    }
    let mut f = std::fs::File::open(&old_p).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let j: String = json5::from_str(&s).unwrap();
    let mut f = std::fs::File::create(&new_p).unwrap();
    let b = j.as_bytes();
    f.write_all(b).unwrap();
}*/

//
// Config
//
#[tauri::command]
fn get_config_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_configs_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        v.push(s);
    }
    v.sort();
    v
}

#[tauri::command]
fn get_config_data(config_name: String) -> String {
    let config_name = if config_name.is_empty() {
        DEFAULT_NAME
    } else {
        config_name.as_str()
    };
    get_config_str(&config_name)
}

#[tauri::command]
fn save_config(config_data: String, config_name: String) -> Result<(), Error> {
    if let Err(e) = json5::from_str::<Config>(&config_data) {
        let target_name = if config_name.is_empty() {
            DEFAULT_NAME.to_string()
        } else {
            config_name.clone()
        };
        let msg = format!("Error parsing config '{}': {:?}", target_name, e);
        return Err(Error::Config(ConfigError { msg }));
    }
    _save_config(&config_data, &config_name);
    Ok(())
}

//
// Evolution config
//
#[tauri::command]
fn get_evolution_config_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_evolution_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let ext = p.extension().and_then(|v| v.to_str()).unwrap_or_default();
        if ext != "json5" {
            continue;
        }
        let s = p.file_stem().unwrap().to_string_lossy().to_string();
        v.push(s);
    }
    v.sort();
    v
}

#[tauri::command]
fn get_evolution_config_data(evolution_name: String) -> String {
    let raw = get_evolution_str(&evolution_name);
    if raw.trim().is_empty() {
        return raw;
    }
    match json5::from_str::<EvolutionConfigFile>(&raw) {
        Ok(parsed) => match serde_json::to_string_pretty(&parsed) {
            Ok(v) => v,
            Err(_) => raw,
        },
        Err(_) => raw,
    }
}

#[tauri::command]
fn save_evolution_config(evolution_data: String, evolution_name: String) -> Result<(), Error> {
    let evolution_name = if evolution_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        evolution_name
    };
    let parsed: EvolutionConfigFile = match json5::from_str(&evolution_data) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!(
                "Error parsing evolution config '{}': {:?}",
                evolution_name, e
            );
            return Err(Error::Config(ConfigError { msg }));
        }
    };
    if let Err(msg) = parsed.validate() {
        return Err(Error::Config(ConfigError {
            msg: format!("Invalid evolution config '{}': {}", evolution_name, msg),
        }));
    }
    let formatted_data = match serde_json::to_string_pretty(&parsed) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Config(ConfigError {
                msg: format!(
                    "Failed to format evolution config '{}' for saving: {:?}",
                    evolution_name, e
                ),
            }));
        }
    };
    _save_evolution_config(&formatted_data, &evolution_name);
    Ok(())
}

#[tauri::command]
fn rename_evolution_config(old_name: String, new_name: String) -> Result<(), Error> {
    let old_name = old_name.trim().to_string();
    let new_name = new_name.trim().to_string();
    if old_name.is_empty() || new_name.is_empty() {
        return Err(Error::Config(ConfigError {
            msg: "Evolution config names cannot be empty".to_string(),
        }));
    }
    if old_name == new_name {
        return Ok(());
    }
    let old_path = get_evolution_p(&old_name);
    if !old_path.exists() {
        return Err(Error::Config(ConfigError {
            msg: format!("Evolution config '{}' does not exist", old_name),
        }));
    }
    let new_path = get_evolution_p(&new_name);
    if new_path.exists() {
        return Err(Error::Config(ConfigError {
            msg: format!("Evolution config '{}' already exists", new_name),
        }));
    }
    match std::fs::rename(&old_path, &new_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Config(ConfigError {
            msg: format!(
                "Failed to rename evolution config '{}' to '{}': {:?}",
                old_name, new_name, e
            ),
        })),
    }
}

#[tauri::command]
fn delete_evolution_config(evolution_name: String) -> Result<(), Error> {
    let evolution_name = evolution_name.trim().to_string();
    if evolution_name.is_empty() {
        return Ok(());
    }
    let path = get_evolution_p(&evolution_name);
    if !path.exists() {
        return Ok(());
    }
    match std::fs::remove_file(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Config(ConfigError {
            msg: format!(
                "Failed to delete evolution config '{}': {:?}",
                evolution_name, e
            ),
        })),
    }
}

fn load_evolve_root_seed(config_name: &str, kerning_name: &str) -> Result<EvolveSeedState, Error> {
    let config_str = get_config_str(config_name);
    if config_str.trim().is_empty() {
        return Err(Error::Config(ConfigError {
            msg: format!("Config '{}' does not exist or is empty", config_name),
        }));
    }
    let config: Config = match json5::from_str(&config_str) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Config(ConfigError {
                msg: format!("Error parsing config '{}': {:?}", config_name, e),
            }));
        }
    };
    let kerning_data = get_kerning_map(kerning_name)?;
    Ok(EvolveSeedState {
        config,
        kerning_data,
    })
}

fn parse_evolve_config_data(config_data: &str) -> Result<Config, Error> {
    match json5::from_str(config_data) {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::Config(ConfigError {
            msg: format!("Error parsing evolve seed config: {:?}", e),
        })),
    }
}

fn serialize_evolve_config_data(config: &Config) -> Result<String, Error> {
    match serde_json::to_string_pretty(config) {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::Config(ConfigError {
            msg: format!("Failed to serialize evolve preview config: {:?}", e),
        })),
    }
}

fn serialize_evolve_kerning_data(kerning_data: &KerningMap) -> String {
    let mut pairs: Vec<(u16, u16)> = kerning_data.keys().copied().collect();
    pairs.sort_unstable();
    let mut lines: Vec<String> = Vec::with_capacity(pairs.len());
    for (prev, next) in pairs {
        let value = match kerning_data.get(&(prev, next)) {
            Some(v) => *v,
            None => continue,
        };
        let prev_char = char::from_u32(prev as u32).unwrap_or('\u{FFFD}');
        let next_char = char::from_u32(next as u32).unwrap_or('\u{FFFD}');
        lines.push(format!("{},{},,{}", prev_char, next_char, value));
    }
    lines.join("\n")
}

fn sanitize_font_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut previous_was_sep = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_sep = false;
            continue;
        }
        if !previous_was_sep {
            out.push('_');
            previous_was_sep = true;
        }
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        "default".to_string()
    } else {
        out
    }
}

fn evolution_preview_font_name(evolution_name: &str, slot_index: usize) -> String {
    format!(
        "{}{}_evo_{}",
        EVOLUTION_PREVIEW_FONT_PREFIX,
        sanitize_font_token(evolution_name),
        slot_index + 1
    )
}

fn cleanup_evolution_preview_fonts() {
    let fonts_dir = get_fonts_dir();
    let entries = match fonts_dir.read_dir() {
        Ok(v) => v,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let metadata = match entry.metadata() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if !metadata.is_dir() {
            continue;
        }
        let font_name = entry.file_name().to_string_lossy().to_string();
        if !font_name.starts_with(EVOLUTION_PREVIEW_FONT_PREFIX) {
            continue;
        }
        let _ = delete_font_dir(&font_name);
    }
}

#[tauri::command]
fn evolve(
    evolution_name: String,
    config_name: String,
    kerning_name: String,
    glyph_set: String,
    content: String,
    check_collision: bool,
    session_id: Option<u64>,
    seed_index: Option<usize>,
    reset_to_root: Option<bool>,
    seed_config_data: Option<String>,
    seed_kerning_data: Option<String>,
) -> Result<EvolveResponse, Error> {
    let evolution_name = if evolution_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        evolution_name
    };
    let config_name = if config_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        config_name
    };
    let kerning_name = if kerning_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        kerning_name
    };
    let reset_to_root = reset_to_root.unwrap_or(false);

    let mut resolved_session_id = session_id;
    let existing_session = if let Some(id) = session_id {
        let runtime = evolve_runtime_state().lock().unwrap();
        match runtime.sessions.get(&id) {
            Some(session)
                if session.evolution_name == evolution_name
                    && session.config_name == config_name
                    && session.kerning_name == kerning_name =>
            {
                Some(session.clone())
            }
            _ => {
                resolved_session_id = None;
                None
            }
        }
    } else {
        None
    };
    let mut session = match existing_session {
        Some(v) => v,
        None => EvolveSessionState {
            evolution_name: evolution_name.clone(),
            config_name: config_name.clone(),
            kerning_name: kerning_name.clone(),
            root_seed: load_evolve_root_seed(&config_name, &kerning_name)?,
            current_seeds: Vec::new(),
            generation: 0,
        },
    };
    let base_seed = if reset_to_root {
        session.root_seed.clone()
    } else if let Some(idx) = seed_index {
        session
            .current_seeds
            .get(idx)
            .cloned()
            .unwrap_or_else(|| session.root_seed.clone())
    } else if let Some(seed) = session.current_seeds.first() {
        seed.clone()
    } else {
        session.root_seed.clone()
    };
    let mut base_seed = base_seed;
    if let Some(config_data) = seed_config_data {
        if !config_data.trim().is_empty() {
            base_seed.config = parse_evolve_config_data(&config_data)?;
        }
    }
    if let Some(kerning_data) = seed_kerning_data {
        if !kerning_data.trim().is_empty() {
            base_seed.kerning_data = parse_kerning_map_from_data(&kerning_data)?;
        } else {
            base_seed.kerning_data = HashMap::default();
        }
    }
    let next_generation = session.generation + 1;
    let mut engine = EvolutionEngine::load_from_file(&evolution_name, None)?;

    cleanup_evolution_preview_fonts();

    let mut items: Vec<EvolvePreviewItem> = Vec::with_capacity(EVOLUTION_VARIATION_COUNT + 1);
    let original_font_name = evolution_preview_font_name(&evolution_name, 0);
    let original_candidate = EvolutionCandidate {
        generation: 0,
        config: base_seed.config.clone(),
        kerning_data: base_seed.kerning_data.clone(),
    };
    engine.generate_font_for_mutation(
        &original_candidate,
        &glyph_set,
        EvolutionCompileMode::Fast {
            content: content.clone(),
        },
        check_collision,
        &original_font_name,
    )?;
    items.push(EvolvePreviewItem {
        label: "Base Variant".to_string(),
        font_name: original_font_name,
        config_data: serialize_evolve_config_data(&base_seed.config)?,
        kerning_data: serialize_evolve_kerning_data(&base_seed.kerning_data),
    });
    let mut next_seeds: Vec<EvolveSeedState> = Vec::with_capacity(EVOLUTION_VARIATION_COUNT + 1);
    next_seeds.push(base_seed.clone());

    let mut accepted_candidates: Vec<EvolutionCandidate> =
        Vec::with_capacity(EVOLUTION_VARIATION_COUNT);
    let mut attempts: usize = 0;
    while accepted_candidates.len() < EVOLUTION_VARIATION_COUNT {
        attempts += 1;
        if check_collision && attempts > EVOLUTION_MAX_COLLISION_ATTEMPTS {
            return Err(Error::Collision(CollisionError {
                msg: format!(
                    "Could not generate {} collision-free evolution variants after {} attempts",
                    EVOLUTION_VARIATION_COUNT, attempts
                ),
                debug: None,
            }));
        }

        let candidate = engine.mutate_once(&base_seed.config, &base_seed.kerning_data)?;
        let mutation_index = accepted_candidates.len() + 1;
        let mutation_font_name = evolution_preview_font_name(&evolution_name, mutation_index);
        match engine.generate_font_for_mutation(
            &candidate,
            &glyph_set,
            EvolutionCompileMode::Fast {
                content: content.clone(),
            },
            check_collision,
            &mutation_font_name,
        ) {
            Ok(()) => {
                next_seeds.push(EvolveSeedState {
                    config: candidate.config.clone(),
                    kerning_data: candidate.kerning_data.clone(),
                });
                accepted_candidates.push(candidate);
                let last_seed = next_seeds.last().unwrap();
                items.push(EvolvePreviewItem {
                    label: format!("Mutation {} · G{}", mutation_index, next_generation),
                    font_name: mutation_font_name,
                    config_data: serialize_evolve_config_data(&last_seed.config)?,
                    kerning_data: serialize_evolve_kerning_data(&last_seed.kerning_data),
                });
            }
            Err(Error::Collision(_)) if check_collision => {
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    if let Some(last_candidate) = accepted_candidates.last() {
        engine.save_checkpoint_last(last_candidate)?;
    }

    session.current_seeds = next_seeds;
    session.generation = next_generation;

    let final_session_id = {
        let mut runtime = evolve_runtime_state().lock().unwrap();
        let id = match resolved_session_id {
            Some(v) => v,
            None => {
                runtime.next_session_id = runtime.next_session_id.saturating_add(1);
                if runtime.next_session_id == 0 {
                    runtime.next_session_id = 1;
                }
                runtime.next_session_id
            }
        };
        runtime.sessions.insert(id, session);
        if runtime.sessions.len() > 16 {
            if let Some(old_id) = runtime.sessions.keys().copied().filter(|v| *v != id).min() {
                runtime.sessions.remove(&old_id);
            }
        }
        id
    };

    Ok(EvolveResponse {
        session_id: final_session_id,
        generation: next_generation,
        items,
    })
}

#[tauri::command]
fn evolve_replace_variant(
    evolution_name: String,
    config_name: String,
    kerning_name: String,
    glyph_set: String,
    content: String,
    check_collision: bool,
    base_config_data: String,
    base_kerning_data: String,
    target_index: usize,
    generation: u64,
) -> Result<EvolvePreviewItem, Error> {
    let evolution_name = if evolution_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        evolution_name
    };
    let config_name = if config_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        config_name
    };
    let kerning_name = if kerning_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        kerning_name
    };

    let root_seed = load_evolve_root_seed(&config_name, &kerning_name)?;
    let base_config = if base_config_data.trim().is_empty() {
        root_seed.config.clone()
    } else {
        parse_evolve_config_data(&base_config_data)?
    };
    let base_kerning = if base_kerning_data.trim().is_empty() {
        root_seed.kerning_data.clone()
    } else {
        parse_kerning_map_from_data(&base_kerning_data)?
    };
    let base_seed = EvolveSeedState {
        config: base_config,
        kerning_data: base_kerning,
    };

    let mut engine = EvolutionEngine::load_from_file(&evolution_name, None)?;
    let mut attempts: usize = 0;
    loop {
        attempts += 1;
        if check_collision && attempts > EVOLUTION_MAX_COLLISION_ATTEMPTS {
            return Err(Error::Collision(CollisionError {
                msg: format!(
                    "Could not generate a collision-free replacement variant after {} attempts",
                    attempts
                ),
                debug: None,
            }));
        }

        let candidate = engine.mutate_once(&base_seed.config, &base_seed.kerning_data)?;
        let font_name = evolution_preview_font_name(&evolution_name, target_index);
        match engine.generate_font_for_mutation(
            &candidate,
            &glyph_set,
            EvolutionCompileMode::Fast {
                content: content.clone(),
            },
            check_collision,
            &font_name,
        ) {
            Ok(()) => {
                let label = if target_index == 0 {
                    format!("Base Variant · G{}", generation)
                } else {
                    format!("Mutation {} · G{}", target_index, generation)
                };
                return Ok(EvolvePreviewItem {
                    label,
                    font_name,
                    config_data: serialize_evolve_config_data(&candidate.config)?,
                    kerning_data: serialize_evolve_kerning_data(&candidate.kerning_data),
                });
            }
            Err(Error::Collision(_)) if check_collision => {
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

#[tauri::command]
fn evolve_render_variant(
    evolution_name: String,
    config_name: String,
    kerning_name: String,
    glyph_set: String,
    content: String,
    check_collision: bool,
    render_config_data: String,
    render_kerning_data: String,
    target_index: usize,
    generation: u64,
    label: Option<String>,
) -> Result<EvolvePreviewItem, Error> {
    let evolution_name = if evolution_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        evolution_name
    };
    let config_name = if config_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        config_name
    };
    let kerning_name = if kerning_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        kerning_name
    };

    let root_seed = load_evolve_root_seed(&config_name, &kerning_name)?;
    let render_config = if render_config_data.trim().is_empty() {
        root_seed.config.clone()
    } else {
        parse_evolve_config_data(&render_config_data)?
    };
    let render_kerning = parse_kerning_map_from_data(&render_kerning_data)?;

    let engine = EvolutionEngine::load_from_file(&evolution_name, None)?;
    let font_name = evolution_preview_font_name(&evolution_name, target_index);
    let candidate = EvolutionCandidate {
        generation,
        config: render_config,
        kerning_data: render_kerning,
    };
    eprintln!(
        "[evolution][render-backend] target_index={} generation={} font_name='{}' source={:?} glyph_width={:?} space_width={:?} space_width_ratio={:?}",
        target_index,
        generation,
        font_name,
        candidate.config.source,
        candidate.config.glyph_width,
        candidate.config.space_width,
        candidate.config.space_width_ratio
    );
    engine.generate_font_for_mutation(
        &candidate,
        &glyph_set,
        EvolutionCompileMode::Fast { content },
        check_collision,
        &font_name,
    )?;

    let resolved_label = label.unwrap_or_else(|| {
        if target_index == 0 {
            format!("Base Variant · G{}", generation)
        } else {
            format!("Mutation {} · G{}", target_index, generation)
        }
    });
    Ok(EvolvePreviewItem {
        label: resolved_label,
        font_name,
        config_data: serialize_evolve_config_data(&candidate.config)?,
        kerning_data: serialize_evolve_kerning_data(&candidate.kerning_data),
    })
}

#[tauri::command]
fn save_evolution_variant_as(
    save_name: String,
    glyph_set: String,
    config_data: String,
    kerning_data: String,
    check_collision: bool,
) -> Result<(), Error> {
    let save_name = save_name.trim().to_string();
    if save_name.is_empty() {
        return Err(Error::Config(ConfigError {
            msg: "Save name is empty".to_string(),
        }));
    }
    let source_glyph_set = glyph_set.trim().to_string();
    if source_glyph_set.is_empty() {
        return Err(Error::Config(ConfigError {
            msg: "Glyph set is empty".to_string(),
        }));
    }

    let mut config = parse_evolve_config_data(&config_data)?;
    let kerning_map = parse_kerning_map_from_data(&kerning_data)?;

    let config_json = match serde_json::to_string_pretty(&config) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Config(ConfigError {
                msg: format!("Failed to serialize config for '{}': {:?}", save_name, e),
            }));
        }
    };
    _save_config(&config_json, &save_name);

    let kerning_serialized = serialize_evolve_kerning_data(&kerning_map);
    let kerning_path = get_kerning_p(&save_name);
    let mut kerning_file = match std::fs::File::create(&kerning_path) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Kerning(KerningError {
                msg: format!("Failed to create kerning file {:?}: {:?}", kerning_path, e),
            }));
        }
    };
    if let Err(e) = kerning_file.write_all(kerning_serialized.as_bytes()) {
        return Err(Error::Kerning(KerningError {
            msg: format!("Failed to write kerning file {:?}: {:?}", kerning_path, e),
        }));
    }

    if source_glyph_set != save_name {
        copy_glyph_set(source_glyph_set.clone(), save_name.clone());
    }

    save_tool_set(
        ToolSet {
            config_name: save_name.clone(),
            kerning_name: save_name.clone(),
            glyph_set: save_name.clone(),
        },
        save_name.clone(),
    );

    let mut args = get_args(&mut config, &save_name)?;
    args.target_fontname = save_name.clone();
    let font_dir = get_font_dir(&save_name);
    if font_dir.exists() {
        delete_font_dir(&save_name)?;
    }
    compile(&args, &save_name, check_collision)?;
    make_woff2(&args)?;
    copy_config_kern_glyph_files(&save_name, &save_name, &save_name, &save_name);

    Ok(())
}

//
// Kerning
//
#[tauri::command]
fn get_kerning_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_kernings_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        v.push(s);
    }
    v
}

#[tauri::command]
fn get_kerning_data(kerning_name: String) -> String {
    get_kerning_str(kerning_name)
}

#[tauri::command]
fn save_kerning_data(kerning_data: String, kerning_name: String) {
    if kerning_data.is_empty() {
        return;
    }
    let p = get_kerning_p(&kerning_name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(kerning_data.as_bytes()).unwrap();
}

//
// Font
//
#[tauri::command]
fn get_font_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_fonts_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        match std::fs::metadata(&p) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    if let Some(s) = p.file_name().and_then(|v| v.to_str()) {
                        v.push(s.to_string());
                    }
                }
            }
            Err(_) => {}
        }
    }
    v.sort();
    v
}

#[tauri::command]
fn get_font_data(font_name: &str) -> Vec<u8> {
    crate::file::get_font_data(font_name)
}

#[tauri::command]
fn delete_font(font_name: String) -> Result<(), Error> {
    delete_font_dir(&font_name)
}

#[tauri::command]
fn delete_glyph_set(glyph_set: String) {
    crate::file::delete_glyph_set(&glyph_set);
}

#[tauri::command]
fn save_font(old_name: String, new_name: String) -> Result<(), Error> {
    let old_dir = get_font_dir(&old_name);
    let new_dir = get_font_dir(&new_name);
    if old_dir.exists() {
        if new_dir.exists() {
            match delete_font_dir(&new_name) {
                Ok(_) => {}
                Err(e) => {
                    return Err(Error::Font(FontError {
                        msg: format!("Error in deleting font {:?}: {:?}", new_name, e),
                    }))
                }
            }
        }
        match std::fs::rename(&old_dir, &new_dir) {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::Font(FontError {
                    msg: format!("Error in renaming {:?} to {:?}: {:?}", old_dir, new_dir, e),
                }))
            }
        }
    }
    for ext in &["ttf", "woff2"] {
        let mut p = get_font_dir(&new_name);
        let mut p2 = get_font_dir(&new_name);
        p.push(format!("{}.{}", old_name, ext));
        p2.push(format!("{}.{}", new_name, ext));
        if p.exists() {
            std::fs::rename(&p, &p2).unwrap();
        }
    }
    Ok(())
}

#[tauri::command]
fn get_glyph_set_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_glyph_sets_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        if !s.starts_with(".") {
            v.push(s);
        }
    }
    v.sort();
    v
}

#[tauri::command]
fn copy_glyph_set(glyph_set: String, new_glyph_set: String) {
    let old_p = get_glyph_set_dir(&glyph_set);
    let new_p = get_glyph_set_dir(&new_glyph_set);
    if !new_p.exists() {
        std::fs::create_dir(&new_p).unwrap();
    }
    let mut els = old_p.read_dir().unwrap();
    while let Some(el) = els.next() {
        let el = el.unwrap();
        let p = el.path();
        let mut np = new_p.clone();
        let s = p.file_name().unwrap().to_str().unwrap();
        np.push(s);
        if s.ends_with(".lua") {
            std::fs::copy(&p, &np).unwrap();
        }
    }
}

#[tauri::command]
fn save_glyph_data(glyph_set: String, glyph_name: String, glyph_data: String) {
    let filename = match get_glyph_filename(&glyph_set, &glyph_name) {
        Some(v) => v,
        None => return,
    };
    let mut f = std::fs::File::create(&filename).unwrap();
    f.write_all(glyph_data.as_bytes()).unwrap();
}

#[tauri::command]
fn get_glyph_data(glyph_set: String, glyph_name: String) -> String {
    let filename = match get_glyph_filename(&glyph_set, &glyph_name) {
        Some(v) => v,
        None => return "No data".to_string(),
    };
    let mut f = std::fs::File::open(&filename).unwrap();
    let mut s1 = String::new();
    f.read_to_string(&mut s1).unwrap();
    s1
}

#[tauri::command]
fn get_content_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_contents_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        v.push(s);
    }
    v.sort();
    v
}

#[tauri::command]
fn get_content(content_name: String) -> String {
    let mut s1: String = String::with_capacity(1024 * 1024);
    let p = get_content_p(&content_name);
    if p.exists() {
        let mut f = std::fs::File::open(&p).unwrap();
        f.read_to_string(&mut s1).unwrap();
    }
    s1
}

#[tauri::command]
fn save_content(content: String, content_name: String) {
    use std::io::Write;
    let p = get_content_p(&content_name);
    let mut f = std::fs::File::create(&p).unwrap();
    let bytes = content.as_bytes();
    f.write_all(bytes).unwrap();
}

#[tauri::command]
fn get_setting(name: String) -> Result<Option<String>, Error> {
    crate::settings::get_setting_value(&name)
}

#[tauri::command]
fn set_setting(name: String, value: String) -> Result<(), Error> {
    crate::settings::set_setting_value(&name, &value)
}

fn copy_config_kern_glyph_files(
    font_name: &str,
    config_name: &str,
    kerning_name: &str,
    glyph_set: &str,
) {
    let font_dir = get_font_dir(font_name);
    // Config
    let old_p = get_config_p(config_name);
    let new_p = get_font_config_p(font_name);
    std::fs::copy(&old_p, &new_p).unwrap();
    // Kerning
    let old_p = get_kerning_p(kerning_name);
    let new_p = get_font_kerning_p(font_name);
    std::fs::copy(&old_p, &new_p).unwrap();
    // Glyph data
    let old_p = get_glyph_set_dir(glyph_set);
    let options = fs_extra::dir::CopyOptions::new().overwrite(true);
    fs_extra::copy_items(&[old_p], &font_dir, &options).unwrap();
    let mut old_p = font_dir.clone();
    old_p.push(glyph_set);
    let mut new_p = font_dir.clone();
    new_p.push(FONT_GLYPH_DATA_DIRNAME);
    std::fs::rename(&old_p, &new_p).unwrap();
}

#[tauri::command]
fn run_compile(
    app: AppHandle,
    config_name: String,
    kerning_name: String,
    glyph_set: String,
    check_collision: bool,
) -> Result<(), Error> {
    use tauri::Emitter;
    let config_name: String = if config_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        config_name.clone()
    };
    let config_str = get_config_str(&config_name);
    let mut config = match json5::from_str(&config_str) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Error in parsing config: {:#?}\n{:#?}", e, config_str);
            return Err(Error::Config(ConfigError { msg }));
        }
    };
    let args = get_args(&mut config, &kerning_name)?;
    *CONFIG
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = args.clone();
    let _ = std::thread::spawn(move || {
        let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Delete the existing generated font folder.
            let font_dir = get_font_dir(COMPILED_FONT_NAME);
            if font_dir.exists() {
                if let Err(e) = delete_font_dir(COMPILED_FONT_NAME) {
                    return Err(Error::Font(FontError {
                        msg: format!(
                            "Error deleting existing font '{}': {:?}",
                            COMPILED_FONT_NAME, e
                        ),
                    }));
                }
            }
            compile(&args, &glyph_set, check_collision)?;
            make_woff2(&args)?;
            copy_config_kern_glyph_files(
                COMPILED_FONT_NAME,
                &config_name,
                &kerning_name,
                &glyph_set,
            );
            Ok::<(), Error>(())
        }));
        match compile_result {
            Ok(Ok(())) => {
                let _ = app.emit("msg", "compile_ended");
            }
            Ok(Err(e)) => {
                if let Error::Collision(CollisionError {
                    debug: Some(debug), ..
                }) = &e
                {
                    let _ = app.emit("collision_debug", debug);
                }
                let _ = app.emit("error", format!("{:?}", e));
            }
            Err(panic_payload) => {
                let panic_msg = panic_payload_to_message(&panic_payload);
                let _ = app.emit(
                    "error",
                    format!("Unexpected panic while compiling glyphs: {}", panic_msg),
                );
            }
        }
    });
    Ok(())
}

#[tauri::command]
fn run_compile_content(
    app: AppHandle,
    config_name: String,
    kerning_name: String,
    glyph_set: String,
    content: String,
    check_collision: bool,
) -> Result<(), Error> {
    use tauri::Emitter;
    let config_name: String = if config_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        config_name.clone()
    };
    let config_str = get_config_str(&config_name);
    let mut config = match json5::from_str(&config_str) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Error in parsing config: {:#?}\n{:#?}", e, config_str);
            return Err(Error::Config(ConfigError { msg }));
        }
    };
    let args = get_args(&mut config, &kerning_name)?;
    *CONFIG
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = args.clone();
    let _ = std::thread::spawn(move || {
        let compile_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let font_dir = get_font_dir(COMPILED_FONT_NAME);
            if font_dir.exists() {
                if let Err(e) = delete_font_dir(COMPILED_FONT_NAME) {
                    return Err(Error::Font(FontError {
                        msg: format!(
                            "Error deleting existing font '{}': {:?}",
                            COMPILED_FONT_NAME, e
                        ),
                    }));
                }
            }
            let target_codepoints = collect_hangul_syllable_codepoints(&content);
            compile_selected_hangul_composites(
                &args,
                &glyph_set,
                &target_codepoints,
                check_collision,
            )?;
            make_woff2(&args)?;
            copy_config_kern_glyph_files(
                COMPILED_FONT_NAME,
                &config_name,
                &kerning_name,
                &glyph_set,
            );
            Ok::<(), Error>(())
        }));
        match compile_result {
            Ok(Ok(())) => {
                let _ = app.emit("msg", "compile_ended");
            }
            Ok(Err(e)) => {
                if let Error::Collision(CollisionError {
                    debug: Some(debug), ..
                }) = &e
                {
                    let _ = app.emit("collision_debug", debug);
                }
                let _ = app.emit("error", format!("{:?}", e));
            }
            Err(panic_payload) => {
                let panic_msg = panic_payload_to_message(&panic_payload);
                let _ = app.emit(
                    "error",
                    format!("Unexpected panic while compiling glyphs: {}", panic_msg),
                );
            }
        }
    });
    Ok(())
}

pub fn just_compile() {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            if let Err(e) = create_data_folders(&app_handle) {
                eprintln!("warning: failed to initialize data folders: {:?}", e);
            }
            if let Err(e) = crate::settings::init_settings_db() {
                eprintln!("warning: failed to initialize settings db: {:?}", e);
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            run_compile,
            run_compile_content,
            // toolset
            get_tool_set_names,
            get_tool_set_data,
            delete_tool_set,
            save_tool_set,
            //copy_tool_set,
            // glyph set
            get_glyph_set_names,
            get_glyph_data,
            delete_glyph_set,
            copy_glyph_set,
            save_glyph_data,
            // config
            get_config_names,
            get_config_data,
            save_config,
            // evolution config
            get_evolution_config_names,
            get_evolution_config_data,
            save_evolution_config,
            rename_evolution_config,
            delete_evolution_config,
            evolve,
            evolve_replace_variant,
            evolve_render_variant,
            save_evolution_variant_as,
            // kerning
            get_kerning_names,
            get_kerning_data,
            save_kerning_data,
            // content
            get_content_names,
            get_content,
            save_content,
            // settings
            get_setting,
            set_setting,
            // font
            get_font_names,
            get_font_data,
            delete_font,
            save_font,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
