use crate::consts::*;
use crate::glyph::get_glyph_id_of_codepoint;
use crate::glyph::get_glyph_x_y_advance_sidebearing;
use crate::glyph::make_glyph;
use crate::structs::*;
use write_fonts::types::GlyphId16;
use write_fonts::tables::glyf::Bbox;
use write_fonts::tables::glyf::Component;
use write_fonts::tables::glyf::CompositeGlyph;

pub fn get_first_chosung_component_bbox(
    codepoint: u16,
    font_tables: &mut FontTables,
) -> (Component, Bbox, u16, i16) {
    let args = &*ARGS.read().unwrap();
    let glyph_id = get_glyph_id_of_codepoint(codepoint, &font_tables.codepoint_to_glyph_id);
    let anchor = write_fonts::tables::glyf::Anchor::Offset { x: 0, y: 0 };
    let transform = write_fonts::tables::glyf::Transform {
        xx: write_fonts::types::F2Dot14::from_f32(1.0),
        yx: write_fonts::types::F2Dot14::from_f32(0.0),
        xy: write_fonts::types::F2Dot14::from_f32(0.0),
        yy: write_fonts::types::F2Dot14::from_f32(1.0),
    };
    let flags = write_fonts::tables::glyf::ComponentFlags::default();
    let component = write_fonts::tables::glyf::Component::new(
        GlyphId16::new(glyph_id),
        anchor,
        transform,
        flags,
    );
    let (
        _source_x_min,
        source_x_max,
        _source_y_min,
        source_y_max,
        source_advance,
        source_side_bearing,
    ) = get_glyph_x_y_advance_sidebearing(glyph_id, &font_tables);
    let bbox = Bbox {
        x_min: 0,
        y_min: 0,
        x_max: source_x_max,
        y_max: source_y_max,
    };
    let advance = source_advance + args.cho_gap;
    (component, bbox, advance, source_side_bearing)
}

pub fn get_composite_glyph_component_and_bbox(
    source_codepoint: u16,
    font_tables: &mut FontTables,
    y_min: &mut i16,
    y_max: &mut i16,
    advance: &mut u16,
    sung: &Sung,
) -> (Component, Bbox) {
    let args = &*ARGS.read().unwrap();
    let source_glyph_id =
        get_glyph_id_of_codepoint(source_codepoint, &font_tables.codepoint_to_glyph_id);
    let (
        source_x_min,
        source_x_max,
        _source_y_min,
        source_y_max,
        source_advance,
        _source_side_bearing,
    ) = get_glyph_x_y_advance_sidebearing(source_glyph_id, &font_tables);
    let flags = write_fonts::read::tables::glyf::CompositeGlyphFlags::empty();
    let transform = write_fonts::read::tables::glyf::Transform {
        xx: write_fonts::types::F2Dot14::from_f32(1.0),
        yx: write_fonts::types::F2Dot14::from_f32(0.0),
        xy: write_fonts::types::F2Dot14::from_f32(0.0),
        yy: write_fonts::types::F2Dot14::from_f32(1.0),
    };
    let x: i16 = *advance as i16;
    let y: i16 = match sung {
        Sung::Cho => 0,
        Sung::Jung => -(GLYPH_HEIGHT as f32 * 0.0) as i16,
        Sung::Jong => (GLYPH_HEIGHT as f32 * 0.0) as i16,
    };
    let anchor = write_fonts::tables::glyf::Anchor::Offset { x, y };
    let component = write_fonts::tables::glyf::Component::new(
        GlyphId16::new(source_glyph_id),
        anchor,
        transform,
        flags,
    );
    let bbox_y_max = source_y_max + y;
    let bbox = Bbox {
        x_min: source_x_min,
        y_min: y,
        x_max: source_x_max,
        y_max: bbox_y_max,
    };
    if y < *y_min {
        *y_min = y;
    }
    if source_y_max > *y_max {
        *y_max = bbox_y_max;
    }
    *advance = x as u16 + source_advance;
    match sung {
        Sung::Cho => *advance += args.cho_gap,
        Sung::Jung => *advance += args.jung_gap,
        Sung::Jong => *advance += args.jong_gap,
    }
    (component, bbox)
}

pub fn add_components(
    glyph: &mut CompositeGlyph,
    font_tables: &mut FontTables,
    source_codepoints: &[u16],
    x_max: &mut i16,
    y_min: &mut i16,
    y_max: &mut i16,
    advance: &mut u16,
    sung: &Sung,
) {
    for source_codepoint in source_codepoints.iter() {
        let prev_advance = *advance;
        let (component, bbox) = get_composite_glyph_component_and_bbox(
            *source_codepoint,
            font_tables,
            y_min,
            y_max,
            advance,
            sung,
        );
        *x_max = prev_advance as i16 + bbox.x_max;
        glyph.add_component(component, bbox);
    }
}

pub fn map_composite_chosungs(font_tables: &mut FontTables) {
    for data in COMPOSITE_CHOSUNGS_TO_MAKE.iter() {
        let target_codepoint = data[0][0];
        let chosung_codepoints = &data[1];
        let jungsung_codepoints = &data[2];
        let jongsung_codepoints = &data[3];
        make_glyph(
            font_tables,
            target_codepoint,
            chosung_codepoints,
            jungsung_codepoints,
            jongsung_codepoints,
            true,
        );
    }
}
