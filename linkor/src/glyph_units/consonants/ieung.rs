use crate::consts::*;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use write_fonts::tables::glyf::Glyph;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
    let x_mid = GLYPH_WIDTH / 2;
    let y_mid = GLYPH_HEIGHT / 2;
    let x_c4 = GLYPH_WIDTH;
    let y_c4 = 0;
    let x_c3 = GLYPH_WIDTH;
    let y_c3 = GLYPH_HEIGHT;
    let x_c2 = 0;
    let y_c2 = GLYPH_HEIGHT;
    let x_c1 = 0;
    let y_c1 = 0;
    let curves = vec![
      vec![
        (x_mid, 0, true),
        (x_c1, y_c1, false),
        (0, y_mid, true),
        (x_c2, y_c2, false),
        (x_mid, GLYPH_HEIGHT, true),
        (x_c3, y_c3, false),
        (GLYPH_WIDTH, y_mid, true),
        (x_c4, y_c4, false),
        (x_mid, 0, true),
      ],
      vec![
        (x_mid, sw, true),
        (x_c4 - sw, y_c4 + sw, false),
        (GLYPH_WIDTH - sw, y_mid, true),
        (x_c3 - sw , y_c3 - sw, false),
        (x_mid, GLYPH_HEIGHT - sw, true),
        (x_c2 + sw, y_c2 - sw, false),
        (sw, y_mid, true),
        (x_c1 + sw, y_c1 + sw, false),
        (x_mid, sw, true),
      ]
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x110b, Glyph::Simple(glyph.clone()));
    let glyph = create_glyph_with_points(curves, Sung::Jong);
    m.insert(0x11bc, Glyph::Simple(glyph));
}
