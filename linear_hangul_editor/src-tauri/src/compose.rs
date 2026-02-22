use crate::consts::*;
use crate::error::*;
use crate::glyph::get_glyph_id_of_codepoint;
use crate::glyph::get_glyph_x_y_advance_sidebearing;
use crate::glyph::make_glyph;
use crate::structs::*;
use write_fonts::tables::glyf::Bbox;
use write_fonts::tables::glyf::Component;
use write_fonts::tables::glyf::CompositeGlyph;
use write_fonts::types::GlyphId16;

fn overflow_font_error(context: &str) -> Error {
    Error::Font(FontError {
        msg: format!("Numeric overflow while composing glyphs ({})", context),
    })
}

fn checked_u16_add(a: u16, b: u16, context: &str) -> Result<u16, Error> {
    a.checked_add(b).ok_or_else(|| overflow_font_error(context))
}

fn checked_i16_add(a: i16, b: i16, context: &str) -> Result<i16, Error> {
    a.checked_add(b).ok_or_else(|| overflow_font_error(context))
}

fn checked_f32_to_i16(value: f32, context: &str) -> Result<i16, Error> {
    if !value.is_finite() {
        return Err(overflow_font_error(context));
    }
    let rounded = value.round();
    if rounded < i16::MIN as f32 || rounded > i16::MAX as f32 {
        return Err(overflow_font_error(context));
    }
    Ok(rounded as i16)
}

fn checked_u16_to_i16(value: u16, context: &str) -> Result<i16, Error> {
    i16::try_from(value).map_err(|_| overflow_font_error(context))
}

pub fn get_first_chosung_component_bbox(
    codepoint: u16,
    font_tables: &mut FontTables,
) -> Result<(Component, Bbox, u16, i16), Error> {
    let glyph_id = get_glyph_id_of_codepoint(codepoint, &font_tables.codepoint_to_glyph_id)?;
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
    ) = get_glyph_x_y_advance_sidebearing(glyph_id, &font_tables)?;
    let bbox = Bbox {
        x_min: 0,
        y_min: 0,
        x_max: source_x_max,
        y_max: source_y_max,
    };
    Ok((component, bbox, source_advance, source_side_bearing))
}

pub fn get_composite_glyph_component_and_bbox(
    source_codepoint: u16,
    font_tables: &mut FontTables,
    y_min: &mut i16,
    y_max: &mut i16,
    advance: &mut u16,
    kern: f32,
    sung: &Sung,
) -> Result<(Component, Bbox), Error> {
    let source_glyph_id =
        get_glyph_id_of_codepoint(source_codepoint, &font_tables.codepoint_to_glyph_id)?;
    let (
        source_x_min,
        source_x_max,
        _source_y_min,
        source_y_max,
        source_advance,
        _source_side_bearing,
    ) = get_glyph_x_y_advance_sidebearing(source_glyph_id, &font_tables)?;
    let flags = write_fonts::read::tables::glyf::CompositeGlyphFlags::empty();
    let transform = write_fonts::read::tables::glyf::Transform {
        xx: write_fonts::types::F2Dot14::from_f32(1.0),
        yx: write_fonts::types::F2Dot14::from_f32(0.0),
        xy: write_fonts::types::F2Dot14::from_f32(0.0),
        yy: write_fonts::types::F2Dot14::from_f32(1.0),
    };
    let base_advance_i16 = checked_u16_to_i16(*advance, "u16->i16 base advance")?;
    let kern_offset = checked_f32_to_i16(kern * source_x_max as f32, "kerning x offset")?;
    let x = checked_i16_add(base_advance_i16, kern_offset, "component x position")?;
    let y: i16 = match sung {
        Sung::Cho => 0,
        Sung::Jung => 0,
        Sung::Jong => 0,
    };
    let anchor = write_fonts::tables::glyf::Anchor::Offset { x, y };
    let component = write_fonts::tables::glyf::Component::new(
        GlyphId16::new(source_glyph_id),
        anchor,
        transform,
        flags,
    );
    let bbox_y_max = checked_i16_add(source_y_max, y, "component bbox y_max")?;
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
    let x_advance = if x < 0 {
        0
    } else {
        u16::try_from(x).map_err(|_| overflow_font_error("component x advance"))?
    };
    *advance = checked_u16_add(x_advance, source_advance, "advance + source advance")?;
    Ok((component, bbox))
}

pub fn get_kerning(prev: u16, next: u16, args: &Args) -> f32 {
    *args.kerning_data.get(&(prev, next)).unwrap_or(&0.0)
}

fn get_transition_gap(prev_sung: &Sung, next_sung: &Sung, args: &Args) -> u16 {
    match (prev_sung, next_sung) {
        (Sung::Cho, Sung::Cho) => args.cho_cho_gap,
        (Sung::Jung, Sung::Jung) => args.jung_jung_gap,
        (Sung::Jong, Sung::Jong) => args.jong_jong_gap,
        (Sung::Cho, Sung::Jung) => args.cho_jung_gap,
        (Sung::Jung, Sung::Jong) => args.jung_jong_gap,
        // These transitions are not expected in normal syllable composition.
        _ => 0,
    }
}

pub fn add_components(
    glyph: &mut CompositeGlyph,
    font_tables: &mut FontTables,
    source_codepoints: &[u16],
    x_max: &mut i16,
    y_min: &mut i16,
    y_max: &mut i16,
    advance: &mut u16,
    last_codepoint: Option<u16>,
    args: &Args,
    prev_sung: &Sung,
    sung: &Sung,
) -> Result<(), Error> {
    let len_codepoints = source_codepoints.len();
    for i in 0..len_codepoints {
        let mut codepoint = source_codepoints[i];
        if codepoint == 0x11bc && *sung == Sung::Jong {
            codepoint = 0x3181;
        }
        let kern: f32 = if i == 0 && last_codepoint.is_some() {
            let last_codepoint = last_codepoint.unwrap();
            get_kerning(last_codepoint, codepoint, args)
        } else {
            get_kerning(source_codepoints[i - 1], codepoint, args)
        };
        let gap = if i == 0 {
            get_transition_gap(prev_sung, sung, args)
        } else {
            get_transition_gap(sung, sung, args)
        };
        *advance = checked_u16_add(*advance, gap, "component transition gap")?;
        let prev_advance = *advance;
        let (component, bbox) = get_composite_glyph_component_and_bbox(
            codepoint,
            font_tables,
            y_min,
            y_max,
            advance,
            kern,
            sung,
        )?;
        let prev_advance_i16 = checked_u16_to_i16(prev_advance, "u16->i16 x_max advance")?;
        *x_max = checked_i16_add(prev_advance_i16, bbox.x_max, "composite x_max")?;
        glyph.add_component(component, bbox);
    }
    Ok(())
}

pub fn make_compatibility_jamos(font_tables: &mut FontTables) -> Result<(), Error> {
    for data in COMPATIBILITY_JAMOS_TO_MAKE.iter() {
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
            false,
            None,
        )?;
    }
    Ok(())
}
