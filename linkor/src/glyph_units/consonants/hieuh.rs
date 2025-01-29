use crate::consts::*;
use crate::glyph::create_glyph_with_points;
use ahash::HashMap;
use write_fonts::tables::glyf::Glyph;
use crate::structs::Sung;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
    let x_mid = GLYPH_WIDTH / 2;
    let top_circle_top = GLYPH_HEIGHT;
    let top_circle_bottom = GLYPH_HEIGHT - sw;
    let top_circle_r = std::cmp::max(sw / 2, MIN_GAP);
    let top_circle_x_c4 = x_mid + top_circle_r;
    let top_circle_y_c4 = top_circle_bottom;
    let top_circle_x_c3 = x_mid + top_circle_r;
    let top_circle_y_c3 = top_circle_top;
    let top_circle_x_c2 = x_mid - top_circle_r;
    let top_circle_y_c2 = top_circle_top;
    let top_circle_x_c1 = x_mid - top_circle_r;
    let top_circle_y_c1 = top_circle_bottom;
    let top_circle_y_mid = top_circle_top - (top_circle_top - top_circle_bottom) / 2;
    let bar_top = std::cmp::min((GLYPH_HEIGHT as f64 * 0.8) as i16, top_circle_bottom - MIN_GAP);
    let bar_bottom = bar_top - sw;
    let circle_top = std::cmp::min((GLYPH_HEIGHT as f64 * 0.7) as i16, bar_bottom - MIN_GAP);
    let circle_y_mid = circle_top / 2;
    let circle_x_c4 = GLYPH_WIDTH;
    let circle_y_c4 = 0;
    let circle_x_c3 = GLYPH_WIDTH;
    let circle_y_c3 = circle_top;
    let circle_x_c2 = 0;
    let circle_y_c2 = circle_top;
    let circle_x_c1 = 0;
    let circle_y_c1 = 0;
    let curves = vec![
        vec![
            (x_mid, 0, true),
            (circle_x_c1, circle_y_c1, false),
            (0, circle_y_mid, true),
            (circle_x_c2, circle_y_c2, false),
            (x_mid, circle_top, true),
            (circle_x_c3, circle_y_c3, false),
            (GLYPH_WIDTH, circle_y_mid, true),
            (circle_x_c4, circle_y_c4, false),
            (x_mid, 0, true),
        ],
        vec![
            (x_mid, sw, true),
            (
                circle_x_c4 - sw,
                circle_y_c4 + sw,
                false,
            ),
            (GLYPH_WIDTH - sw, circle_y_mid, true),
            (
                circle_x_c3 - sw,
                circle_y_c3 - sw,
                false,
            ),
            (x_mid, circle_top - sw, true),
            (
                circle_x_c2 + sw,
                circle_y_c2 - sw,
                false,
            ),
            (sw, circle_y_mid, true),
            (
                circle_x_c1 + sw,
                circle_y_c1 + sw,
                false,
            ),
            (x_mid, sw, true),
        ],
        vec![
            (0, bar_top - sw, true),
            (0, bar_top, true),
            (GLYPH_WIDTH, bar_top, true),
            (GLYPH_WIDTH, bar_top - sw, true),
            (0, bar_top - sw, true),
        ],
        vec![
            (x_mid, top_circle_bottom, true),
            (top_circle_x_c1, top_circle_y_c1, false),
            (x_mid - top_circle_r, top_circle_y_mid, true),
            (top_circle_x_c2, top_circle_y_c2, false),
            (x_mid, top_circle_top, true),
            (top_circle_x_c3, top_circle_y_c3, false),
            (x_mid + top_circle_r, top_circle_y_mid, true),
            (top_circle_x_c4, top_circle_y_c4, false),
            (x_mid, top_circle_bottom, true),
        ],
    ];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x1112, Glyph::Simple(glyph.clone()));
    let glyph = create_glyph_with_points(curves, Sung::Jong);
    m.insert(0x11c2, Glyph::Simple(glyph));
}
