use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let mut x_max = (GLYPH_WIDTH as f64 * 1.2) as i16;
  let xm = x_max / 2;
  let xl = std::cmp::max(xm - sw / 2, sw);
  let xr = xl + sw;
  x_max = std::cmp::max(x_max, xr + sw);
    let curves = vec![
      vec![
        (0, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT - sw, true),
        (xr, GLYPH_HEIGHT - sw, true),
        (xr, 0, true),
        (xl, 0, true),
        (xl, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x116e, Glyph::Simple(glyph));
}
