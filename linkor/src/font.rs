use crate::structs::*;
use ahash::HashMap;
use write_fonts::from_obj::FromObjRef;
use write_fonts::read::FontRead;
use write_fonts::read::FontRef;
use write_fonts::read::TableProvider;
use write_fonts::tables::hmtx::Hmtx;
use write_fonts::types::FWord;
use write_fonts::types::NameId;
use write_fonts::types::UfWord;
use crate::consts::*;
use crate::glyph::*;
use write_fonts::tables::glyf::Glyph;
use write_fonts::tables::maxp::Maxp;
use write_fonts::tables::post::Post;
use write_fonts::validate::Validate;
use write_fonts::FontBuilder;
use write_fonts::OffsetMarker;

pub fn collect_glyphs(
    font: &FontRef,
    glyphs: &mut Vec<Glyph>,
    maxp: &Maxp,
    hmtx: &mut Hmtx,
    loca_format: i16,
    codepoint_to_glyph_id: &HashMap<u16, u16>,
    loaded_glyphs: HashMap<u16, Glyph>,
) {
    let glyf_marker = font.glyf().unwrap();
    let loca_marker = font.loca(loca_format == 1).unwrap();
    for glyph_id in 0..maxp.num_glyphs as usize {
        let mut codepoint: u16 = u16::MAX;
        for (c, gid) in codepoint_to_glyph_id.iter() {
            if *gid == glyph_id as u16 {
                codepoint = *c;
                break;
            }
        }
        if loaded_glyphs.contains_key(&codepoint) {
            let g = loaded_glyphs.get(&codepoint).unwrap();
            let bbox = g.bbox().unwrap();
            hmtx.h_metrics[glyph_id].advance = bbox.x_max as u16;
            glyphs.push(loaded_glyphs.get(&codepoint).unwrap().clone());
        } else {
            let read_glyph = loca_marker
                .get_glyf(
                    write_fonts::types::GlyphId::new(glyph_id as u32),
                    &glyf_marker,
                )
                .unwrap();
            if let None = read_glyph {
                println!("Glyph {} is not in loca and glyf.", glyph_id);
                glyphs.push(write_fonts::tables::glyf::Glyph::Empty);
                continue;
            }
            let read_glyph = read_glyph.unwrap();
            match read_glyph {
                write_fonts::read::tables::glyf::Glyph::Simple(g) => {
                    let data = g.offset_data();
                    let write_glyph = write_fonts::tables::glyf::SimpleGlyph::read(data).unwrap();
                    glyphs.push(write_fonts::tables::glyf::Glyph::Simple(write_glyph));
                }
                write_fonts::read::tables::glyf::Glyph::Composite(g) => {
                    let data = g.offset_data();
                    let write_glyph = write_fonts::tables::glyf::CompositeGlyph::read(data).unwrap();
                    glyphs.push(write_fonts::tables::glyf::Glyph::Composite(write_glyph));
                }
            }
        }
    }
}

pub fn generate_glyphs(font_tables: &mut FontTables) {
    let cho_start: u16 = 0x1100;
    let jung_start: u16 = 0x1161;
    let jong_start: u16 = 0x11a7;
    let code_no_start: u16 = 0xac00;
    let code_no_end: u16 = 0xd7a3;
    for target_codepoint in code_no_start..(code_no_end + 1) {
        let code_offset = target_codepoint - code_no_start;
        let jong_offset = code_offset % 28;
        let jung_offset = (code_offset / 28) % 21;
        let cho_offset = (code_offset / 28) / 21;
        let cho_code_no = cho_start + cho_offset;
        let jung_code_no = jung_start + jung_offset;
        let jong_code_no = jong_start + jong_offset;
        let chosung_codepoints = CHO_CONVERSION_TABLE
            .get(&cho_code_no)
            .map(|v| v.clone())
            .unwrap_or_else(|| vec![cho_code_no]);
        let jungsung_codepoints = JUNG_CONVERSION_TABLE
            .get(&jung_code_no)
            .map(|v| v.clone())
            .unwrap_or_else(|| vec![jung_code_no]);
        let jongsung_codepoints = JONG_CONVERSION_TABLE
            .get(&jong_code_no)
            .map(|v| v.clone())
            .unwrap_or_else(|| vec![jong_code_no]);
        make_glyph(
            font_tables,
            target_codepoint,
            &chosung_codepoints,
            &jungsung_codepoints,
            &jongsung_codepoints,
            false,
        );
    }
}

pub fn get_font_tables_and_builder<'a>(
    font: &'a FontRef<'a>,
) -> (FontTables, FontBuilder<'a>) {
    let loaded_glyphs = crate::glyph::create_glyphs();
    let mut builder = FontBuilder::default();
    let mut head: write_fonts::tables::head::Head = write_fonts::tables::head::Head::default();
    let mut cmap: write_fonts::tables::cmap::Cmap = write_fonts::tables::cmap::Cmap::default();
    let mut hhea: write_fonts::tables::hhea::Hhea = write_fonts::tables::hhea::Hhea::default();
    let mut hmtx: write_fonts::tables::hmtx::Hmtx = write_fonts::tables::hmtx::Hmtx::default();
    let mut maxp: write_fonts::tables::maxp::Maxp = write_fonts::tables::maxp::Maxp::default();
    let mut name: write_fonts::tables::name::Name = write_fonts::tables::name::Name::default();
    let mut post: write_fonts::tables::post::Post = write_fonts::tables::post::Post::default();
    let mut codepoint_to_glyph_id: HashMap<u16, u16> = HashMap::default();
    let mut glyphs: Vec<write_fonts::tables::glyf::Glyph> = Vec::new();
    for table in font.table_directory.table_records() {
        let tag = table.tag();
        let font_data = font.table_data(tag).unwrap();
        match tag.as_ref() {
            b"cmap" => {
                cmap = write_fonts::tables::cmap::Cmap::read(font_data).unwrap();
                for encoding_record in cmap.encoding_records.iter() {
                    match encoding_record.subtable.as_ref() {
                        write_fonts::tables::cmap::CmapSubtable::Format4(cmap4) => {
                            let id_range_offsets_len: u16 = cmap4.id_range_offsets.len() as u16;
                            for i in 0..cmap4.end_code.len() {
                                let range_start = cmap4.start_code[i];
                                let range_end = cmap4.end_code[i];
                                let range_offset = cmap4.id_range_offsets[i];
                                let delta = cmap4.id_delta[i];
                                for codepoint in range_start..(range_end + 1) {
                                    let glyph_id = if range_offset == 0 {
                                        (codepoint as i16 + delta) as u16
                                    } else {
                                        let glyph_index_offset =
                                            (range_offset / 2) + codepoint - range_start;
                                        let glyph_id_array_offset = ((glyph_index_offset
                                            - (id_range_offsets_len - i as u16))
                                            as u32
                                            % 65536)
                                            as u16;
                                        cmap4.glyph_id_array[glyph_id_array_offset as usize]
                                    };
                                    codepoint_to_glyph_id.insert(codepoint, glyph_id);
                                }
                            }
                            let mut codepoints: Vec<u16> =
                                codepoint_to_glyph_id.keys().map(|&v| v).collect();
                            codepoints.sort();
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format0(_cmap0) => {
                            println!("cmap Format 0");
                            /*for (c, glyph_id) in cmap0.glyph_id_array.iter().enumerate() {
                                println!("codepoint {:x} graph ID {}", c, glyph_id);
                            }*/
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format2(_cmap2) => {
                            println!("!!! Format2 Cmap table");
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format6(_cmap6) => {
                            println!("!!! Format6 Cmap table");
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format8(_cmap8) => {
                            println!("!!! Format8 Cmap table");
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format10(_cmap10) => {
                            println!("!!! Format10 Cmap table");
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format12(_cmap12) => {
                            println!("!!! Format12 Cmap table");
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format13(_cmap13) => {
                            println!("!!! Format13 Cmap table");
                        }
                        write_fonts::tables::cmap::CmapSubtable::Format14(_cmap14) => {
                            println!("!!! Format14 Cmap table");
                        }
                    }
                }
            }
            b"head" => {
                head = write_fonts::tables::head::Head::read(font_data).unwrap();
                //println!("head: {:#?}", head);
            }
            b"hhea" => {
                hhea = write_fonts::tables::hhea::Hhea::read(font_data).unwrap();
                //println!("hhea: {:#?}", hhea);
            }
            b"hmtx" => {
                let hmtx_marker = font.hmtx().unwrap();
                hmtx = write_fonts::tables::hmtx::Hmtx::from_obj_ref(&hmtx_marker, font_data);
                //println!("hmtx: {:#?}", hmtx);
                //println!("hmtx {} metrics", hmtx.h_metrics.len());
            }
            b"maxp" => {
                maxp = write_fonts::tables::maxp::Maxp::read(font_data).unwrap();
                //println!("maxp: {:#?}", maxp);
            }
            b"name" => {
                name = write_fonts::tables::name::Name::read(font_data).unwrap();
                for record in name.name_record.iter_mut() {
                    match record.name_id {
                        NameId::FAMILY_NAME => {
                            record.string = OffsetMarker::new("Linear Korean".to_string())
                        }
                        NameId::UNIQUE_ID => {
                            record.string = OffsetMarker::new("linearkorean".to_string())
                        }
                        NameId::FULL_NAME => {
                            record.string = OffsetMarker::new("Linear Korean Regular".to_string())
                        }
                        NameId::POSTSCRIPT_NAME => {
                            record.string = OffsetMarker::new("Linear Korean Regular".to_string())
                        }
                        NameId::TYPOGRAPHIC_FAMILY_NAME => {
                            record.string = OffsetMarker::new("Linear Korean".to_string())
                        }
                        NameId::TYPOGRAPHIC_SUBFAMILY_NAME => {
                            record.string = OffsetMarker::new("Regular".to_string())
                        }
                        _ => {}
                    }
                }
                //println!("name: {:#?}", name);
            }
            b"post" => {
                post = write_fonts::tables::post::Post::read(font_data).unwrap();
                //println!("post: {:#?}", post);
            }
            b"glyf" => {
                //println!("Skipping glyf");
            }
            b"loca" => {
                //println!("Skipping loca");
            }
            _ => {
                println!("=== Adding without processing: {}", tag);
                let data = font.table_data(tag).unwrap();
                builder.add_raw(tag, data);
            }
        }
    }
    println!("{} glyphs. {} h_metrics", glyphs.len(), hmtx.h_metrics.len());
    if maxp.num_glyphs as usize > hmtx.h_metrics.len() {
        while hmtx.h_metrics.len() < maxp.num_glyphs as usize {
            hmtx.h_metrics
                .push(hmtx.h_metrics[hmtx.h_metrics.len() - 1].clone());
        }
    }
    collect_glyphs(
        font,
        &mut glyphs,
        &maxp,
        &mut hmtx,
        head.index_to_loc_format,
        &codepoint_to_glyph_id,
        loaded_glyphs,
    );
    let glyph_names: Vec<String> = post
        .glyph_name_index
        .as_ref()
        .unwrap()
        .iter()
        .map(|&v| {
            if v <= 257 {
                match v {
                    0 => ".notdef",
                    3 => "space",
                    _ => {
                        eprintln!("Standard Macintosh set name not defined for {}.", v);
                        std::process::exit(1);
                    }
                }
                .to_string()
            } else {
                post.string_data.as_ref().unwrap()[v as usize - 258].to_string()
            }
        })
        .collect();
    /*for i in 0..(maxp.num_glyphs as usize) {
        println!(
            "{}: {} advance: {}",
            i, glyph_names[i], hmtx.h_metrics[i].advance
        );
    }*/
    let font_tables = FontTables {
        cmap,
        head,
        hhea,
        hmtx,
        maxp,
        name,
        post,
        codepoint_to_glyph_id,
        glyphs,
        glyph_names,
    };
    (font_tables, builder)
}

pub fn modify_post(font_tables: &mut FontTables) {
    let glyph_names: Vec<&str> = font_tables.glyph_names.iter().map(|v| v.as_str()).collect();
    let post = Post::new_v2(glyph_names);
    font_tables.post = post;
}

pub fn modify_maxp(font_tables: &mut FontTables) {
    let num_glyphs = font_tables.glyphs.len() as u16;
    font_tables.maxp.num_glyphs = num_glyphs;
    let mut max_composite_points: u16 = 0;
    let mut max_composite_contours: u16 = 0;
    let mut max_component_elements: u16 = 0;
    let max_component_depth: u16 = 2;
    for glyph in font_tables.glyphs.iter() {
        match glyph {
            Glyph::Composite(glyph) => {
                let num_components = glyph.components().len() as u16;
                if num_components > max_component_elements {
                    max_component_elements = num_components;
                }
                let mut num_points: u16 = 0;
                let mut num_contours: u16 = 0;
                for component in glyph.components() {
                    let component_glyph = &font_tables.glyphs[component.glyph.to_u16() as usize];
                    match component_glyph {
                        Glyph::Simple(simple_glyph) => {
                            num_contours += simple_glyph.contours.len() as u16;
                            for contour in simple_glyph.contours.iter() {
                                num_points += contour.len() as u16;
                            }
                        }
                        _ => {}
                    }
                }
                if num_contours > max_composite_contours {
                    max_composite_contours = num_contours;
                }
                if num_points > max_composite_points {
                    max_composite_points = num_points;
                }
            }
            _ => {}
        }
    }
    font_tables.maxp.max_composite_points = Some(max_composite_points);
    font_tables.maxp.max_composite_contours = Some(max_composite_contours);
    font_tables.maxp.max_component_elements = Some(max_component_elements);
    font_tables.maxp.max_component_depth = Some(max_component_depth);
}

pub fn modify_head_hhea(font_tables: &mut FontTables) {
    let mut x_min: i16 = i16::MAX;
    let mut x_max: i16 = 0;
    let mut y_min: i16 = i16::MAX;
    let mut y_max: i16 = 0;
    let mut max_advance: u16 = 0;
    let mut x_max_extent: i16 = 0;
    /*println!(
        "Updating head. {} glyphs. {} h_metrics",
        font_tables.maxp.num_glyphs,
        font_tables.hmtx.h_metrics.len()
    );*/
    for glyph_id in 0..font_tables.maxp.num_glyphs as usize {
        let (
            source_x_min,
            source_x_max,
            source_y_min,
            source_y_max,
            source_advance,
            source_side_bearing,
        ) = get_glyph_x_y_advance_sidebearing(glyph_id as u16, &font_tables);
        if source_x_min < x_min {
            x_min = source_x_min;
        }
        if source_x_max > x_max {
            x_max = source_x_max;
        }
        if source_y_min < y_min {
            y_min = source_y_min;
        }
        if source_y_max > y_max {
            y_max = source_y_max;
        }
        if source_advance > max_advance {
            max_advance = source_advance;
        }
        let extent = source_side_bearing + source_x_max - source_x_min;
        if extent > x_max_extent {
            x_max_extent = extent;
        }
    }
    /*println!(
        "head x_min: {}, x_max: {}, y_min: {}, y_max: {}",
        x_min, x_max, y_min, y_max
    );*/
    font_tables.head.x_min = x_min;
    font_tables.head.x_max = x_max;
    font_tables.head.y_min = y_min;
    font_tables.head.y_max = y_max;
    font_tables.head.index_to_loc_format = 1;
    font_tables.hhea.advance_width_max = UfWord::new(max_advance);
    font_tables.hhea.x_max_extent = FWord::new(x_max_extent);
    font_tables.hhea.number_of_long_metrics = font_tables.hmtx.h_metrics.len() as u16;
}

pub fn build_font(font_tables: &FontTables, mut builder: FontBuilder, font_name: &str) {
    println!("Build filename: {}", font_name);
    let mut gl_builder = write_fonts::tables::glyf::GlyfLocaBuilder::new();
    for glyph in font_tables.glyphs.iter() {
        gl_builder.add_glyph(glyph).unwrap();
    }
    let (glyf, loca, _loca_format) = gl_builder.build();
    //println!("loca format: {:?}", loca_format);
    //println!("Validating tables...");
    glyf.validate().unwrap();
    font_tables.head.validate().unwrap();
    font_tables.cmap.validate().unwrap();
    font_tables.hmtx.validate().unwrap();
    loca.validate().unwrap();
    font_tables.maxp.validate().unwrap();
    font_tables.name.validate().unwrap();
    font_tables.post.validate().unwrap();
    //println!("Done validation");
    builder.add_table(&glyf).unwrap();
    builder.add_table(&font_tables.head).unwrap();
    builder.add_table(&font_tables.cmap).unwrap();
    builder.add_table(&font_tables.hhea).unwrap();
    builder.add_table(&font_tables.hmtx).unwrap();
    builder.add_table(&loca).unwrap();
    builder.add_table(&font_tables.maxp).unwrap();
    builder.add_table(&font_tables.name).unwrap();
    builder.add_table(&font_tables.post).unwrap();
    let data = builder.build();
    std::fs::write(font_name, data).unwrap();
}

/*pub fn add_eng_font(font_tables: &mut FontTables) {
    let eng_otf_path = "../PrimerPrint-Regular.ttf";
    let font_bytes = std::fs::read(eng_otf_path).unwrap();
    let eng_font = write_fonts::read::FontRef::new(&font_bytes).unwrap();
    /*for table in eng_font.table_directory.table_records() {
        let tag = table.tag();
        println!("Eng font tag: {:?}", tag);
    }*/
    let glyf = eng_font.glyf().unwrap();
    let loca = eng_font.loca(false).unwrap();
    let hmtx = eng_font.hmtx().unwrap();
    let cmap = eng_font.cmap().unwrap();
    for codepoint in 33..127 {
        let glyph_id = match cmap.map_codepoint(codepoint) {
            Some(v) => v,
            None => {
                println!("Eng glyph for codepoint {} does not exist.", codepoint);
                continue;
            }
        };
        let glyph = match loca.get_glyf(glyph_id, &glyf).unwrap() {
            Some(v) => v,
            None => {
                println!("Eng glyph for {} does not exist.", codepoint);
                continue;
            }
        };
        match glyph {
            write_fonts::read::tables::glyf::Glyph::Simple(source_glyph) => {
                let data = source_glyph.offset_data();
                let glyph = write_fonts::tables::glyf::SimpleGlyph::read(data).unwrap();
                let new_glyph_id = font_tables.glyphs.len() as u16;
                font_tables
                    .codepoint_to_glyph_id
                    .insert(codepoint, new_glyph_id);
                for encoding_record in font_tables.cmap.encoding_records.iter_mut() {
                    let subtable = encoding_record.subtable.as_mut();
                    match subtable {
                        CmapSubtable::Format4(cmap4) => {
                            let num_ranges = cmap4.end_code.len();
                            let last_end_code = cmap4.end_code[1];
                            if codepoint == last_end_code + 1 {
                                cmap4.end_code[1] = codepoint;
                            } else {
                                for i in 0..(num_ranges - 1) {
                                    if cmap4.id_range_offsets[i] > 0 {
                                        cmap4.id_range_offsets[i] += 2;
                                    }
                                }
                                cmap4.end_code.insert(1, codepoint);
                                cmap4.start_code.insert(1, codepoint);
                                cmap4
                                    .id_delta
                                    .insert(1, new_glyph_id as i16 - codepoint as i16);
                                cmap4.id_range_offsets.insert(1, 0);
                            }
                        }
                        _ => {}
                    }
                }
                font_tables
                    .glyph_names
                    .push(format!("uni{:X}", new_glyph_id));
                let h_metric = hmtx.h_metrics()[glyph_id.to_u32() as usize];
                font_tables.hmtx.h_metrics.insert(
                    1 + codepoint as usize - 33,
                    LongMetric::new(h_metric.advance.get(), h_metric.side_bearing.get()),
                );
                font_tables.glyphs.push(Glyph::Simple(glyph));
            }
            write_fonts::read::tables::glyf::Glyph::Composite(_g) => {
                /*println!(
                    "Eng composite glyph {}: {:?}",
                    codepoint,
                    g.number_of_contours()
                );*/
            }
        }
    }
}*/

/*pub fn show_cmap(font_tables: &FontTables) {
    for encoding_record in font_tables.cmap.encoding_records.iter() {
        match encoding_record.subtable.as_ref() {
            CmapSubtable::Format4(cmap4) => {
                println!("=== Format4");
                println!("start_code: {:?}", cmap4.end_code);
                println!("end_code: {:?}", cmap4.start_code);
                println!("id_delta: {:?}", cmap4.id_delta);
                println!("id_range_offsets: {:?}", cmap4.id_range_offsets);
                println!("glyph_id_array: {:?}", cmap4.glyph_id_array);
            }
            _ => {}
        }
    }
}*/
