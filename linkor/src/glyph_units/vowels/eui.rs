use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let x_max = (GLYPH_WIDTH as f64 * 1.0) as i16;
  let y = (GLYPH_HEIGHT as f32 * 0.3) as i16;
  let yb = y - sw / 2;
  let yt = y + sw / 2;
    let curves = vec![
      vec![
        (0, yb, true),
        (0, yt, true),
        (x_max, yt, true),
        (x_max, yb, true),
        (0, yb, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x1173, Glyph::Simple(glyph));
}
