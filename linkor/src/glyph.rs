use crate::compose::add_components;
use crate::compose::get_first_chosung_component_bbox;
use crate::consts::*;
use crate::structs::*;
use ahash::HashMap;
use write_fonts::tables::cmap::CmapSubtable;
use write_fonts::tables::glyf::Bbox;
use write_fonts::tables::glyf::CompositeGlyph;
use write_fonts::tables::glyf::Contour;
use write_fonts::tables::glyf::Glyph;
use write_fonts::tables::glyf::SimpleGlyph;
use write_fonts::tables::hmtx::LongMetric;

/*pub fn load_glyphs() -> HashMap<u16, Glyph> {
    let filename = "./glyph_points.json5";
    let mut f = std::fs::File::open(filename).unwrap();
    let mut s: String = String::new();
    f.read_to_string(&mut s);
    let j: Value = json5::from_str(&s).unwrap();
    let mut m: HashMap<u16, Glyph> = HashMap::default();
    if let Value::Object(obj) = j {
        for (k, vv) in obj.iter() {
            let codepoint: u16 = k.parse().unwrap();
            let mut contours: Vec<Contour> = Vec::new();
            let mut x_max: i16 = 0;
            let mut y_max: i16 = 0;
            if let Value::Array(vv) = vv {
                let mut curve_points: Vec<CurvePoint> = Vec::new();
                for v in vv.iter() {
                    let mut x: i16 = i16::MAX;
                    let mut y: i16 = i16::MAX;
                    let mut on_curve: bool = true;
                    if let Value::Array(v) = v {
                        x = match v.get(0).unwrap() {
                            Value::Number(n) => n.as_i64().unwrap() as i16,
                            _ => i16::MAX,
                        };
                        y = match v.get(1).unwrap() {
                            Value::Number(n) => n.as_i64().unwrap() as i16,
                            _ => i16::MAX,
                        };
                        on_curve = match v.get(2).unwrap() {
                            Value::Bool(b) => *b,
                            _ => true,
                        };
                        if x > x_max {
                            x_max = x;
                        }
                        if y > y_max {
                            y_max = y;
                        }
                    }
                    if x == i16::MAX || y == i16::MAX {
                        eprintln!("Glyph parsing error: {}: {:?}", k, v);
                        continue;
                    }
                    curve_points.push(CurvePoint { x, y, on_curve });
                }
                contours.push(curve_points.into());
            }
            let instructions = vec![];
            let glyph = SimpleGlyph {
                bbox: Bbox {
                    x_min: 0,
                    y_min: 0,
                    x_max,
                    y_max,
                },
                contours,
                instructions,
            };
            m.insert(codepoint, Glyph::Simple((glyph)));
        }
    }
    println!("loaded glyphs: {:?}", m);
    m
}*/

pub fn create_glyphs() -> HashMap<u16, Glyph> {
    use crate::glyph_units;
    let mut m: HashMap<u16, Glyph> = HashMap::default();
    glyph_units::consonants::gieug::put(&mut m);
    glyph_units::consonants::ssang_gieug::put(&mut m);
    glyph_units::consonants::nieun::put(&mut m);
    glyph_units::consonants::dieud::put(&mut m);
    glyph_units::consonants::ssang_dieud::put(&mut m);
    glyph_units::consonants::lieul::put(&mut m);
    glyph_units::consonants::mieum::put(&mut m);
    glyph_units::consonants::bieub::put(&mut m);
    glyph_units::consonants::ssang_bieub::put(&mut m);
    glyph_units::consonants::sieus::put(&mut m);
    glyph_units::consonants::ssang_sieus::put(&mut m);
    glyph_units::consonants::ieung::put(&mut m);
    glyph_units::consonants::jieuj::put(&mut m);
    glyph_units::consonants::ssang_jieuj::put(&mut m);
    glyph_units::consonants::chieuch::put(&mut m);
    glyph_units::consonants::kieuk::put(&mut m);
    glyph_units::consonants::tieut::put(&mut m);
    glyph_units::consonants::pieup::put(&mut m);
    glyph_units::consonants::hieuh::put(&mut m);
    glyph_units::vowels::a::put(&mut m);
    glyph_units::vowels::ae::put(&mut m);
    glyph_units::vowels::ya::put(&mut m);
    glyph_units::vowels::yae::put(&mut m);
    glyph_units::vowels::eo::put(&mut m);
    glyph_units::vowels::eoe::put(&mut m);
    glyph_units::vowels::yeo::put(&mut m);
    glyph_units::vowels::yeoe::put(&mut m);
    glyph_units::vowels::o::put(&mut m);
    glyph_units::vowels::yo::put(&mut m);
    glyph_units::vowels::u::put(&mut m);
    glyph_units::vowels::yu::put(&mut m);
    glyph_units::vowels::eui::put(&mut m);
    glyph_units::vowels::i::put(&mut m);
    glyph_units::vowels::are_a::put(&mut m);
    m
}

pub fn add_underbar(contours: &mut Vec<Contour>, x_max: i16) {
    use write_fonts::read::tables::glyf::CurvePoint;
    let args = &*ARGS.read().unwrap();
    let top = -300;
    let bottom = top - args.sw;
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
}

pub fn add_underdot(contours: &mut Vec<Contour>, x_max: i16) {
    use write_fonts::read::tables::glyf::CurvePoint;
    let args = &*ARGS.read().unwrap();
    let x_mid = x_max / 2;
    let top_circle_top = -300;
    let top_circle_bottom = top_circle_top - args.sw;
    let top_circle_r = std::cmp::min(args.sw / 2, MIN_GAP);
    let top_circle_x_c4 = x_mid + top_circle_r;
    let top_circle_y_c4 = top_circle_bottom;
    let top_circle_x_c3 = x_mid + top_circle_r;
    let top_circle_y_c3 = top_circle_top;
    let top_circle_x_c2 = x_mid - top_circle_r;
    let top_circle_y_c2 = top_circle_top;
    let top_circle_x_c1 = x_mid - top_circle_r;
    let top_circle_y_c1 = top_circle_bottom;
    let top_circle_y_mid = top_circle_top - (top_circle_top - top_circle_bottom) / 2;
    let curve = vec![
        CurvePoint{ x: x_mid, y: top_circle_bottom, on_curve: true},
        CurvePoint{ x: top_circle_x_c1, y: top_circle_y_c1, on_curve: false},
        CurvePoint{ x: x_mid - top_circle_r, y: top_circle_y_mid, on_curve: true},
        CurvePoint{ x: top_circle_x_c2, y: top_circle_y_c2, on_curve: false},
        CurvePoint{ x: x_mid, y: top_circle_top, on_curve: true},
        CurvePoint{ x: top_circle_x_c3, y: top_circle_y_c3, on_curve: false},
        CurvePoint{ x: x_mid + top_circle_r, y: top_circle_y_mid, on_curve: true},
        CurvePoint{ x: top_circle_x_c4, y: top_circle_y_c4, on_curve: false},
        CurvePoint{ x: x_mid, y: top_circle_bottom, on_curve: true},
    ];
    contours.push(curve.into());
}

pub fn create_glyph_with_points(curves: Vec<Vec<(i16, i16, bool)>>, sung: Sung) -> SimpleGlyph {
    use write_fonts::read::tables::glyf::CurvePoint;
    let args = &*ARGS.read().unwrap();
    let mut contours: Vec<Contour> = Vec::new();
    let mut x_max: i16 = 0;
    let mut y_max: i16 = 0;
    for curve in curves.iter() {
        let mut curve_points: Vec<CurvePoint> = Vec::with_capacity(curve.len());
        for point in curve.iter() {
            let y = match sung {
                Sung::Jung => (args.jong_h_ratio * point.1 as f32) as i16,
                Sung::Jong => (args.jong_h_ratio * point.1 as f32) as i16,
                _ => point.1,
            };
            if point.0 > x_max {
                x_max = point.0;
            }
            if point.1 > y_max {
                y_max = point.1;
            }
            let point = CurvePoint {
                x: point.0,
                y,
                on_curve: point.2,
            };
            curve_points.push(point);
        }
        contours.push(curve_points.into());
    }
    match sung {
        Sung::Jung => {
            let bar_x_max = x_max + args.jung_gap as i16;
            if args.jung_type == UNDERBAR {
                add_underbar(&mut contours, bar_x_max);
            } else if args.jung_type == UNDERDOT {
                add_underdot(&mut contours, bar_x_max);
            }
        }
        Sung::Jong => {
            let bar_x_max = x_max + args.jong_gap as i16;
            if args.jong_type == UNDERBAR {
                add_underbar(&mut contours, bar_x_max);
            } else if args.jong_type == UNDERDOT {
                add_underdot(&mut contours, bar_x_max);
            }
        }
        _ => {}
    }
    let instructions = vec![];
    SimpleGlyph {
        bbox: Bbox {
            x_min: 0,
            y_min: 0,
            x_max,
            y_max,
        },
        contours,
        instructions,
    }
}

pub fn get_glyph_id_of_codepoint(codepoint: u16, codepoint_to_glyph_id: &HashMap<u16, u16>) -> u16 {
    match codepoint_to_glyph_id.get(&codepoint) {
        Some(v) => *v,
        None => {
            eprintln!("No glyph ID for {:x}", codepoint);
            std::process::exit(1);
        }
    }
}

pub fn get_glyph_x_y_advance_sidebearing(
    glyph_id: u16,
    font_tables: &FontTables,
) -> (i16, i16, i16, i16, u16, i16) {
    if glyph_id >= font_tables.glyphs.len() as u16 {
        eprintln!("Glyph ID {:x} not glyphs", glyph_id);
        std::process::exit(1);
    }
    let glyph = &font_tables.glyphs[glyph_id as usize];
    let h_metric = &font_tables.hmtx.h_metrics
        [std::cmp::min(glyph_id as usize, font_tables.hmtx.h_metrics.len() - 1)];
    match glyph {
        Glyph::Empty => (0, 0, 0, 0, 0, 0),
        Glyph::Simple(g) => (
            g.bbox.x_min,
            g.bbox.x_max,
            g.bbox.y_min,
            g.bbox.y_max,
            h_metric.advance,
            h_metric.side_bearing,
        ),
        Glyph::Composite(g) => (
            g.bbox.x_min,
            g.bbox.x_max,
            g.bbox.y_min,
            g.bbox.y_max,
            h_metric.advance,
            h_metric.side_bearing,
        ),
    }
}

pub fn make_glyph(
    font_tables: &mut FontTables,
    target_codepoint: u16,
    chosung_codepoints: &[u16],
    jungsung_codepoints: &[u16],
    jongsung_codepoints: &[u16],
    do_not_add_char_gap: bool,
) {
    let args = &*ARGS.read().unwrap();
    let mut y_min: i16 = 0;
    let mut y_max: i16 = 0;
    let codepoint = chosung_codepoints[0];
    let (component, bbox, mut advance, side_bearing) =
        get_first_chosung_component_bbox(codepoint, font_tables);
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
            &Sung::Cho,
        );
    }
    if jungsung_codepoints.len() > 0 {
        add_components(
            &mut glyph,
            font_tables,
            &jungsung_codepoints,
            &mut x_max,
            &mut y_min,
            &mut y_max,
            &mut advance,
            &Sung::Jung,
        );
        if jongsung_codepoints.len() > 0 {
            add_components(
                &mut glyph,
                font_tables,
                &jongsung_codepoints,
                &mut x_max,
                &mut y_min,
                &mut y_max,
                &mut advance,
                &Sung::Jong,
            );
        }
        if !do_not_add_char_gap {
            advance += args.char_gap;
        }
    } else {
        if !do_not_add_char_gap {
            advance += args.char_gap;
        }
    }
    glyph.bbox.x_max = x_max;
    glyph.bbox.y_min = y_min;
    glyph.bbox.y_max = y_max;
    let new_glyph_id = font_tables.glyphs.len() as u16;
    font_tables
        .codepoint_to_glyph_id
        .insert(target_codepoint, new_glyph_id);
    for encoding_record in font_tables.cmap.encoding_records.iter_mut() {
        let subtable = encoding_record.subtable.as_mut();
        match subtable {
            CmapSubtable::Format4(cmap4) => {
                let num_ranges = cmap4.end_code.len();
                let last_end_code = cmap4.end_code[num_ranges - 2];
                if target_codepoint == last_end_code + 1 {
                    cmap4.end_code[num_ranges - 2] = target_codepoint;
                } else {
                    for i in 0..(num_ranges - 1) {
                        if cmap4.id_range_offsets[i] > 0 {
                            cmap4.id_range_offsets[i] += 2;
                        }
                    }
                    cmap4.end_code.insert(num_ranges - 1, target_codepoint);
                    cmap4.start_code.insert(num_ranges - 1, target_codepoint);
                    cmap4.id_delta.insert(
                        num_ranges - 1,
                        new_glyph_id as i16 - target_codepoint as i16,
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
    if target_codepoint as u32 == '개' as u32 {
        println!(
            "h_metrics len: {} {:?}, glyph name: {}, glyph len: {}",
            font_tables.hmtx.h_metrics.len(),
            font_tables.hmtx.h_metrics[font_tables.hmtx.h_metrics.len() - 1],
            font_tables.glyph_names[font_tables.glyph_names.len() - 1],
            font_tables.glyphs.len()
        );
    }
}
