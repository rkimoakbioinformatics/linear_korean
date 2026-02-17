use crate::consts::*;
use crate::error::*;
use crate::glyph::*;
use crate::structs::*;
use ahash::HashMap;
use chrono::TimeZone;
use write_fonts::from_obj::FromObjRef;
use write_fonts::read::FontData;
use write_fonts::read::FontRead;
use write_fonts::read::FontRef;
use write_fonts::read::TableProvider;
use write_fonts::tables::cmap::EncodingRecord;
use write_fonts::tables::glyf::Glyph;
use write_fonts::tables::maxp::Maxp;
use write_fonts::tables::name::NameRecord;
use write_fonts::tables::post::Post;
use write_fonts::tables::vmtx::LongMetric;
use write_fonts::types::FWord;
use write_fonts::types::LongDateTime;
use write_fonts::types::NameId;
use write_fonts::types::UfWord;
use write_fonts::types::Version16Dot16;
use write_fonts::validate::Validate;
use write_fonts::FontBuilder;
use write_fonts::OffsetMarker;

pub fn collect_glyphs(
    font: Option<FontRef>,
    font_tables: &mut FontTables,
    created_glyphs: HashMap<u16, Glyph>,
) -> Result<(), Error> {
    let args = &*CONFIG.read().unwrap();
    let glyph_width = args.glyph_width;
    let loca_format = font_tables.head.index_to_loc_format;
    let (glyf_marker, loca_marker) = if let Some(ref font) = font {
        (
            Some(font.glyf().unwrap()),
            Some(font.loca(loca_format == 1).unwrap()),
        )
    } else {
        (None, None)
    };
    for glyph_id in 0..font_tables.maxp.num_glyphs as usize {
        let mut codepoint: u16 = u16::MAX;
        for (c, gid) in font_tables.codepoint_to_glyph_id.iter() {
            if *gid == glyph_id as u16 {
                codepoint = *c;
                break;
            }
        }
        if created_glyphs.contains_key(&codepoint) {
            let g = created_glyphs.get(&codepoint).unwrap();
            if codepoint == 32 && args.space_width.is_some() {
                font_tables.hmtx.h_metrics[glyph_id].advance = args.space_width.unwrap();
                font_tables.glyphs.push(g.clone());
                continue;
            }
            let mut bbox = g.bbox().unwrap();
            if bbox.x_max == 0 {
                bbox.x_max = (args.space_width_ratio * args.glyph_width as f32) as i16;
                bbox.y_max = args.x_height;
            }
            let mut x_max = bbox.x_max as u16;
            if x_max == 0 {
                x_max = glyph_width as u16;
            }
            font_tables.hmtx.h_metrics[glyph_id].advance = x_max;
            font_tables
                .glyphs
                .push(created_glyphs.get(&codepoint).unwrap().clone());
        } else {
            if glyf_marker.is_some() && loca_marker.is_some() {
                let glyf_marker = glyf_marker.as_ref().unwrap();
                let loca_marker = loca_marker.as_ref().unwrap();
                let read_glyph = loca_marker
                    .get_glyf(
                        write_fonts::types::GlyphId::new(glyph_id as u32),
                        &glyf_marker,
                    )
                    .unwrap();
                if let None = read_glyph {
                    let msg = format!(
                        "Glyph {} was not created and is not in loca and glyf.",
                        glyph_id
                    );
                    return Err(Error::Glyph(GlyphError { msg }));
                    /*font_tables
                        .glyphs
                        .push(write_fonts::tables::glyf::Glyph::Empty);
                    continue;*/
                }
                let read_glyph = read_glyph.unwrap();
                match read_glyph {
                    write_fonts::read::tables::glyf::Glyph::Simple(g) => {
                        let data = g.offset_data();
                        let write_glyph =
                            write_fonts::tables::glyf::SimpleGlyph::read(data).unwrap();
                        font_tables
                            .glyphs
                            .push(write_fonts::tables::glyf::Glyph::Simple(write_glyph));
                    }
                    write_fonts::read::tables::glyf::Glyph::Composite(g) => {
                        let data = g.offset_data();
                        let write_glyph =
                            write_fonts::tables::glyf::CompositeGlyph::read(data).unwrap();
                        font_tables
                            .glyphs
                            .push(write_fonts::tables::glyf::Glyph::Composite(write_glyph));
                    }
                }
            } else {
                let msg = format!("No glyph for codepoint: {:x}", codepoint);
                return Err(Error::Glyph(GlyphError { msg }));
                //std::process::exit(1);
            }
        }
    }
    Ok(())
}

pub fn generate_hangul_composite_glyphs(font_tables: &mut FontTables) -> Result<(), Error> {
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
        )?;
    }
    Ok(())
}

pub fn load_existing_cmap(font_data: FontData, font_tables: &mut FontTables) {
    font_tables.cmap = write_fonts::tables::cmap::Cmap::read(font_data).unwrap();
    for encoding_record in font_tables.cmap.encoding_records.iter() {
        match encoding_record.subtable.as_ref() {
            write_fonts::tables::cmap::CmapSubtable::Format4(cmap4) => {
                let id_range_offsets_len: u16 = cmap4.id_range_offsets.len() as u16;
                for i in 0..cmap4.end_code.len() {
                    let range_start = cmap4.start_code[i];
                    let range_end = cmap4.end_code[i];
                    let range_offset = cmap4.id_range_offsets[i];
                    let delta = cmap4.id_delta[i];
                    if range_start >= u16::MAX || range_end >= u16::MAX - 1 {
                        continue;
                    }
                    for codepoint in range_start..(range_end + 1) {
                        let glyph_id = if range_offset == 0 {
                            (codepoint as i32 + delta as i32) as u16
                        } else {
                            let glyph_index_offset = (range_offset / 2) + codepoint - range_start;
                            let glyph_id_array_offset =
                                ((glyph_index_offset - (id_range_offsets_len - i as u16)) as u32
                                    % 65536) as u16;
                            cmap4.glyph_id_array[glyph_id_array_offset as usize]
                        };
                        font_tables
                            .codepoint_to_glyph_id
                            .insert(codepoint, glyph_id);
                    }
                }
                let mut codepoints: Vec<u16> = font_tables
                    .codepoint_to_glyph_id
                    .keys()
                    .map(|&v| v)
                    .collect();
                codepoints.sort();
            }
            write_fonts::tables::cmap::CmapSubtable::Format0(_cmap0) => {}
            write_fonts::tables::cmap::CmapSubtable::Format2(_cmap2) => {}
            write_fonts::tables::cmap::CmapSubtable::Format6(_cmap6) => {}
            write_fonts::tables::cmap::CmapSubtable::Format8(_cmap8) => {}
            write_fonts::tables::cmap::CmapSubtable::Format10(_cmap10) => {}
            write_fonts::tables::cmap::CmapSubtable::Format12(_cmap12) => {}
            write_fonts::tables::cmap::CmapSubtable::Format13(_cmap13) => {}
            write_fonts::tables::cmap::CmapSubtable::Format14(_cmap14) => {}
        }
    }
}

pub fn load_existing_name(font_data: FontData, font_tables: &mut FontTables) {
    font_tables.name = write_fonts::tables::name::Name::read(font_data).unwrap();
    for record in font_tables.name.name_record.iter_mut() {
        match record.name_id {
            NameId::FAMILY_NAME => record.string = OffsetMarker::new("Linear Korean".to_string()),
            NameId::UNIQUE_ID => record.string = OffsetMarker::new("linearkorean".to_string()),
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
}

pub fn load_existing_hmtx(font_data: FontData, font_tables: &mut FontTables, font: &FontRef) {
    let hmtx_marker = font.hmtx().unwrap();
    font_tables.hmtx = write_fonts::tables::hmtx::Hmtx::from_obj_ref(&hmtx_marker, font_data);
}

pub fn get_initial_font_tables() -> FontTables {
    let head: write_fonts::tables::head::Head = write_fonts::tables::head::Head::default();
    let cmap: write_fonts::tables::cmap::Cmap = write_fonts::tables::cmap::Cmap::default();
    let hhea: write_fonts::tables::hhea::Hhea = write_fonts::tables::hhea::Hhea::default();
    let hmtx: write_fonts::tables::hmtx::Hmtx = write_fonts::tables::hmtx::Hmtx::default();
    let maxp: write_fonts::tables::maxp::Maxp = write_fonts::tables::maxp::Maxp::default();
    let name: write_fonts::tables::name::Name = write_fonts::tables::name::Name::default();
    let post: write_fonts::tables::post::Post = write_fonts::tables::post::Post::default();
    let codepoint_to_glyph_id: HashMap<u16, u16> = HashMap::default();
    let glyphs: Vec<write_fonts::tables::glyf::Glyph> = Vec::new();
    let glyph_names: Vec<String> = Vec::new();
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
    font_tables
}

pub fn load_existing_head(font_data: FontData, font_tables: &mut FontTables) {
    font_tables.head = write_fonts::tables::head::Head::read(font_data).unwrap();
}

pub fn load_existing_hhea(font_data: FontData, font_tables: &mut FontTables) {
    font_tables.hhea = write_fonts::tables::hhea::Hhea::read(font_data).unwrap();
}

pub fn load_existing_maxp(font_data: FontData, font_tables: &mut FontTables) {
    font_tables.maxp = write_fonts::tables::maxp::Maxp::read(font_data).unwrap();
}

pub fn load_exsiting_post(font_data: FontData, font_tables: &mut FontTables) {
    font_tables.post = write_fonts::tables::post::Post::read(font_data).unwrap();
}

pub fn handle_no_source(
    font_tables: &mut FontTables,
    created_glyphs: &HashMap<u16, Glyph>,
    created_codepoints: &Vec<u16>,
) {
    let num_glyphs = created_glyphs.len();
    font_tables.post.num_glyphs = Some(num_glyphs as u16);
    let mut glyph_name_idxs: Vec<u16> = Vec::with_capacity(num_glyphs);
    let mut glyph_names: Vec<String> = Vec::with_capacity(num_glyphs);
    let mut glyph_name_idx: u16 = 0;
    let mut end_code: Vec<u16> = Vec::new();
    let mut start_code: Vec<u16> = Vec::new();
    let mut id_delta: Vec<i16> = Vec::new();
    let mut id_range_offsets: Vec<u16> = Vec::new();
    let glyph_id_array: Vec<u16> = Vec::new();
    for codepoint in created_codepoints.iter() {
        if *codepoint <= 257 {
            glyph_name_idxs.push(*codepoint);
        } else {
            glyph_name_idxs.push(glyph_name_idx + 258);
            glyph_names.push(format!("uni{:x}", codepoint));
            glyph_name_idx += 1;
        }
        if end_code.is_empty() {
            end_code.push(*codepoint);
            start_code.push(*codepoint);
            id_delta.push(glyph_name_idx as i16 - 1 - *codepoint as i16);
            id_range_offsets.push(0);
        } else {
            let idx = end_code.len() - 1;
            if *codepoint == end_code[idx] + 1 {
                end_code[idx] = *codepoint;
            } else {
                end_code.push(*codepoint);
                start_code.push(*codepoint);
                id_delta.push(glyph_name_idx as i16 - 1 - *codepoint as i16);
                id_range_offsets.push(0);
            }
        }
    }
    let glyph_name_strs: Vec<&str> = glyph_names.iter().map(|v| v.as_str()).collect();
    //font_tables.post = write_fonts::tables::post::Post::new_v2(glyph_name_strs);
    font_tables.post = write_fonts::tables::post::Post::new(
        write_fonts::types::Fixed::from_i32(0),
        FWord::new(0),
        FWord::new(0),
        0,
        0,
        0,
        0,
        0,
    );
    font_tables.post.version = Version16Dot16::VERSION_3_0;
    let cmap4 = write_fonts::tables::cmap::Cmap4 {
        language: 0,
        end_code,
        start_code,
        id_delta,
        id_range_offsets,
        glyph_id_array,
    };
    let cmap4 = write_fonts::tables::cmap::CmapSubtable::Format4(cmap4);
    let offset_marker = OffsetMarker::new(cmap4);
    let encoding_record: EncodingRecord = EncodingRecord {
        platform_id: write_fonts::tables::cmap::PlatformId::Unicode,
        encoding_id: 3,
        subtable: offset_marker,
    };
    font_tables.cmap.encoding_records.push(encoding_record);
    font_tables.head.font_revision = write_fonts::types::Fixed::ONE;
    font_tables.head.flags = 3;
    font_tables.head.units_per_em = 2048;
    font_tables.head.lowest_rec_ppem = 7;
    font_tables.hhea.ascender = FWord::new(1565);
    font_tables.hhea.descender = FWord::new(-483);
    font_tables.hhea.line_gap = FWord::new(407);
    font_tables.hhea.min_right_side_bearing = FWord::new(-185);
    font_tables.hhea.caret_slope_rise = 1;
    let mut name_records: Vec<NameRecord> = Vec::new();
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::COPYRIGHT_NOTICE,
        string: OffsetMarker::new("2025 Ryangguk Kim".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::FAMILY_NAME,
        string: OffsetMarker::new("Linear Korean".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::SUBFAMILY_NAME,
        string: OffsetMarker::new("Regular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::UNIQUE_ID,
        string: OffsetMarker::new("linearkorean".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::FULL_NAME,
        string: OffsetMarker::new("Linear Korean Regular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::VERSION_STRING,
        string: OffsetMarker::new("Version 1.000".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::POSTSCRIPT_NAME,
        string: OffsetMarker::new("LinearKoreanRegular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::MANUFACTURER,
        string: OffsetMarker::new("Bukdu Group, LLC".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::DESIGNER,
        string: OffsetMarker::new("Ryangguk Kim".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::DESCRIPTION,
        string: OffsetMarker::new("Linear Korean Regular Font".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::VENDOR_URL,
        string: OffsetMarker::new("https://bukdugroup.com".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::DESIGNER_URL,
        string: OffsetMarker::new("https://bukdugroup.com".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::LICENSE_DESCRIPTION,
        string: OffsetMarker::new("Proprietary".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::LICENSE_URL,
        string: OffsetMarker::new("https://bukdugroup.com".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::TYPOGRAPHIC_SUBFAMILY_NAME,
        string: OffsetMarker::new("Regular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::COMPATIBLE_FULL_NAME,
        string: OffsetMarker::new("Linear Korean Regular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::new(256),
        string: OffsetMarker::new("All Typographic Features".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 1,
        encoding_id: 0,
        language_id: 0,
        name_id: NameId::new(257),
        string: OffsetMarker::new("All Features".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::COPYRIGHT_NOTICE,
        string: OffsetMarker::new("2025 Ryangguk Kim".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::FAMILY_NAME,
        string: OffsetMarker::new("Linear Korean".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::SUBFAMILY_NAME,
        string: OffsetMarker::new("Regular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::UNIQUE_ID,
        string: OffsetMarker::new("linearkorean".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::FULL_NAME,
        string: OffsetMarker::new("Linear Korean Regular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::VERSION_STRING,
        string: OffsetMarker::new("Version 1.000".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::POSTSCRIPT_NAME,
        string: OffsetMarker::new("LinearKoreanRegular".to_string()),
    });
    name_records.push(NameRecord {
        platform_id: 3,
        encoding_id: 1,
        language_id: 0,
        name_id: NameId::MANUFACTURER,
        string: OffsetMarker::new("Bukdu Group, LLC".to_string()),
    });
    font_tables.name.name_record = name_records;
    font_tables.name.lang_tag_record = None;
}

pub fn get_font_tables_and_builder<'a>(
    //font: &'a FontRef<'a>,
    font_bytes: &'a [u8],
    glyph_set: &str,
) -> Result<(FontTables, FontBuilder<'a>), Error> {
    let args = &*CONFIG.read().unwrap();
    let mut builder = FontBuilder::default();
    let mut font_tables = get_initial_font_tables();
    let created_glyphs = crate::glyph::create_glyphs(glyph_set)?;
    let font_o: Option<FontRef>;
    if !font_bytes.is_empty() {
        let font = write_fonts::read::FontRef::new(&font_bytes).unwrap();
        for table in font.table_directory.table_records() {
            let tag = table.tag();
            let font_data = font.table_data(tag).unwrap();
            match tag.as_ref() {
                b"cmap" => load_existing_cmap(font_data, &mut font_tables),
                b"head" => load_existing_head(font_data, &mut font_tables),
                b"hhea" => load_existing_hhea(font_data, &mut font_tables),
                b"hmtx" => load_existing_hmtx(font_data, &mut font_tables, &font),
                b"maxp" => load_existing_maxp(font_data, &mut font_tables),
                b"name" => load_existing_name(font_data, &mut font_tables),
                b"post" => load_exsiting_post(font_data, &mut font_tables),
                b"glyf" => {}
                b"loca" => {}
                _ => {
                    let data = font.table_data(tag).unwrap();
                    builder.add_raw(tag, data);
                }
            }
        }
        font_o = Some(font);
        if font_tables.maxp.num_glyphs as usize > font_tables.hmtx.h_metrics.len() {
            while font_tables.hmtx.h_metrics.len() < font_tables.maxp.num_glyphs as usize {
                font_tables
                    .hmtx
                    .h_metrics
                    .push(font_tables.hmtx.h_metrics[font_tables.hmtx.h_metrics.len() - 1].clone());
            }
        }
        let vv = font_tables.post.glyph_name_index.as_ref().unwrap();
        let mut vvv: Vec<String> = Vec::new();
        for v in vv.iter() {
            let v = if *v <= 257 {
                match v {
                    0 => ".notdef",
                    32 => "space",
                    _ => {
                        let msg = format!("Standard Macintosh set name not defined for {}.", v);
                        return Err(Error::Font(FontError { msg }));
                        //std::process::exit(1);
                    }
                }
                .to_string()
            } else {
                font_tables.post.string_data.as_ref().unwrap()[*v as usize - 258].to_string()
            };
            vvv.push(v);
        }
        font_tables.glyph_names = vvv;
    } else {
        let mut created_codepoints: Vec<u16> = created_glyphs.keys().map(|v| *v).collect();
        created_codepoints.sort();
        handle_no_source(&mut font_tables, &created_glyphs, &created_codepoints);
        font_o = None;
        modify_maxp_with_simple_glyphs(&mut font_tables.maxp, &created_glyphs);
        for (glyph_id, codepoint) in created_codepoints.iter().enumerate() {
            font_tables
                .codepoint_to_glyph_id
                .insert(*codepoint, glyph_id as u16);
            let glyph = created_glyphs.get(codepoint).unwrap();
            if *codepoint == 32 && args.space_width.is_some() {
                font_tables.hmtx.h_metrics.push(LongMetric {
                    advance: args.space_width.unwrap(),
                    side_bearing: 0,
                });
                continue;
            }
            let bbox = glyph.bbox();
            let bbox = bbox.as_ref().unwrap();
            font_tables.hmtx.h_metrics.push(LongMetric {
                advance: bbox.x_max as u16,
                side_bearing: bbox.x_min,
            });
        }
    }
    collect_glyphs(font_o, &mut font_tables, created_glyphs)?;
    Ok((font_tables, builder))
}

pub fn modify_post(font_tables: &mut FontTables) {
    /*
    let glyph_names: Vec<&str> = font_tables.glyph_names.iter().map(|v| v.as_str()).collect();
    let mut post = Post::new_v2(glyph_names);
    post.max_mem_type42 = 0;
    post.max_mem_type1 = 0;
    post.is_fixed_pitch = 0;
    font_tables.post = post;
    */
}

pub fn modify_maxp_with_simple_glyphs(maxp: &mut Maxp, glyphs: &HashMap<u16, Glyph>) {
    let num_glyphs = glyphs.len() as u16;
    maxp.num_glyphs = num_glyphs;
    let mut max_points: u16 = 0;
    let mut max_contours: u16 = 0;
    let max_composite_points: u16 = 0;
    let max_composite_contours: u16 = 0;
    let max_zones: u16 = 1;
    let max_twilight_points: u16 = 2;
    let max_storage: u16 = 30;
    let max_function_defs: u16 = 7;
    let max_instruction_defs: u16 = 0;
    let max_stack_elements: u16 = 1024;
    let max_size_of_instructions: u16 = 0;
    let max_component_elements: u16 = 0;
    let max_component_depth: u16 = 2;
    for (_, glyph) in glyphs.iter() {
        match glyph {
            Glyph::Simple(glyph) => {
                max_points = std::cmp::max(
                    max_points,
                    glyph
                        .contours
                        .iter()
                        .map(|v| v.iter().count())
                        .sum::<usize>() as u16,
                );
                max_contours = std::cmp::max(max_contours, glyph.contours.len() as u16);
            }
            _ => {}
        }
    }
    maxp.max_points = Some(max_points);
    maxp.max_contours = Some(max_contours);
    maxp.max_composite_points = Some(max_composite_points);
    maxp.max_composite_contours = Some(max_composite_contours);
    maxp.max_zones = Some(max_zones);
    maxp.max_twilight_points = Some(max_twilight_points);
    maxp.max_storage = Some(max_storage);
    maxp.max_function_defs = Some(max_function_defs);
    maxp.max_instruction_defs = Some(max_instruction_defs);
    maxp.max_stack_elements = Some(max_stack_elements);
    maxp.max_size_of_instructions = Some(max_size_of_instructions);
    maxp.max_component_elements = Some(max_component_elements);
    maxp.max_component_depth = Some(max_component_depth);
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

pub fn modify_name(font_tables: &mut FontTables) -> Result<(), Error> {
    /*let name = &mut font_tables.name;
    for item in name.name_record.iter_mut() {
        item.platform_id =
        item.language_id = 23;
    }*/
    Ok(())
}

pub fn modify_head_hhea(font_tables: &mut FontTables) -> Result<(), Error> {
    let mut x_min: i16 = i16::MAX;
    let mut x_max: i16 = 0;
    let mut y_min: i16 = i16::MAX;
    let mut y_max: i16 = 0;
    let mut max_advance: u16 = 0;
    let mut x_max_extent: i16 = 0;
    for glyph_id in 0..font_tables.maxp.num_glyphs as usize {
        let (
            source_x_min,
            source_x_max,
            source_y_min,
            source_y_max,
            source_advance,
            source_side_bearing,
        ) = get_glyph_x_y_advance_sidebearing(glyph_id as u16, &font_tables)?;
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
    font_tables.head.x_min = x_min;
    font_tables.head.x_max = x_max;
    font_tables.head.y_min = y_min;
    font_tables.head.y_max = y_max;
    font_tables.head.index_to_loc_format = 1;
    let epoch = chrono::Utc.with_ymd_and_hms(1904, 1, 1, 0, 0, 0).unwrap();
    let now = chrono::Utc::now();
    let diff = (now - epoch).num_seconds();
    font_tables.head.created = LongDateTime::new(diff);
    font_tables.head.modified = LongDateTime::new(diff);
    font_tables.hhea.advance_width_max = UfWord::new(max_advance);
    font_tables.hhea.x_max_extent = FWord::new(x_max_extent);
    font_tables.hhea.number_of_h_metrics = font_tables.hmtx.h_metrics.len() as u16;
    Ok(())
}

pub fn build_font_data(font_tables: &FontTables, mut builder: FontBuilder) -> Vec<u8> {
    let mut gl_builder = write_fonts::tables::glyf::GlyfLocaBuilder::new();
    for glyph in font_tables.glyphs.iter() {
        gl_builder.add_glyph(glyph).unwrap();
    }
    let (glyf, loca, _loca_format) = gl_builder.build();
    glyf.validate().unwrap();
    font_tables.head.validate().unwrap();
    font_tables.cmap.validate().unwrap();
    font_tables.hhea.validate().unwrap();
    font_tables.hmtx.validate().unwrap();
    loca.validate().unwrap();
    font_tables.maxp.validate().unwrap();
    font_tables.name.validate().unwrap();
    font_tables.post.validate().unwrap();
    builder.add_table(&glyf).unwrap();
    builder.add_table(&font_tables.head).unwrap();
    builder.add_table(&font_tables.cmap).unwrap();
    builder.add_table(&font_tables.hhea).unwrap();
    builder.add_table(&font_tables.hmtx).unwrap();
    builder.add_table(&loca).unwrap();
    builder.add_table(&font_tables.maxp).unwrap();
    builder.add_table(&font_tables.name).unwrap();
    builder.add_table(&font_tables.post).unwrap();
    builder.build()
}
