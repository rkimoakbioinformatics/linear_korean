use ahash::HashMap;
use serde::Deserialize;
use std::collections::BTreeMap;
use write_fonts::tables::{
    cmap::Cmap, glyf::Glyph, head::Head, hhea::Hhea, hmtx::Hmtx, maxp::Maxp, name::Name, post::Post,
};

pub struct FontTables {
    pub head: Head,
    pub cmap: Cmap,
    pub hhea: Hhea,
    pub hmtx: Hmtx,
    pub maxp: Maxp,
    pub name: Name,
    pub post: Post,
    pub codepoint_to_glyph_id: HashMap<u16, u16>,
    pub glyphs: Vec<Glyph>,
    pub glyph_names: Vec<String>,
}

#[derive(PartialEq)]
pub enum Sung {
    Cho,
    Jung,
    Jong,
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct Args {
    pub source_filename: Option<String>,
    pub target_fontname: String,
    pub cho_type: u8,
    pub jung_type: u8,
    pub jong_type: u8,
    pub cho_h_ratio: f32,
    pub jung_w_ratio: f32,
    pub jong_w_ratio: f32,
    pub jung_h_ratio: f32,
    pub jong_h_ratio: f32,
    pub char_gap: u16,
    pub cho_cho_gap: u16,
    pub jung_jung_gap: u16,
    pub jong_jong_gap: u16,
    pub cho_jung_gap: u16,
    pub jung_jong_gap: u16,
    pub x_sw: i16, // x-axis stroke width ratio
    pub y_sw: i16, // y-axis stroke width ratio
    pub sw: i16,   // stroke width
    pub text_size: u16,
    pub underdot_y: i16,
    pub underdot_r_ratio: f32,
    pub upperdot_y: i16,
    pub upperdot_r_ratio: f32,
    pub baseline: i16,
    pub x_height: i16,
    pub cap_height: i16,
    pub glyph_width: i16,
    pub min_gap: i16,
    pub kerning_data: crate::KerningMap,
    pub space_width: Option<u16>,
    pub space_width_ratio: f32,
    pub lua_script_variables: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub source: Option<String>,
    pub cho_type: String,
    pub jung_type: String,
    pub jong_type: String,
    pub jung_w_ratio: Option<f32>,
    pub jong_w_ratio: Option<f32>,
    pub cho_h_ratio: Option<f32>,
    pub jung_h_ratio: Option<f32>,
    pub jong_h_ratio: Option<f32>,
    pub char_gap: Option<u16>,
    pub cho_cho_gap: Option<u16>,
    pub jung_jung_gap: Option<u16>,
    pub jong_jong_gap: Option<u16>,
    pub cho_jung_gap: Option<u16>,
    pub jung_jong_gap: Option<u16>,
    pub x_sw: Option<f32>,
    pub y_sw: Option<f32>,
    pub text_size: Option<u16>,
    pub underdot_y: Option<i16>,
    pub underdot_r_ratio: Option<f32>,
    pub upperdot_y: Option<i16>,
    pub upperdot_r_ratio: Option<f32>,
    pub glyph_width: Option<i16>,
    pub cap_height: Option<i16>,
    pub x_height: Option<i16>,
    pub baseline: Option<i16>,
    pub min_gap: Option<i16>,
    pub space_width: Option<u16>,
    pub space_width_ratio: Option<f32>,
    #[serde(default)]
    pub lua_variables: BTreeMap<String, f32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolSet {
    pub config_name: String,
    pub kerning_name: String,
    pub glyph_set: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvolutionConfigFile {
    pub version: u32,
    #[serde(default)]
    pub kerning: Option<KerningMutationAxes>,
    #[serde(default)]
    pub config: Option<BTreeMap<String, ConfigMutationRule>>,
    #[serde(default)]
    pub lua_variables: Option<LuaVariableMutationFilter>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LuaVariableMutationFilter {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KerningMutationAxes {
    #[serde(default)]
    pub cho: Option<KerningMutationTargets>,
    #[serde(default)]
    pub jung: Option<KerningMutationTargets>,
    #[serde(default)]
    pub jong: Option<KerningMutationTargets>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KerningMutationTargets {
    #[serde(default)]
    pub cho: Option<KerningMutationRule>,
    #[serde(default)]
    pub jung: Option<KerningMutationRule>,
    #[serde(default)]
    pub jong: Option<KerningMutationRule>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KerningMutationRule {
    #[serde(default)]
    pub active: bool,
    pub mutation_number: MutationNumber,
    #[serde(default)]
    pub include_prev: Vec<String>,
    #[serde(default)]
    pub include_next: Vec<String>,
    #[serde(default)]
    pub include_pairs: Vec<[String; 2]>,
    #[serde(default)]
    pub exclude_prev: Vec<String>,
    #[serde(default)]
    pub exclude_next: Vec<String>,
    #[serde(default)]
    pub exclude_pairs: Vec<[String; 2]>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationNumber {
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationString {
    pub options: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConfigMutationRule {
    String { mutation_string: MutationString },
    Float { mutation_number: MutationNumber },
}

impl EvolutionConfigFile {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!(
                "Unsupported evolution config version {} (expected 1)",
                self.version
            ));
        }
        if let Some(kerning) = &self.kerning {
            if let Some(targets) = &kerning.cho {
                Self::validate_kerning_targets("kerning.cho", targets)?;
            }
            if let Some(targets) = &kerning.jung {
                Self::validate_kerning_targets("kerning.jung", targets)?;
            }
            if let Some(targets) = &kerning.jong {
                Self::validate_kerning_targets("kerning.jong", targets)?;
            }
        }
        if let Some(config) = &self.config {
            for (name, rule) in config {
                if name.trim().is_empty() {
                    return Err("config key cannot be empty".to_string());
                }
                match rule {
                    ConfigMutationRule::String { mutation_string } => {
                        if mutation_string.options.is_empty() {
                            return Err(format!(
                                "config.{}.mutation_string.options cannot be empty",
                                name
                            ));
                        }
                    }
                    ConfigMutationRule::Float { mutation_number } => {
                        Self::validate_mutation_number(
                            &format!("config.{}.mutation_number", name),
                            mutation_number,
                        )?;
                    }
                }
            }
        }
        if let Some(lua_variables) = &self.lua_variables {
            Self::validate_lua_variable_names("lua_variables.include", &lua_variables.include)?;
            Self::validate_lua_variable_names("lua_variables.exclude", &lua_variables.exclude)?;
        }
        Ok(())
    }

    fn validate_kerning_targets(
        path: &str,
        targets: &KerningMutationTargets,
    ) -> Result<(), String> {
        if let Some(rule) = &targets.cho {
            Self::validate_kerning_rule(&format!("{}.cho", path), rule)?;
        }
        if let Some(rule) = &targets.jung {
            Self::validate_kerning_rule(&format!("{}.jung", path), rule)?;
        }
        if let Some(rule) = &targets.jong {
            Self::validate_kerning_rule(&format!("{}.jong", path), rule)?;
        }
        Ok(())
    }

    fn validate_kerning_rule(path: &str, rule: &KerningMutationRule) -> Result<(), String> {
        Self::validate_mutation_number(
            &format!("{}.mutation_number", path),
            &rule.mutation_number,
        )?;
        Self::validate_char_list(&format!("{}.include_prev", path), &rule.include_prev)?;
        Self::validate_char_list(&format!("{}.include_next", path), &rule.include_next)?;
        Self::validate_pairs(&format!("{}.include_pairs", path), &rule.include_pairs)?;
        Self::validate_char_list(&format!("{}.exclude_prev", path), &rule.exclude_prev)?;
        Self::validate_char_list(&format!("{}.exclude_next", path), &rule.exclude_next)?;
        Self::validate_pairs(&format!("{}.exclude_pairs", path), &rule.exclude_pairs)?;
        Ok(())
    }

    fn validate_mutation_number(path: &str, m: &MutationNumber) -> Result<(), String> {
        if !m.min.is_finite() || !m.max.is_finite() || !m.step.is_finite() {
            return Err(format!("{} must contain finite numeric values", path));
        }
        if m.min > m.max {
            return Err(format!("{} has min > max", path));
        }
        if m.step <= 0.0 {
            return Err(format!("{} has non-positive step", path));
        }
        Ok(())
    }

    fn validate_char_list(path: &str, list: &[String]) -> Result<(), String> {
        for item in list {
            if item.chars().count() != 1 {
                return Err(format!(
                    "{} contains '{}' but each item must be a single character",
                    path, item
                ));
            }
        }
        Ok(())
    }

    fn validate_pairs(path: &str, pairs: &[[String; 2]]) -> Result<(), String> {
        for pair in pairs {
            if pair[0].chars().count() != 1 || pair[1].chars().count() != 1 {
                return Err(format!(
                    "{} contains [{:?}, {:?}] but each pair item must be a single character",
                    path, pair[0], pair[1]
                ));
            }
        }
        Ok(())
    }

    fn validate_lua_variable_names(path: &str, list: &[String]) -> Result<(), String> {
        for item in list {
            let name = item.trim();
            if name.is_empty() {
                return Err(format!("{} contains an empty variable name", path));
            }
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(format!(
                    "{} contains '{}' but variable names must use ASCII letters/digits/_ only",
                    path, item
                ));
            }
        }
        Ok(())
    }
}
