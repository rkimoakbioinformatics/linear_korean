use write_fonts::tables::{cmap::Cmap, glyf::Glyph, head::Head, hhea::Hhea, hmtx::Hmtx, maxp::Maxp, name::Name, post::Post};
use ahash::HashMap;

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

pub enum Sung {
    Cho,
    Jung,
    Jong,
}

#[derive(Default, Clone, Debug)]
pub struct Args {
  pub source_filename: String,
  pub target_filename: String,
  pub woff2_filename: String,
  pub jung_type: u8,
  pub jong_type: u8,
  pub jung_h_ratio: f32,
  pub jong_h_ratio: f32,
  pub char_gap: u16,
  pub cho_gap: u16,
  pub jung_gap: u16,
  pub jong_gap: u16,
  pub sw: i16, // stroke width
  pub text_size: u16,
}