mod compose;
mod consts;
mod error;
mod file;
mod font;
mod glyph;
mod structs;

use crate::compose::*;
use crate::consts::*;
use crate::error::*;
use crate::file::*;
use crate::font::*;
use crate::structs::*;
use ahash::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use tauri::AppHandle;

type KerningMap = HashMap<(u16, u16), f32>;

pub fn compile(args: &Args, glyph_set: &str) -> Result<(), Error> {
    *CONFIG.write().unwrap() = args.clone();
    let font_bytes: Vec<u8>;
    if let Some(source_filename) = &args.source_filename {
        font_bytes = std::fs::read(source_filename).unwrap();
    } else {
        font_bytes = Vec::new();
    }
    let (mut font_tables, builder) = get_font_tables_and_builder(&font_bytes, glyph_set)?;
    make_compatibility_jamos(&mut font_tables)?;
    generate_hangul_composite_glyphs(&mut font_tables)?;
    //add_eng_font(&mut font_tables);
    modify_post(&mut font_tables);
    modify_maxp(&mut font_tables);
    modify_head_hhea(&mut font_tables)?;
    modify_name(&mut font_tables)?;
    let font_data = build_font_data(&font_tables, builder);
    let p = get_font_ttf_p(&args.target_fontname);
    let dir = p.parent().unwrap();
    if !dir.exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }
    std::fs::write(&p, font_data).unwrap();
    Ok(())
}

pub fn make_woff2(args: &Args) -> Result<(), Error> {
    let ttf_p = get_font_ttf_p(&args.target_fontname);
    let woff2_p = get_font_woff2_p(&args.target_fontname);
    let ttf_data = match std::fs::read(&ttf_p) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Font(FontError {
                msg: format!("Failed to read {:?} before WOFF2 encoding: {:?}", ttf_p, e),
            }));
        }
    };
    let woff2_data = match ttf2woff2::encode(&ttf_data, ttf2woff2::BrotliQuality::default()) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Font(FontError {
                msg: format!("Failed to convert {:?} to WOFF2: {:?}", ttf_p, e),
            }));
        }
    };
    if let Err(e) = std::fs::write(&woff2_p, &woff2_data) {
        return Err(Error::Font(FontError {
            msg: format!("Failed to write WOFF2 {:?}: {:?}", woff2_p, e),
        }));
    }
    Ok(())
}

pub fn get_args(config: &mut Config, kerning_name: &str) -> Result<Args, Error> {
    let config_str = get_config_str(DEFAULT_NAME);
    if !config_str.is_empty() {
        let in_config: Config = json5::from_str(&config_str).unwrap();
        if config.source.is_none() {
            config.source = in_config.source;
        }
        if config.cho_type.is_empty() {
            config.cho_type = in_config.cho_type;
        }
        if config.jung_type.is_empty() {
            config.jung_type = in_config.jung_type;
        }
        if config.jong_type.is_empty() {
            config.jong_type = in_config.jong_type;
        }
        if config.cho_h_ratio.is_none() {
            config.cho_h_ratio = in_config.cho_h_ratio;
        }
        if config.jung_h_ratio.is_none() {
            config.jung_h_ratio = in_config.jung_h_ratio;
        }
        if config.jong_h_ratio.is_none() {
            config.jong_h_ratio = in_config.jong_h_ratio;
        }
        if config.char_gap.is_none() {
            config.char_gap = in_config.char_gap;
        }
        if config.jung_gap.is_none() {
            config.jung_gap = in_config.jung_gap;
        }
        if config.jong_gap.is_none() {
            config.jong_gap = in_config.jong_gap;
        }
        if config.sw_ratio.is_none() {
            config.sw_ratio = in_config.sw_ratio;
        }
        if config.text_size.is_none() {
            config.text_size = in_config.text_size;
        }
        if config.underdot_y.is_none() {
            config.underdot_y = in_config.underdot_y;
        }
        if config.underdot_r_ratio.is_none() {
            config.underdot_r_ratio = in_config.underdot_r_ratio;
        }
        if config.glyph_width.is_none() {
            config.glyph_width = in_config.glyph_width;
        }
        if config.cap_height.is_none() {
            config.cap_height = in_config.cap_height;
        }
        if config.x_height.is_none() {
            config.x_height = in_config.x_height;
        }
        if config.baseline.is_none() {
            config.baseline = in_config.baseline;
        }
        if config.min_gap.is_none() {
            config.min_gap = in_config.min_gap;
        }
        if config.space_width.is_none() {
            config.space_width = in_config.space_width;
        }
    }
    let source_filename: Option<String> = match &config.source {
        Some(v) => {
            let p = PathBuf::from(&v);
            if !p.exists() {
                let msg = format!("{} does not exist.", v);
                return Err(Error::Config(ConfigError { msg }));
                //std::process::exit(1);
            }
            let source_p: PathBuf = p.canonicalize().unwrap();
            let source_filename = source_p.to_str().unwrap().to_string();
            Some(source_filename)
        }
        None => None,
    };
    let target_fontname = COMPILED_FONT_NAME.to_string();
    let cho_type_v = config.cho_type.clone();
    let mut cho_type: u8 = 0;
    cho_type_v.iter().for_each(|v| {
        if v == "underdot" {
            cho_type += UNDERDOT;
        } else if v == "underbar" {
            cho_type += UNDERBAR;
        }
    });
    let jung_type_v = config.jung_type.clone();
    let mut jung_type: u8 = 0;
    jung_type_v.iter().for_each(|v| {
        if v == "underdot" {
            jung_type += UNDERDOT;
        } else if v == "underbar" {
            jung_type += UNDERBAR;
        }
    });
    let jong_type_v = config.jong_type.clone();
    let mut jong_type: u8 = 0;
    jong_type_v.iter().for_each(|v| {
        if v == "underdot" {
            jong_type += UNDERDOT;
        } else if v == "underbar" {
            jong_type += UNDERBAR;
        }
    });
    if config.cho_h_ratio.is_none() {
        config.cho_h_ratio = Some(0.0);
    }
    if config.jung_w_ratio.is_none() {
        config.jung_w_ratio = Some(1.0);
    }
    if config.jong_w_ratio.is_none() {
        config.jong_w_ratio = Some(1.0);
    }
    if config.jung_h_ratio.is_none() {
        config.jung_h_ratio = Some(1.0);
    }
    if config.jong_h_ratio.is_none() {
        config.jong_h_ratio = Some(1.0);
    }
    if config.char_gap.is_none() {
        config.char_gap = Some(0);
    }
    if config.cho_gap.is_none() {
        config.cho_gap = Some(0);
    }
    if config.jung_gap.is_none() {
        config.jung_gap = Some(0);
    }
    if config.jong_gap.is_none() {
        config.jong_gap = Some(0);
    }
    if config.text_size.is_none() {
        config.text_size = Some(16);
    }
    if config.underdot_y.is_none() {
        config.underdot_y = Some(-300);
    }
    if config.underdot_r_ratio.is_none() {
        config.underdot_r_ratio = Some(0.5);
    }
    if config.glyph_width.is_none() {
        config.glyph_width = Some(800);
    }
    if config.baseline.is_none() {
        config.baseline = Some(0);
    }
    if config.x_height.is_none() {
        config.x_height = Some(1500);
    }
    if config.cap_height.is_none() {
        config.cap_height = Some(1800);
    }
    if config.min_gap.is_none() {
        config.min_gap = Some(200);
    }
    if config.sw_ratio.is_none() {
        config.sw_ratio = Some(0.08);
    }
    if config.space_width_ratio.is_none() {
        config.space_width_ratio = Some(2.0);
    }
    if config_str.is_empty() {
        let s = json5::to_string(&config).unwrap();
        let j: Config = serde_json::from_str(&s).unwrap();
        let s = serde_json::to_string_pretty(&j).unwrap();
        _save_config(&s, DEFAULT_NAME);
    }
    let cho_h_ratio: f32 = config.cho_h_ratio.unwrap();
    let jung_w_ratio: f32 = config.jung_w_ratio.unwrap();
    let jong_w_ratio: f32 = config.jong_w_ratio.unwrap();
    let jung_h_ratio: f32 = config.jung_h_ratio.unwrap();
    let jong_h_ratio: f32 = config.jong_h_ratio.unwrap();
    let char_gap: u16 = config.char_gap.unwrap();
    let cho_gap: u16 = config.cho_gap.unwrap();
    let jung_gap: u16 = config.jung_gap.unwrap();
    let jong_gap: u16 = config.jong_gap.unwrap();
    let text_size: u16 = config.text_size.unwrap();
    let underdot_y: i16 = config.underdot_y.unwrap();
    let underdot_r_ratio: f32 = config.underdot_r_ratio.unwrap();
    let glyph_width: i16 = config.glyph_width.unwrap();
    let baseline: i16 = config.baseline.unwrap();
    let x_height: i16 = config.x_height.unwrap();
    let cap_height: i16 = config.cap_height.unwrap();
    let min_gap: i16 = config.min_gap.unwrap();
    let sw_ratio: f32 = config.sw_ratio.unwrap();
    let sw: i16 = (sw_ratio * glyph_width as f32) as i16;
    let kerning_data: KerningMap = get_kerning_map(kerning_name)?;
    let space_width: Option<u16> = config.space_width;
    let space_width_ratio: f32 = config.space_width_ratio.unwrap();
    Ok(Args {
        source_filename,
        target_fontname,
        cho_type,
        jung_type,
        jong_type,
        jung_w_ratio,
        jong_w_ratio,
        cho_h_ratio,
        jung_h_ratio,
        jong_h_ratio,
        char_gap,
        cho_gap,
        jung_gap,
        jong_gap,
        sw_ratio,
        sw,
        text_size,
        underdot_y,
        underdot_r_ratio,
        glyph_width,
        baseline,
        x_height,
        cap_height,
        min_gap,
        kerning_data,
        space_width,
        space_width_ratio,
    })
}

fn get_kerning_map(kerning_name: &str) -> Result<KerningMap, Error> {
    use std::io::BufRead;
    let mut m: KerningMap = HashMap::default();
    let p = get_kerning_p(kerning_name);
    if !p.exists() {
        return Ok(m);
    }
    let f = match std::fs::File::open(&p) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Cannot open kerning file {:?}: {:?}", p, e);
            return Err(Error::Kerning(KerningError { msg }));
        }
    };
    let bf = std::io::BufReader::new(f);
    for line in bf.lines() {
        let line = match line {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Error reading kerning file {:?}: {:?}", p, e);
                return Err(Error::Kerning(KerningError { msg }));
            }
        };
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split(" ");
        let prev = match parts.next() {
            Some(v) => v,
            None => {
                let msg = format!("Error parsing kerning data: {}", line);
                return Err(Error::Font(FontError { msg }));
                //return m;
            }
        };
        let next = match parts.next() {
            Some(v) => v,
            None => {
                let msg = format!("Error parsing kerning data: {}", line);
                return Err(Error::Font(FontError { msg }));
                //return m;
            }
        };
        let kern: f32 = match parts.next() {
            Some(v) => match v.parse::<f32>() {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!("Error parsing kerning data: {}\n{:?}", line, e);
                    return Err(Error::Font(FontError { msg }));
                    //return m;
                }
            },
            None => {
                let msg = format!("Error parsing kerning data: {}", line);
                return Err(Error::Font(FontError { msg }));
                //return m;
            }
        };
        let prev = prev.chars().next().unwrap();
        let next = next.chars().next().unwrap();
        let prev = prev as u16;
        let next = next as u16;
        let prev = KERN_JAMO_MAP.get(&prev).unwrap_or_else(|| &prev);
        let next = KERN_JAMO_MAP.get(&next).unwrap_or_else(|| &next);
        m.insert((*prev, *next), kern);
    }
    Ok(m)
}

fn get_glyph_filename(glyph_set: &str, glyph_name: &str) -> Option<PathBuf> {
    let filename = format!("{}.lua", glyph_name);
    let mut p = get_glyph_set_dir(glyph_set);
    p.push(filename.clone());
    if p.exists() {
        return Some(p);
    } else {
        return None;
    }
}

//
// Toolset
//
#[tauri::command]
fn get_tool_set_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_tool_sets_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let ext = p.extension().unwrap().to_str().unwrap();
        if ext == "toolset" {
            let s = p.file_stem().unwrap().to_str().unwrap();
            v.push(s.to_string());
        }
    }
    v.sort();
    v
}

#[tauri::command]
fn get_tool_set_data(tool_set_name: String) -> Result<ToolSet, Error> {
    crate::file::get_tool_set_data(&tool_set_name)
}

#[tauri::command]
fn delete_tool_set(tool_set_name: String) {
    crate::file::delete_tool_set(&tool_set_name);
}

#[tauri::command]
fn save_tool_set(tool_set: ToolSet, tool_set_name: String) {
    if tool_set.config_name.is_empty()
        || tool_set.kerning_name.is_empty()
        || tool_set.glyph_set.is_empty()
    {
        return;
    }
    let p = get_tool_set_p(&tool_set_name);
    let mut f = std::fs::File::create(&p).unwrap();
    let s = json5::to_string(&tool_set).unwrap();
    f.write_all(s.as_bytes()).unwrap();
}

/*#[tauri::command]
fn copy_tool_set(tool_set_name: String, new_tool_set_name: String) {
    let old_p = get_tool_set_p(&tool_set_name);
    let new_p = get_tool_set_p(&new_tool_set_name);
    if !old_p.exists() {
        return;
    }
    if new_p.exists() {
        std::fs::remove_file(&new_p).unwrap();
    }
    let mut f = std::fs::File::open(&old_p).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let j: String = json5::from_str(&s).unwrap();
    let mut f = std::fs::File::create(&new_p).unwrap();
    let b = j.as_bytes();
    f.write_all(b).unwrap();
}*/

//
// Config
//
#[tauri::command]
fn get_config_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_configs_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        v.push(s);
    }
    v.sort();
    v
}

#[tauri::command]
fn get_config_data(config_name: String) -> String {
    let config_name = if config_name.is_empty() {
        DEFAULT_NAME
    } else {
        config_name.as_str()
    };
    get_config_str(&config_name)
}

#[tauri::command]
fn save_config(config_data: String, config_name: String) {
    _save_config(&config_data, &config_name);
}

//
// Kerning
//
#[tauri::command]
fn get_kerning_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_kernings_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        v.push(s);
    }
    v
}

#[tauri::command]
fn get_kerning_data(kerning_name: String) -> String {
    get_kerning_str(kerning_name)
}

#[tauri::command]
fn save_kerning_data(kerning_data: String, kerning_name: String) {
    if kerning_data.is_empty() {
        return;
    }
    let p = get_kerning_p(&kerning_name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(kerning_data.as_bytes()).unwrap();
}

//
// Font
//
#[tauri::command]
fn get_font_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_fonts_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        match std::fs::metadata(&p) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    let s = p.file_stem().unwrap().to_string_lossy().to_string();
                    v.push(s);
                }
            }
            Err(_) => {}
        }
    }
    v.sort();
    v
}

#[tauri::command]
fn get_font_data(font_name: &str) -> Vec<u8> {
    crate::file::get_font_data(font_name)
}

#[tauri::command]
fn delete_font(font_name: String) -> Result<(), Error> {
    delete_font_dir(&font_name)
}

#[tauri::command]
fn delete_glyph_set(glyph_set: String) {
    crate::file::delete_glyph_set(&glyph_set);
}

#[tauri::command]
fn save_font(old_name: String, new_name: String) -> Result<(), Error> {
    let old_dir = get_font_dir(&old_name);
    let new_dir = get_font_dir(&new_name);
    if old_dir.exists() {
        if new_dir.exists() {
            match delete_font_dir(&new_name) {
                Ok(_) => {}
                Err(e) => {
                    return Err(Error::Font(FontError {
                        msg: format!("Error in deleting font {:?}: {:?}", new_name, e),
                    }))
                }
            }
        }
        match std::fs::rename(&old_dir, &new_dir) {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::Font(FontError {
                    msg: format!("Error in renaming {:?} to {:?}: {:?}", old_dir, new_dir, e),
                }))
            }
        }
    }
    for ext in &["ttf", "woff2"] {
        let mut p = get_font_dir(&new_name);
        let mut p2 = get_font_dir(&new_name);
        p.push(format!("{}.{}", old_name, ext));
        p2.push(format!("{}.{}", new_name, ext));
        if p.exists() {
            std::fs::rename(&p, &p2).unwrap();
        }
    }
    Ok(())
}

#[tauri::command]
fn get_glyph_set_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_glyph_sets_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        if !s.starts_with(".") {
            v.push(s);
        }
    }
    v.sort();
    v
}

#[tauri::command]
fn copy_glyph_set(glyph_set: String, new_glyph_set: String) {
    let old_p = get_glyph_set_dir(&glyph_set);
    let new_p = get_glyph_set_dir(&new_glyph_set);
    if !new_p.exists() {
        std::fs::create_dir(&new_p).unwrap();
    }
    let mut els = old_p.read_dir().unwrap();
    while let Some(el) = els.next() {
        let el = el.unwrap();
        let p = el.path();
        let mut np = new_p.clone();
        let s = p.file_name().unwrap().to_str().unwrap();
        np.push(s);
        if s.ends_with(".lua") {
            std::fs::copy(&p, &np).unwrap();
        }
    }
}

#[tauri::command]
fn save_glyph_data(glyph_set: String, glyph_name: String, glyph_data: String) {
    let filename = match get_glyph_filename(&glyph_set, &glyph_name) {
        Some(v) => v,
        None => return,
    };
    let mut f = std::fs::File::create(&filename).unwrap();
    f.write_all(glyph_data.as_bytes()).unwrap();
}

#[tauri::command]
fn get_glyph_data(glyph_set: String, glyph_name: String) -> String {
    let filename = match get_glyph_filename(&glyph_set, &glyph_name) {
        Some(v) => v,
        None => return "No data".to_string(),
    };
    let mut f = std::fs::File::open(&filename).unwrap();
    let mut s1 = String::new();
    f.read_to_string(&mut s1).unwrap();
    s1
}

#[tauri::command]
fn get_content_names() -> Vec<String> {
    let mut v: Vec<String> = Vec::with_capacity(128);
    let p = get_contents_dir();
    for entry in p.read_dir().unwrap() {
        let entry = entry.unwrap();
        let s = entry
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        v.push(s);
    }
    v.sort();
    v
}

#[tauri::command]
fn get_content(content_name: String) -> String {
    let mut s1: String = String::with_capacity(1024 * 1024);
    let p = get_content_p(&content_name);
    if p.exists() {
        let mut f = std::fs::File::open(&p).unwrap();
        f.read_to_string(&mut s1).unwrap();
    }
    s1
}

#[tauri::command]
fn save_content(content: String, content_name: String) {
    use std::io::Write;
    let p = get_content_p(&content_name);
    let mut f = std::fs::File::create(&p).unwrap();
    let bytes = content.as_bytes();
    f.write_all(bytes).unwrap();
}

fn copy_config_kern_glyph_files(
    font_name: &str,
    config_name: &str,
    kerning_name: &str,
    glyph_set: &str,
) {
    let font_dir = get_font_dir(font_name);
    // Config
    let old_p = get_config_p(config_name);
    let new_p = get_font_config_p(font_name);
    std::fs::copy(&old_p, &new_p).unwrap();
    // Kerning
    let old_p = get_kerning_p(kerning_name);
    let new_p = get_font_kerning_p(font_name);
    std::fs::copy(&old_p, &new_p).unwrap();
    // Glyph data
    let old_p = get_glyph_set_dir(glyph_set);
    let options = fs_extra::dir::CopyOptions::new().overwrite(true);
    fs_extra::copy_items(&[old_p], &font_dir, &options).unwrap();
    let mut old_p = font_dir.clone();
    old_p.push(glyph_set);
    let mut new_p = font_dir.clone();
    new_p.push(FONT_GLYPH_DATA_DIRNAME);
    std::fs::rename(&old_p, &new_p).unwrap();
}

#[tauri::command]
fn run_compile(
    app: AppHandle,
    config_name: String,
    kerning_name: String,
    glyph_set: String,
) -> Result<(), Error> {
    use tauri::Emitter;
    let config_name: String = if config_name.is_empty() {
        DEFAULT_NAME.to_string()
    } else {
        config_name.clone()
    };
    let config_str = get_config_str(&config_name);
    let mut config = match json5::from_str(&config_str) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("Error in parsing config: {:#?}\n{:#?}", e, config_str);
            return Err(Error::Config(ConfigError { msg }));
        }
    };
    let args = get_args(&mut config, &kerning_name)?;
    *CONFIG.write().unwrap() = args.clone();
    let _ = std::thread::spawn(move || {
        // Delete the existing new folder
        let font_dir = get_font_dir(COMPILED_FONT_NAME);
        if font_dir.exists() {
            match delete_font_dir(COMPILED_FONT_NAME) {
                Ok(_) => {}
                Err(e) => {
                    app.emit(
                        "error",
                        format!("Error in deleting font {}: {:?}", COMPILED_FONT_NAME, e),
                    )
                    .unwrap();
                    return;
                }
            }
        }
        match compile(&args, &glyph_set) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("compile error: {:?}", e);
                app.emit("error", format!("{:?}", e)).unwrap();
                return;
            }
        }
        if let Err(e) = make_woff2(&args) {
            app.emit("error", format!("{:?}", e)).unwrap();
            return;
        }
        copy_config_kern_glyph_files(COMPILED_FONT_NAME, &config_name, &kerning_name, &glyph_set);
        app.emit("msg", "compile_ended").unwrap();
    });
    Ok(())
}

pub fn just_compile() {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            if let Err(e) = create_data_folders(&app_handle) {
                eprintln!("warning: failed to initialize data folders: {:?}", e);
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            run_compile,
            // toolset
            get_tool_set_names,
            get_tool_set_data,
            delete_tool_set,
            save_tool_set,
            //copy_tool_set,
            // glyph set
            get_glyph_set_names,
            get_glyph_data,
            delete_glyph_set,
            copy_glyph_set,
            save_glyph_data,
            // config
            get_config_names,
            get_config_data,
            save_config,
            // kerning
            get_kerning_names,
            get_kerning_data,
            save_kerning_data,
            // content
            get_content_names,
            get_content,
            save_content,
            // font
            get_font_names,
            get_font_data,
            delete_font,
            save_font,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
