use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
    let curves = vec![
      vec![
        (0, 0, true),
        (0, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT - sw, true),
        (sw, GLYPH_HEIGHT - sw, true),
        (sw, sw, true),
        (GLYPH_WIDTH, sw, true),
        (GLYPH_WIDTH, 0, true),
        (0, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x1103, Glyph::Simple(glyph.clone()));
    let glyph = create_glyph_with_points(curves, Sung::Jong);
    m.insert(0x11ae, Glyph::Simple(glyph));
}
