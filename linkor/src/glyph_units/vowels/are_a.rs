use crate::consts::*;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use write_fonts::tables::glyf::Glyph;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
    let x_max = (GLYPH_WIDTH as f32 * 0.8) as i16;
    let x_mid = x_max / 2;
    let y = (GLYPH_HEIGHT as f32 * 0.3) as i16;
    let small_circle_r = sw * 2;
    let small_circle_x_c4 = x_mid + small_circle_r;
    let small_circle_y_c4 = y - small_circle_r;
    let small_circle_x_c3 = x_mid + small_circle_r;
    let small_circle_y_c3 = y + small_circle_r;
    let small_circle_x_c2 = x_mid - small_circle_r;
    let small_circle_y_c2 = y + small_circle_r;
    let small_circle_x_c1 = x_mid - small_circle_r;
    let small_circle_y_c1 = y - small_circle_r;
    let curves = vec![
        vec![
            (x_mid, y - small_circle_r, true),
            (small_circle_x_c1, small_circle_y_c1, false),
            (x_mid - small_circle_r, y, true),
            (small_circle_x_c2, small_circle_y_c2, false),
            (x_mid, y + small_circle_r, true),
            (small_circle_x_c3, small_circle_y_c3, false),
            (x_mid + small_circle_r, y, true),
            (small_circle_x_c4, small_circle_y_c4, false),
            (x_mid, y - small_circle_r, true),
        ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Jung);
    m.insert(0x119e, Glyph::Simple(glyph.clone()));
}
