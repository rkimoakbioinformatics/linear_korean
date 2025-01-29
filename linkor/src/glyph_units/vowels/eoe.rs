use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let mut x_max = (GLYPH_WIDTH as f64 * 0.7) as i16;
  let y_mid = GLYPH_HEIGHT / 2;
  let y_top = y_mid + sw / 2;
  let y_bot = y_mid - sw / 2;
  let xbar_mid = (x_max as f32 * 0.5) as i16;
  let xbar_l = xbar_mid - sw / 2;
  let xbar_r = xbar_mid + sw / 2;
  if x_max - sw * 2 < xbar_r {
    x_max = xbar_r + sw * 2;
  }
    let curves = vec![
      vec![
        (x_max, 0, true),
        (x_max - sw, 0, true),
        (x_max - sw, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT, true),
        (x_max, 0, true),
      ],
      vec![
        (xbar_l, 0, true),
        (xbar_l, y_bot, true),
        (0, y_bot, true),
        (0, y_top, true),
        (xbar_l, y_top, true),
        (xbar_l, GLYPH_HEIGHT, true),
        (xbar_r, GLYPH_HEIGHT, true),
        (xbar_r, 0, true),
        (xbar_l, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x1166, Glyph::Simple(glyph));
}
