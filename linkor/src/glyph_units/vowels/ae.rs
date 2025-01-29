use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let mut x_max = (GLYPH_WIDTH as f64 * 0.5) as i16;
  let mut xbar_l = sw;
  let mut xbar_r = x_max - sw;
  if xbar_r - xbar_l < sw {
    xbar_l = sw;
    xbar_r = sw * 2;
    x_max = sw * 3;
  }
  let bar1_mid = GLYPH_HEIGHT / 2;
  let bar1_bot = bar1_mid - sw / 2;
  let bar1_top = bar1_mid + sw / 2;
    let curves = vec![
      vec![
        (0, 0, true),
        (0, GLYPH_HEIGHT, true),
        (xbar_l, GLYPH_HEIGHT, true),
        (xbar_l, bar1_top, true),
        (xbar_r, bar1_top, true),
        (xbar_r, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT, true),
        (x_max, 0, true),
        (x_max - sw, 0, true),
        (x_max - sw, bar1_bot, true),
        (xbar_l, bar1_bot, true),
        (xbar_l, 0, true),
        (0, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x1162, Glyph::Simple(glyph));
}
