use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let mut x_max = (GLYPH_WIDTH as f64 * 0.5) as i16;
  let bar1_mid = (GLYPH_HEIGHT as f32 * 0.7) as i16;
  let bar2_mid = (GLYPH_HEIGHT as f32 * 0.3) as i16;
  let bar1_bot = bar1_mid - sw / 2;
  let bar1_top = bar1_mid + sw / 2;
  let bar2_bot = bar2_mid - sw / 2;
  let bar2_top = bar2_mid + sw / 2;
  let xbar_mid = (x_max as f32 * 0.5) as i16;
  let xbar_l = std::cmp::max(xbar_mid - sw / 2, sw + sw / 2);
  let xbar_r = xbar_l + sw;
  x_max = std::cmp::max(x_max, xbar_r + sw * 2);
    let curves = vec![
      vec![
        (x_max, 0, true),
        (x_max - sw, 0, true),
        (x_max - sw, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT, true),
        (x_max, 0, true),
      ],
      vec![
        (xbar_r, 0, true),
        (xbar_l, 0, true),
        (xbar_l, bar2_bot, true),
        (0, bar2_bot, true),
        (0, bar2_top, true),
        (xbar_l, bar2_top, true),
        (xbar_l, bar1_bot, true),
        (0, bar1_bot, true),
        (0, bar1_top, true),
        (xbar_l, bar1_top, true),
        (xbar_l, GLYPH_HEIGHT, true),
        (xbar_r, GLYPH_HEIGHT, true),
        (xbar_r, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x1168, Glyph::Simple(glyph));
}
