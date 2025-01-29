use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let x_max = if GLYPH_WIDTH < MIN_GAP * 7 {
    MIN_GAP * 4
  } else {
    GLYPH_WIDTH
  };
  let x_mid = x_max / 2;
  let x_14 = x_max * 2 / 7;
  let x_34 = x_max * 5 / 7;
    let curves = vec![
      vec![
        (0, 0, true),
        (x_14 - sw / 2, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT, true),
        (x_max, GLYPH_HEIGHT - sw, true),
        (x_34 + sw / 2, GLYPH_HEIGHT - sw, true),
        (x_max, 0, true),
        (x_max - sw, 0, true),
        (x_34, GLYPH_HEIGHT - sw * 4, true),
        (x_mid, 0, true),
        (x_14, GLYPH_HEIGHT - sw * 4, true),
        (sw, 0, true),
        (0, 0, true),
      ],
      vec![
        (x_14 + sw / 2, GLYPH_HEIGHT - sw, true),
        (x_mid, GLYPH_HEIGHT - sw * 5, true),
        (x_34 - sw / 2, GLYPH_HEIGHT - sw, true),
        (x_14 + sw / 2, GLYPH_HEIGHT - sw, true),
      ]
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x110d, Glyph::Simple(glyph.clone()));
    let _glyph = create_glyph_with_points(curves, Sung::Jong);
}
