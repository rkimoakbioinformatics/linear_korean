use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let xl = 0;
  let xr = sw;
    let curves = vec![
      vec![
        (xl, 0, true),
        (xl, GLYPH_HEIGHT, true),
        (xr, GLYPH_HEIGHT, true),
        (xr, 0, true),
        (0, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x1175, Glyph::Simple(glyph));
}
