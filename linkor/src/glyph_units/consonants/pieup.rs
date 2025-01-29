use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let x_13 = GLYPH_WIDTH / 3;
  let x_23 = GLYPH_WIDTH * 2 / 3;
  let x_13_left = x_13 - sw / 2;
  let x_13_right = x_13 + sw / 2;
  let x_23_left = x_23 - sw / 2;
  let x_23_right = x_23 + sw / 2;
    let curves = vec![
      vec![
        (0, 0, true),
        (0, sw, true),
        (x_13_left, sw, true),
        (x_13_left, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT - sw, true),
        (x_23_right, GLYPH_HEIGHT - sw, true),
        (x_23_right, sw, true),
        (GLYPH_WIDTH, sw, true),
        (GLYPH_WIDTH, 0, true),
        (0, 0, true),
      ],
      vec![
        (x_13_right, sw, true),
        (x_23_left, sw, true),
        (x_23_left, GLYPH_HEIGHT - sw, true),
        (x_13_right, GLYPH_HEIGHT - sw, true),
        (x_13_right, sw, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x1111, Glyph::Simple(glyph.clone()));
    let glyph = create_glyph_with_points(curves, Sung::Jong);
    m.insert(0x11c1, Glyph::Simple(glyph));
}
