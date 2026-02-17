use ahash::HashMap;
use serde::Deserialize;
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
    pub cho_gap: u16,
    pub jung_gap: u16,
    pub jong_gap: u16,
    pub sw_ratio: f32, // stroke width ratio
    pub sw: i16,       // stroke width
    pub text_size: u16,
    pub underdot_y: i16,
    pub underdot_r_ratio: f32,
    pub baseline: i16,
    pub x_height: i16,
    pub cap_height: i16,
    pub glyph_width: i16,
    pub min_gap: i16,
    pub kerning_data: crate::KerningMap,
    pub space_width: Option<u16>,
    pub space_width_ratio: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub source: Option<String>,
    pub cho_type: Vec<String>,
    pub jung_type: Vec<String>,
    pub jong_type: Vec<String>,
    pub jung_w_ratio: Option<f32>,
    pub jong_w_ratio: Option<f32>,
    pub cho_h_ratio: Option<f32>,
    pub jung_h_ratio: Option<f32>,
    pub jong_h_ratio: Option<f32>,
    pub char_gap: Option<u16>,
    pub cho_gap: Option<u16>,
    pub jung_gap: Option<u16>,
    pub jong_gap: Option<u16>,
    pub sw_ratio: Option<f32>,
    pub text_size: Option<u16>,
    pub underdot_y: Option<i16>,
    pub underdot_r_ratio: Option<f32>,
    pub glyph_width: Option<i16>,
    pub cap_height: Option<i16>,
    pub x_height: Option<i16>,
    pub baseline: Option<i16>,
    pub min_gap: Option<i16>,
    pub space_width: Option<u16>,
    pub space_width_ratio: Option<f32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolSet {
    pub config_name: String,
    pub kerning_name: String,
    pub glyph_set: String,
}
