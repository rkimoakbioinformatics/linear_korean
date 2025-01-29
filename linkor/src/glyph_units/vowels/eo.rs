use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let x_max = (GLYPH_WIDTH as f64 * 0.5) as i16;
  let y_mid = GLYPH_HEIGHT / 2;
  let y_top = y_mid + sw / 2;
  let y_bot = y_mid - sw / 2;
    let curves = vec![
      vec![
        (x_max, 0, true),
        (x_max - sw, 0, true),
        (x_max - sw, y_bot, true),
        (0, y_bot, true),
        (0, y_top, true),
        (x_max - sw, y_top, true),
        (x_max - sw, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT, true),
        (x_max, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x1165, Glyph::Simple(glyph));
}
