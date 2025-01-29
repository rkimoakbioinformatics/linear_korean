use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let y_mid = GLYPH_HEIGHT / 2;
  let y_mid_bot = y_mid - sw / 2;
  let y_mid_top = y_mid + sw / 2;
  let x_mid = GLYPH_WIDTH / 2;
  let x_mid_left = x_mid - sw / 2;
  let x_mid_right = x_mid + sw / 2;
    let curves = vec![
      vec![
        (0, 0, true),
        (0, GLYPH_HEIGHT, true),
        (sw, GLYPH_HEIGHT, true),
        (sw, y_mid_top, true),
        (x_mid_left, y_mid_top, true),
        (x_mid_left, GLYPH_HEIGHT, true),
        (x_mid_right, GLYPH_HEIGHT, true),
        (x_mid_right, y_mid_top, true),
        (GLYPH_WIDTH - sw, y_mid_top, true),
        (GLYPH_WIDTH - sw, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, 0, true),
        (0, 0, true),
      ],
      vec![
        (sw, sw, true),
        (x_mid_left, sw, true),
        (x_mid_left, y_mid_bot, true),
        (sw, y_mid_bot, true),
        (sw, sw, true),
      ],
      vec![
        (x_mid_right, sw, true),
        (GLYPH_WIDTH - sw, sw, true),
        (GLYPH_WIDTH - sw, y_mid_bot, true),
        (x_mid_right, y_mid_bot, true),
        (x_mid_right, sw, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x1108, Glyph::Simple(glyph.clone()));
    let _glyph = create_glyph_with_points(curves, Sung::Jong);
}
