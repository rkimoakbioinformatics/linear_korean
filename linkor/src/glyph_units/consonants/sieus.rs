use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let x_mid = GLYPH_WIDTH / 2;
    let curves = vec![
      vec![
        (0, 0, true),
        (x_mid, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, 0, true),
        (GLYPH_WIDTH - sw, 0, true),
        (x_mid, GLYPH_HEIGHT - sw * 3, true),
        (sw, 0, true),
        (0, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x1109, Glyph::Simple(glyph.clone()));
    let glyph = create_glyph_with_points(curves, Sung::Jong);
    m.insert(0x11ba, Glyph::Simple(glyph));
}
