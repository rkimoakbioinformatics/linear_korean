use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let mid_x = GLYPH_WIDTH / 2;
  let mid_left = mid_x - sw / 2;
  let mid_right = mid_x + sw / 2;
    let curves = vec![
      vec![
        (0, 0, true),
        (0, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT - sw, true),
        (mid_right, GLYPH_HEIGHT - sw, true),
        (mid_right, sw, true),
        (GLYPH_WIDTH, sw, true),
        (GLYPH_WIDTH, 0, true),
        (0, 0, true),
      ],
      vec![
        (sw, sw, true),
        (mid_left, sw, true),
        (mid_left, GLYPH_HEIGHT - sw, true),
        (sw, GLYPH_HEIGHT - sw, true),
        (sw, sw, true),
      ]
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x1104, Glyph::Simple(glyph.clone()));
    let _glyph = create_glyph_with_points(curves, Sung::Jong);
}
