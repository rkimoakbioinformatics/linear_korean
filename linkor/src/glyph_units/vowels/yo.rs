use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let mut x_max = (GLYPH_WIDTH as f64 * 0.8) as i16;
  let xbar2_m = (x_max as f32 * 0.3) as i16;
  let xbar2_l = std::cmp::max(xbar2_m - sw / 2, sw * 2);
  let xbar2_r = xbar2_l + sw;
  let xbar1_m = (x_max as f32 * 0.7) as i16;
  let xbar1_l = std::cmp::max(xbar1_m - sw / 2, xbar2_r + sw);
  let xbar1_r = xbar1_l + sw;
  x_max = std::cmp::max(x_max, xbar1_r + sw * 2);
    let curves = vec![
      vec![
        (0, 0, true),
        (0, sw, true),
        (xbar1_l, sw, true),
        (xbar1_l, GLYPH_HEIGHT, true),
        (xbar1_r, GLYPH_HEIGHT, true),
        (xbar1_r, sw, true),
        (xbar2_l, sw, true),
        (xbar2_l, GLYPH_HEIGHT, true),
        (xbar2_r, GLYPH_HEIGHT, true),
        (xbar2_r, sw, true),
        (x_max, sw, true),
        (x_max, 0, true),
        (0, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x116d, Glyph::Simple(glyph));
}
