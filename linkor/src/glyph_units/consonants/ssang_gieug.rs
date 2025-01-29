use crate::consts::*;
use crate::glyph::create_glyph_with_points;
use crate::structs::Sung;
use ahash::HashMap;
use write_fonts::tables::glyf::Glyph;

pub fn put(m: &mut HashMap<u16, Glyph>) {
  let args = &*ARGS.read().unwrap();
  let sw = args.sw;
    let mid_x = GLYPH_WIDTH / 2;
    let mid_left = mid_x - sw / 2;
    let mid_right = mid_x + sw / 2;
    let curves = vec![vec![
        (0, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, GLYPH_HEIGHT, true),
        (GLYPH_WIDTH, 0, true),
        (GLYPH_WIDTH - sw, 0, true),
        (
            GLYPH_WIDTH - sw,
            GLYPH_HEIGHT - sw,
            true,
        ),
        (mid_right, GLYPH_HEIGHT - sw, true),
        (mid_right, 0, true),
        (mid_left, 0, true),
        (mid_left, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT - sw, true),
        (0, GLYPH_HEIGHT, true),
    ]];
    let glyph = create_glyph_with_points(curves.clone(), Sung::Cho);
    m.insert(0x1101, Glyph::Simple(glyph.clone()));
    let glyph = create_glyph_with_points(curves, Sung::Jong);
    m.insert(0x1109, Glyph::Simple(glyph));
}
