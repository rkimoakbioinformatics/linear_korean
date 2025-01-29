use crate::consts::*;
use write_fonts::tables::glyf::Glyph;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
  let x_max = (GLYPH_WIDTH as f64 * 0.5) as i16;
  let bar1_mid = (GLYPH_HEIGHT as f32 * 0.7) as i16;
  let bar2_mid = (GLYPH_HEIGHT as f32 * 0.3) as i16;
  let bar1_bot = bar1_mid - sw / 2;
  let bar1_top = bar1_mid + sw / 2;
  let bar2_bot = bar2_mid - sw / 2;
  let bar2_top = bar2_mid + sw / 2;
    let curves = vec![
      vec![
        (0, 0, true),
        (0, GLYPH_HEIGHT, true),
        (sw, GLYPH_HEIGHT, true),
        (sw, bar1_top, true),
        (x_max, bar1_top, true),
        (x_max, bar1_bot, true),
        (sw, bar1_bot, true),
        (sw, bar2_top, true),
        (x_max, bar2_top, true),
        (x_max, bar2_bot, true),
        (sw, bar2_bot, true),
        (sw, 0, true),
        (0, 0, true),
      ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x1163, Glyph::Simple(glyph));
}
