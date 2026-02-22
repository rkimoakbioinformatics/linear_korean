use std::io::Read;

use crate::collision::CollisionChecker;
use crate::compose::add_components;
use crate::compose::get_first_chosung_component_bbox;
use crate::consts::*;
use crate::error::*;
use crate::file::get_glyph_set_dir;
use crate::structs::*;
use ahash::HashMap;
use mlua::Lua;
use write_fonts::tables::cmap::CmapSubtable;
use write_fonts::tables::glyf::Bbox;
use write_fonts::tables::glyf::CompositeGlyph;
use write_fonts::tables::glyf::Contour;
use write_fonts::tables::glyf::Glyph;
use write_fonts::tables::glyf::SimpleGlyph;
use write_fonts::tables::hmtx::LongMetric;

fn overflow_font_error(context: &str) -> Error {
    Error::Font(FontError {
        msg: format!("Numeric overflow while generating glyphs ({})", context),
    })
}

fn lua_error(context: &str, err: impl std::fmt::Debug) -> Error {
    Error::Glyph(GlyphError {
        msg: format!("Lua error while {}: {:?}", context, err),
    })
}

fn set_lua_i16(lua: &Lua, key: &str, value: i16) -> Result<(), Error> {
    lua.globals()
        .set(key, value)
        .map_err(|e| lua_error(&format!("setting {}", key), e))
}

fn checked_i16_add(a: i16, b: i16, context: &str) -> Result<i16, Error> {
    a.checked_add(b).ok_or_else(|| overflow_font_error(context))
}

fn checked_i16_sub(a: i16, b: i16, context: &str) -> Result<i16, Error> {
    a.checked_sub(b).ok_or_else(|| overflow_font_error(context))
}

fn checked_i16_mul(a: i16, b: i16, context: &str) -> Result<i16, Error> {
    a.checked_mul(b).ok_or_else(|| overflow_font_error(context))
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

fn scaled_delta_with_baseline(
    top: i16,
    baseline: i16,
    ratio: f32,
    context: &str,
) -> Result<i16, Error> {
    let delta = i32::from(top) - i32::from(baseline);
    let value = delta as f32 * ratio + baseline as f32;
    checked_f32_to_i16(value, context)
}

fn checked_u16_add(a: u16, b: u16, context: &str) -> Result<u16, Error> {
    a.checked_add(b).ok_or_else(|| overflow_font_error(context))
}

fn checked_u16_to_i16(value: u16, context: &str) -> Result<i16, Error> {
    i16::try_from(value).map_err(|_| overflow_font_error(context))
}

pub fn get_glyph_def(glyph_set: &str, glyph_name: &str) -> Result<String, Error> {
    let mut p = get_glyph_set_dir(glyph_set);
    p.push(format!("{}.lua", glyph_name));
    let mut f = match std::fs::File::open(&p) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!(
                "Error loading glyph definition '{}.lua' from set '{}': {:?}",
                glyph_name, glyph_set, e
            );
            return Err(Error::Glyph(GlyphError { msg }));
        }
    };
    let mut s: String = String::new();
    if let Err(e) = f.read_to_string(&mut s) {
        let msg = format!(
            "Error reading glyph definition '{}.lua' from set '{}': {:?}",
            glyph_name, glyph_set, e
        );
        return Err(Error::Glyph(GlyphError { msg }));
    }
    Ok(s)
}

pub fn create_simple_glyph(
    glyph_set: &str,
    glyph_name: &str,
    sung: Sung,
    lua: &Lua,
) -> Result<SimpleGlyph, Error> {
    let curves: Vec<Vec<(i16, i16, bool)>> = get_glyph_curves(glyph_set, glyph_name, lua, &sung)?;
    let glyph = create_glyph_with_points(curves, &sung)?;
    Ok(glyph)
}

pub fn get_glyph_curves(
    glyph_set: &str,
    glyph_name: &str,
    lua: &Lua,
    sung: &Sung,
) -> Result<Vec<Vec<(i16, i16, bool)>>, Error> {
    let args = &*CONFIG
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    set_lua_i16(lua, "X_SW", args.x_sw)?;
    set_lua_i16(lua, "Y_SW", args.y_sw)?;
    match sung {
        Sung::Cho => {
            set_lua_i16(lua, "GLYPH_WIDTH", args.glyph_width)?;
            if args.cho_h_ratio < 0.0 {
                let baseline = scaled_delta_with_baseline(
                    args.cap_height,
                    args.baseline,
                    args.cho_h_ratio * -1.0,
                    "cho baseline",
                )?;
                set_lua_i16(lua, "BASELINE", baseline)?;
                set_lua_i16(lua, "X_HEIGHT", args.x_height)?;
                set_lua_i16(lua, "CAP_HEIGHT", args.cap_height)?;
            } else {
                // A ratio of 0 means "full height" for Cho glyphs.
                let cho_h_ratio = if args.cho_h_ratio == 0.0 {
                    1.0
                } else {
                    args.cho_h_ratio
                };
                set_lua_i16(lua, "BASELINE", args.baseline)?;
                set_lua_i16(
                    lua,
                    "X_HEIGHT",
                    scaled_delta_with_baseline(
                        args.x_height,
                        args.baseline,
                        cho_h_ratio,
                        "cho x_height",
                    )?,
                )?;
                set_lua_i16(
                    lua,
                    "CAP_HEIGHT",
                    scaled_delta_with_baseline(
                        args.cap_height,
                        args.baseline,
                        cho_h_ratio,
                        "cho cap_height",
                    )?,
                )?;
            }
        }
        Sung::Jung => {
            set_lua_i16(
                lua,
                "GLYPH_WIDTH",
                checked_f32_to_i16(
                    args.glyph_width as f32 * args.jung_w_ratio,
                    "jung glyph width",
                )?,
            )?;
            if args.jung_h_ratio < 0.0 {
                let baseline = scaled_delta_with_baseline(
                    args.cap_height,
                    args.baseline,
                    args.jung_h_ratio * -1.0,
                    "jung baseline",
                )?;
                set_lua_i16(lua, "BASELINE", baseline)?;
                set_lua_i16(lua, "X_HEIGHT", args.x_height)?;
                set_lua_i16(lua, "CAP_HEIGHT", args.cap_height)?;
            } else {
                set_lua_i16(lua, "BASELINE", args.baseline)?;
                set_lua_i16(
                    lua,
                    "X_HEIGHT",
                    scaled_delta_with_baseline(
                        args.x_height,
                        args.baseline,
                        args.jung_h_ratio,
                        "jung x_height",
                    )?,
                )?;
                set_lua_i16(
                    lua,
                    "CAP_HEIGHT",
                    scaled_delta_with_baseline(
                        args.cap_height,
                        args.baseline,
                        args.jung_h_ratio,
                        "jung cap_height",
                    )?,
                )?;
            }
        }
        Sung::Jong => {
            set_lua_i16(
                lua,
                "GLYPH_WIDTH",
                checked_f32_to_i16(
                    args.glyph_width as f32 * args.jong_w_ratio,
                    "jong glyph width",
                )?,
            )?;
            if args.jong_h_ratio < 0.0 {
                let baseline = scaled_delta_with_baseline(
                    args.cap_height,
                    args.baseline,
                    args.jong_h_ratio * -1.0,
                    "jong baseline",
                )?;
                set_lua_i16(lua, "BASELINE", baseline)?;
                set_lua_i16(lua, "X_HEIGHT", args.x_height)?;
                set_lua_i16(lua, "CAP_HEIGHT", args.cap_height)?;
            } else {
                set_lua_i16(lua, "BASELINE", args.baseline)?;
                set_lua_i16(
                    lua,
                    "X_HEIGHT",
                    scaled_delta_with_baseline(
                        args.x_height,
                        args.baseline,
                        args.jong_h_ratio,
                        "jong x_height",
                    )?,
                )?;
                set_lua_i16(
                    lua,
                    "CAP_HEIGHT",
                    scaled_delta_with_baseline(
                        args.cap_height,
                        args.baseline,
                        args.jong_h_ratio,
                        "jong cap_height",
                    )?,
                )?;
            }
        }
    }
    let s = get_glyph_def(glyph_set, glyph_name)?;
    let curves: Vec<Vec<Vec<i16>>> = match lua.load(s).eval() {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Error parsing for {}\n{:#?}", glyph_name, e);
            return Err(Error::Glyph(GlyphError { msg }));
            //Vec::new()
        }
    };
    let curves: Vec<Vec<(i16, i16, bool)>> = if curves.len() > 0 {
        let mut vv: Vec<Vec<(i16, i16, bool)>> = Vec::new();
        for curve in curves.iter() {
            let mut v: Vec<(i16, i16, bool)> = Vec::new();
            for point in curve.iter() {
                if point.len() != 3 {
                    let msg = format!(
                        "Invalid point {:?} for {}\n{:#?}",
                        point, glyph_name, curves
                    );
                    return Err(Error::Glyph(GlyphError { msg }));
                    //break;
                } else {
                    v.push((point[0], point[1], point[2] == 1));
                }
            }
            vv.push(v);
        }
        vv
    } else {
        Vec::new()
    };
    Ok(curves)
}

pub fn create_glyphs(glyph_set: &str) -> Result<HashMap<u16, Glyph>, Error> {
    use mlua::prelude::*;
    let lua = Lua::new();
    let args = &*CONFIG
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    set_lua_i16(&lua, "BASELINE", args.baseline)?;
    set_lua_i16(&lua, "GLYPH_WIDTH", args.glyph_width)?;
    set_lua_i16(&lua, "sw", args.sw)?;
    set_lua_i16(&lua, "X_HEIGHT", args.x_height)?;
    set_lua_i16(&lua, "CAP_HEIGHT", args.cap_height)?;
    set_lua_i16(&lua, "MIN_GAP", args.min_gap)?;
    let mut m: HashMap<u16, Glyph> = HashMap::default();
    m.insert(32, Glyph::Empty);
    let glyph_names = [
        ("bieub", 0x1107, 0x11b8),
        ("chieuch", 0x110e, 0x11be),
        ("dieud", 0x1103, 0x11ae),
        ("gieug", 0x1100, 0x11a8),
        ("hieuh", 0x1112, 0x11c2),
        ("ieung", 0x110b, 0x11bc),
        ("jieuj", 0x110c, 0x11bd),
        ("kieuk", 0x110f, 0x11bf),
        ("lieul", 0x1105, 0x11af),
        ("mieum", 0x1106, 0x11b7),
        ("nieun", 0x1102, 0x11ab),
        ("pieup", 0x1111, 0x11c1),
        ("sieus", 0x1109, 0x11ba),
        ("ssang_bieub", 0x1108, 0xffff),
        ("ssang_dieud", 0x1104, 0xffff),
        ("ssang_gieug", 0x1101, 0x11a9),
        ("ssang_jieuj", 0x110d, 0xffff),
        ("ssang_sieus", 0x110a, 0x11bb),
        ("tieut", 0x1110, 0x11c0),
        ("yesieung", 0x3181, 0x3181),
    ];
    for (glyph_name, cho_codepoint, jong_codepoint) in glyph_names.iter() {
        let glyph = create_simple_glyph(glyph_set, glyph_name, Sung::Cho, &lua)?;
        m.insert(*cho_codepoint, Glyph::Simple(glyph.clone()));
        if *jong_codepoint != 0xffff {
            let glyph = create_simple_glyph(glyph_set, glyph_name, Sung::Jong, &lua)?;
            m.insert(*jong_codepoint, Glyph::Simple(glyph));
        }
    }
    let glyph_names = [
        ("a", 0x1161),
        ("ae", 0x1162),
        ("are_a", 0x119e),
        ("eo", 0x1165),
        ("eoe", 0x1166),
        ("eu", 0x1173),
        ("i", 0x1175),
        ("o", 0x1169),
        ("u", 0x116e),
        ("ya", 0x1163),
        ("yae", 0x1164),
        ("yeo", 0x1167),
        ("yeoe", 0x1168),
        ("yo", 0x116d),
        ("yu", 0x1172),
    ];
    for (glyph_name, jung_codepoint) in glyph_names.iter() {
        let glyph = create_simple_glyph(glyph_set, glyph_name, Sung::Jung, &lua)?;
        m.insert(*jung_codepoint, Glyph::Simple(glyph.clone()));
    }
    Ok(m)
}

pub fn add_underbar(contours: &mut Vec<Contour>, x_max: i16) -> Result<(), Error> {
    use write_fonts::read::tables::glyf::CurvePoint;
    let args = &*CONFIG
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let top = args.underdot_y;
    let bottom = checked_i16_sub(top, args.sw, "underbar bottom")?;
    let curve = vec![
        CurvePoint {
            x: 0,
            y: top,
            on_curve: true,
        },
        CurvePoint {
            x: x_max,
            y: top,
            on_curve: true,
        },
        CurvePoint {
            x: x_max,
            y: bottom,
            on_curve: true,
        },
        CurvePoint {
            x: 0,
            y: bottom,
            on_curve: true,
        },
    ];
    contours.push(curve.into());
    Ok(())
}

pub fn add_underdot(contours: &mut Vec<Contour>, x_max: i16) -> Result<(), Error> {
    use write_fonts::read::tables::glyf::CurvePoint;
    let args = &*CONFIG
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let x_mid = x_max / 2;
    let top_circle_r =
        checked_f32_to_i16(args.y_sw as f32 * args.underdot_r_ratio, "underdot radius")?;
    let top_circle_top = args.underdot_y;
    let top_circle_bottom = checked_i16_sub(
        top_circle_top,
        checked_i16_mul(top_circle_r, 2, "underdot radius * 2")?,
        "underdot bottom",
    )?;
    let top_circle_x_c4 = checked_i16_add(x_mid, top_circle_r, "underdot c4 x")?;
    let top_circle_y_c4 = top_circle_bottom;
    let top_circle_x_c3 = checked_i16_add(x_mid, top_circle_r, "underdot c3 x")?;
    let top_circle_y_c3 = top_circle_top;
    let top_circle_x_c2 = checked_i16_sub(x_mid, top_circle_r, "underdot c2 x")?;
    let top_circle_y_c2 = top_circle_top;
    let top_circle_x_c1 = checked_i16_sub(x_mid, top_circle_r, "underdot c1 x")?;
    let top_circle_y_c1 = top_circle_bottom;
    let top_circle_y_mid = checked_f32_to_i16(
        (top_circle_top as f32 + top_circle_bottom as f32) / 2.0,
        "underdot y mid",
    )?;
    let curve = vec![
        CurvePoint {
            x: x_mid,
            y: top_circle_bottom,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c1,
            y: top_circle_y_c1,
            on_curve: false,
        },
        CurvePoint {
            x: checked_i16_sub(x_mid, top_circle_r, "underdot left x")?,
            y: top_circle_y_mid,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c2,
            y: top_circle_y_c2,
            on_curve: false,
        },
        CurvePoint {
            x: x_mid,
            y: top_circle_top,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c3,
            y: top_circle_y_c3,
            on_curve: false,
        },
        CurvePoint {
            x: checked_i16_add(x_mid, top_circle_r, "underdot right x")?,
            y: top_circle_y_mid,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c4,
            y: top_circle_y_c4,
            on_curve: false,
        },
        CurvePoint {
            x: x_mid,
            y: top_circle_bottom,
            on_curve: true,
        },
    ];
    contours.push(curve.into());
    Ok(())
}

pub fn add_upperdot(contours: &mut Vec<Contour>, x_max: i16) -> Result<(), Error> {
    use write_fonts::read::tables::glyf::CurvePoint;
    let args = &*CONFIG
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let x_mid = x_max / 2;
    let top_circle_r =
        checked_f32_to_i16(args.sw as f32 * args.upperdot_r_ratio, "upperdot radius")?;
    let top_circle_top = args.upperdot_y;
    let top_circle_bottom = checked_i16_sub(
        top_circle_top,
        checked_i16_mul(top_circle_r, 2, "upperdot radius * 2")?,
        "upperdot bottom",
    )?;
    let top_circle_x_c4 = checked_i16_add(x_mid, top_circle_r, "upperdot c4 x")?;
    let top_circle_y_c4 = top_circle_bottom;
    let top_circle_x_c3 = checked_i16_add(x_mid, top_circle_r, "upperdot c3 x")?;
    let top_circle_y_c3 = top_circle_top;
    let top_circle_x_c2 = checked_i16_sub(x_mid, top_circle_r, "upperdot c2 x")?;
    let top_circle_y_c2 = top_circle_top;
    let top_circle_x_c1 = checked_i16_sub(x_mid, top_circle_r, "upperdot c1 x")?;
    let top_circle_y_c1 = top_circle_bottom;
    let top_circle_y_mid = checked_f32_to_i16(
        (top_circle_top as f32 + top_circle_bottom as f32) / 2.0,
        "upperdot y mid",
    )?;
    let curve = vec![
        CurvePoint {
            x: x_mid,
            y: top_circle_bottom,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c1,
            y: top_circle_y_c1,
            on_curve: false,
        },
        CurvePoint {
            x: checked_i16_sub(x_mid, top_circle_r, "upperdot left x")?,
            y: top_circle_y_mid,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c2,
            y: top_circle_y_c2,
            on_curve: false,
        },
        CurvePoint {
            x: x_mid,
            y: top_circle_top,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c3,
            y: top_circle_y_c3,
            on_curve: false,
        },
        CurvePoint {
            x: checked_i16_add(x_mid, top_circle_r, "upperdot right x")?,
            y: top_circle_y_mid,
            on_curve: true,
        },
        CurvePoint {
            x: top_circle_x_c4,
            y: top_circle_y_c4,
            on_curve: false,
        },
        CurvePoint {
            x: x_mid,
            y: top_circle_bottom,
            on_curve: true,
        },
    ];
    contours.push(curve.into());
    Ok(())
}

pub fn create_glyph_with_points(
    curves: Vec<Vec<(i16, i16, bool)>>,
    sung: &Sung,
) -> Result<SimpleGlyph, Error> {
    use write_fonts::read::tables::glyf::CurvePoint;
    let args = &*CONFIG
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut contours: Vec<Contour> = Vec::new();
    let mut x_max: i16 = 0;
    let mut y_max: i16 = 0;
    for curve in curves.iter() {
        let mut curve_points: Vec<CurvePoint> = Vec::with_capacity(curve.len());
        for point in curve.iter() {
            if point.0 > x_max {
                x_max = point.0;
            }
            if point.1 > y_max {
                y_max = point.1;
            }
            let point = CurvePoint {
                x: point.0,
                y: point.1,
                on_curve: point.2,
            };
            curve_points.push(point);
        }
        contours.push(curve_points.into());
    }
    match sung {
        Sung::Cho => {
            if args.cho_type & UNDERBAR != 0 {
                let cho_gap_i16 = checked_u16_to_i16(args.cho_cho_gap, "cho_cho gap u16->i16")?;
                add_underbar(
                    &mut contours,
                    checked_i16_add(x_max, cho_gap_i16, "cho underbar x_max")?,
                )?;
            }
            if args.cho_type & UNDERDOT != 0 {
                add_underdot(&mut contours, x_max)?;
            }
            if args.cho_type & UPPERDOT != 0 {
                add_upperdot(&mut contours, x_max)?;
            }
        }
        Sung::Jung => {
            if args.jung_type & UNDERBAR != 0 {
                let jung_gap_i16 =
                    checked_u16_to_i16(args.jung_jung_gap, "jung_jung gap u16->i16")?;
                add_underbar(
                    &mut contours,
                    checked_i16_add(x_max, jung_gap_i16, "jung underbar x_max")?,
                )?;
            }
            if args.jung_type & UNDERDOT != 0 {
                add_underdot(&mut contours, x_max)?;
            }
            if args.jung_type & UPPERDOT != 0 {
                add_upperdot(&mut contours, x_max)?;
            }
        }
        Sung::Jong => {
            if args.jong_type & UNDERBAR != 0 {
                let jong_gap_i16 =
                    checked_u16_to_i16(args.jong_jong_gap, "jong_jong gap u16->i16")?;
                add_underbar(
                    &mut contours,
                    checked_i16_add(x_max, jong_gap_i16, "jong underbar x_max")?,
                )?;
            }
            if args.jong_type & UNDERDOT != 0 {
                add_underdot(&mut contours, x_max)?;
            }
            if args.jong_type & UPPERDOT != 0 {
                add_upperdot(&mut contours, x_max)?;
            }
        }
    }
    let instructions = vec![];
    Ok(SimpleGlyph {
        bbox: Bbox {
            x_min: 0,
            y_min: 0,
            x_max,
            y_max,
        },
        contours,
        instructions,
    })
}

pub fn get_glyph_id_of_codepoint(
    codepoint: u16,
    codepoint_to_glyph_id: &HashMap<u16, u16>,
) -> Result<u16, Error> {
    match codepoint_to_glyph_id.get(&codepoint) {
        Some(v) => Ok(*v),
        None => {
            let msg = format!("No glyph ID for {:x}", codepoint);
            Err(Error::Glyph(GlyphError { msg }))
            //std::process::exit(1);
        }
    }
}

pub fn get_glyph_x_y_advance_sidebearing(
    glyph_id: u16,
    font_tables: &FontTables,
) -> Result<(i16, i16, i16, i16, u16, i16), Error> {
    if glyph_id >= font_tables.glyphs.len() as u16 {
        let msg = format!("Glyph ID {:x} not glyphs", glyph_id);
        return Err(Error::Glyph(GlyphError { msg }));
        //std::process::exit(1);
    }
    let glyph = &font_tables.glyphs[glyph_id as usize];
    let h_metric = &font_tables.hmtx.h_metrics
        [std::cmp::min(glyph_id as usize, font_tables.hmtx.h_metrics.len() - 1)];
    match glyph {
        Glyph::Empty => Ok((0, 0, 0, 0, h_metric.advance, h_metric.side_bearing)),
        Glyph::Simple(g) => Ok((
            g.bbox.x_min,
            g.bbox.x_max,
            g.bbox.y_min,
            g.bbox.y_max,
            h_metric.advance,
            h_metric.side_bearing,
        )),
        Glyph::Composite(g) => Ok((
            g.bbox.x_min,
            g.bbox.x_max,
            g.bbox.y_min,
            g.bbox.y_max,
            h_metric.advance,
            h_metric.side_bearing,
        )),
    }
}

pub fn make_glyph(
    font_tables: &mut FontTables,
    target_codepoint: u16,
    chosung_codepoints: &[u16],
    jungsung_codepoints: &[u16],
    jongsung_codepoints: &[u16],
    do_not_add_char_gap: bool,
    collision_checker: Option<&mut CollisionChecker>,
) -> Result<(), Error> {
    let args = &*CONFIG
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut y_min: i16 = 0;
    let mut y_max: i16 = 0;
    let codepoint = chosung_codepoints[0];
    let (component, bbox, mut advance, side_bearing) =
        get_first_chosung_component_bbox(codepoint, font_tables)?;
    let mut x_max: i16 = bbox.x_max;
    let mut glyph = CompositeGlyph::new(component, bbox);
    if chosung_codepoints.len() > 1 {
        add_components(
            &mut glyph,
            font_tables,
            &chosung_codepoints[1..],
            &mut x_max,
            &mut y_min,
            &mut y_max,
            &mut advance,
            Some(codepoint),
            &args,
            &Sung::Cho,
            &Sung::Cho,
        )?;
    }
    add_components(
        &mut glyph,
        font_tables,
        &jungsung_codepoints,
        &mut x_max,
        &mut y_min,
        &mut y_max,
        &mut advance,
        Some(chosung_codepoints[chosung_codepoints.len() - 1]),
        &args,
        &Sung::Cho,
        &Sung::Jung,
    )?;
    let last_codepoint = if jungsung_codepoints.len() > 0 {
        jungsung_codepoints[jungsung_codepoints.len() - 1]
    } else {
        chosung_codepoints[chosung_codepoints.len() - 1]
    };
    let prev_sung_for_jong = if jungsung_codepoints.is_empty() {
        Sung::Cho
    } else {
        Sung::Jung
    };
    add_components(
        &mut glyph,
        font_tables,
        &jongsung_codepoints,
        &mut x_max,
        &mut y_min,
        &mut y_max,
        &mut advance,
        Some(last_codepoint),
        &args,
        &prev_sung_for_jong,
        &Sung::Jong,
    )?;
    if !do_not_add_char_gap {
        advance = checked_u16_add(advance, args.char_gap, "final char gap")?;
    }
    glyph.bbox.x_max = x_max;
    glyph.bbox.y_min = y_min;
    glyph.bbox.y_max = y_max;
    if let Some(collision_checker) = collision_checker {
        if let Some(raster_debug) =
            collision_checker.composite_collision_debug(&*font_tables, &glyph)?
        {
            let target_char = std::char::from_u32(target_codepoint as u32).unwrap_or('?');
            let msg = format!(
                "Curve collision detected while composing syllable '{}'",
                target_char
            );
            return Err(Error::Collision(CollisionError {
                msg,
                debug: Some(CollisionDebugPayload {
                    character: target_char.to_string(),
                    width: raster_debug.width,
                    height: raster_debug.height,
                    component_a: raster_debug.component_a,
                    component_b: raster_debug.component_b,
                    overlap: raster_debug.overlap,
                }),
            }));
        }
    }
    let new_glyph_id = u16::try_from(font_tables.glyphs.len())
        .map_err(|_| overflow_font_error("glyph count exceeds u16"))?;
    font_tables
        .codepoint_to_glyph_id
        .insert(target_codepoint, new_glyph_id);
    for encoding_record in font_tables.cmap.encoding_records.iter_mut() {
        let subtable = encoding_record.subtable.as_mut();
        match subtable {
            CmapSubtable::Format4(cmap4) => {
                let num_ranges = cmap4.end_code.len();
                let last_end_code = cmap4.end_code[num_ranges - 2];
                let last_end_plus_one = last_end_code.saturating_add(1);
                if target_codepoint == last_end_plus_one {
                    cmap4.end_code[num_ranges - 2] = target_codepoint;
                } else {
                    for i in 0..(num_ranges - 1) {
                        if cmap4.id_range_offsets[i] > 0 {
                            cmap4.id_range_offsets[i] = cmap4.id_range_offsets[i]
                                .checked_add(2)
                                .ok_or_else(|| overflow_font_error("cmap id_range_offsets + 2"))?;
                        }
                    }
                    cmap4.end_code.insert(num_ranges - 1, target_codepoint);
                    cmap4.start_code.insert(num_ranges - 1, target_codepoint);
                    cmap4.id_delta.insert(
                        num_ranges - 1,
                        (new_glyph_id as i16).wrapping_sub(target_codepoint as i16),
                    );
                    cmap4.id_range_offsets.insert(num_ranges - 1, 0);
                }
            }
            _ => {}
        }
    }
    font_tables.glyphs.push(Glyph::Composite(glyph));
    font_tables
        .glyph_names
        .push(format!("uni{:X}", target_codepoint));
    font_tables
        .hmtx
        .h_metrics
        .push(LongMetric::new(advance, side_bearing));
    Ok(())
}
