use crate::consts::*;
use crate::error::*;
use crate::structs::ToolSet;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::path::BaseDirectory;
use tauri::AppHandle;
use tauri::Manager;

static ROOT_DIR: OnceLock<PathBuf> = OnceLock::new();

//
// Initialization
//
fn resolve_root_dir(app: &AppHandle) -> Result<PathBuf, Error> {
    if let Some(p) = ROOT_DIR.get() {
        return Ok(p.clone());
    }
    let root = match app.path().app_data_dir() {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Config(ConfigError {
                msg: format!("Cannot resolve app data dir: {:?}", e),
            }));
        }
    };
    let _ = ROOT_DIR.set(root.clone());
    Ok(ROOT_DIR.get().cloned().unwrap_or(root))
}

pub fn get_root_dir() -> PathBuf {
    if let Some(p) = ROOT_DIR.get() {
        return p.clone();
    }
    // Fallback for contexts that call path helpers before Tauri setup.
    let p = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    p
}

fn resolve_seed_dir(app: &AppHandle, dirname: &str) -> Option<PathBuf> {
    if let Ok(v) = app.path().resolve(dirname, BaseDirectory::Resource) {
        if v.exists() {
            return Some(v);
        }
    }
    if cfg!(debug_assertions) {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _ = p.pop();
        p.push(dirname);
        if p.exists() {
            eprintln!(
                "info: using development seed dir fallback for `{}` at {:?}",
                dirname, p
            );
            return Some(p);
        }
    }
    eprintln!(
        "warning: cannot resolve seed dir `{}` from bundled resources or dev fallback",
        dirname
    );
    None
}

pub fn create_data_folders(app: &AppHandle) -> Result<(), Error> {
    let root_p = resolve_root_dir(app)?;
    if let Err(e) = std::fs::create_dir_all(&root_p) {
        return Err(Error::Config(ConfigError {
            msg: format!("Cannot create data root {:?}: {:?}", root_p, e),
        }));
    }
    let seed_dirs = [
        GLYPH_SETS_DIRNAME,
        CONFIGS_DIRNAME,
        KERNINGS_DIRNAME,
        EVOLUTION_DIRNAME,
        FONTS_DIRNAME,
        CONTENTS_DIRNAME,
        TOOLSETS_DIRNAME,
    ];
    for dirname in seed_dirs {
        let mut target_dir = root_p.clone();
        target_dir.push(dirname);
        if target_dir.exists() {
            continue;
        }
        let source_dir = resolve_seed_dir(app, dirname);
        if let Some(source_dir) = source_dir {
            if source_dir.exists() {
                let options = fs_extra::dir::CopyOptions::new().overwrite(false);
                if let Err(e) = fs_extra::copy_items(&[source_dir], &root_p, &options) {
                    eprintln!(
                        "warning: cannot seed `{}` into {:?}: {:?}",
                        dirname, target_dir, e
                    );
                    if let Err(e2) = std::fs::create_dir_all(&target_dir) {
                        eprintln!(
                            "warning: cannot create empty fallback dir {:?}: {:?}",
                            target_dir, e2
                        );
                    }
                }
                continue;
            }
        }
        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            eprintln!(
                "warning: cannot create empty fallback dir {:?}: {:?}",
                target_dir, e
            );
        }
    }
    Ok(())
}

//
// Toolset
//
pub fn get_tool_sets_dir() -> PathBuf {
    let mut p = get_root_dir();
    p.push(TOOLSETS_DIRNAME);
    p
}

pub fn get_tool_set_p(tool_set: &str) -> PathBuf {
    let mut p = get_tool_sets_dir();
    p.push(format!("{}.toolset", tool_set));
    p
}

pub fn get_tool_set_data(tool_set: &str) -> Result<ToolSet, Error> {
    let p = get_tool_set_p(tool_set);
    if !p.exists() {
        return Err(Error::Config(ConfigError {
            msg: format!("Toolset {} does not exist.", tool_set),
        }));
    }
    let mut f = std::fs::File::open(&p).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let tool_set = match json5::from_str(&s) {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Config(ConfigError {
                msg: format!("Error parsing toolset {}: {:#?}", tool_set, e),
            }));
        }
    };
    Ok(tool_set)
}

pub fn delete_tool_set(tool_set: &str) {
    use std::fs::remove_file;
    let mut p = get_tool_sets_dir();
    p.push(format!("{}.toolset", tool_set));
    if !p.exists() {
        return;
    }
    remove_file(&p).unwrap();
}

//
// Content
//
pub fn get_contents_dir() -> PathBuf {
    let mut p = get_root_dir();
    p.push(CONTENTS_DIRNAME);
    p
}

pub fn get_content_p(content_name: &str) -> PathBuf {
    let mut p = get_contents_dir();
    p.push(content_name);
    p
}

//
// Font
//
pub fn get_fonts_dir() -> PathBuf {
    let mut p = get_root_dir();
    p.push(FONTS_DIRNAME);
    p
}

pub fn get_font_dir(font_name: &str) -> PathBuf {
    let mut p = get_fonts_dir();
    p.push(font_name);
    p
}

pub fn get_font_woff2_p(font_name: &str) -> PathBuf {
    let mut p = get_root_dir();
    p.push(FONTS_DIRNAME);
    p.push(font_name);
    p.push(format!("{}.woff2", font_name));
    p
}

pub fn get_font_ttf_p(font_name: &str) -> PathBuf {
    let mut p = get_root_dir();
    p.push(FONTS_DIRNAME);
    p.push(font_name);
    p.push(format!("{}.ttf", font_name));
    p
}

pub fn get_font_config_p(font_name: &str) -> PathBuf {
    let mut p = get_font_dir(font_name);
    p.push(FONT_CONFIG_FILENAME);
    p
}

pub fn get_font_kerning_p(font_name: &str) -> PathBuf {
    let mut p = get_font_dir(font_name);
    p.push(FONT_KERNING_FILENAME);
    p
}

pub fn get_font_data(font_name: &str) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(1024 * 1024);
    let mut p = get_font_dir(font_name);
    p.push(format!("{}.ttf", font_name));
    if !p.exists() {
        p = get_font_dir(font_name);
        p.push(format!("{}.woff2", font_name));
        if !p.exists() {
            return data;
        }
    }
    let mut f = std::fs::File::open(&p).unwrap();
    f.read_to_end(&mut data).unwrap();
    data
}

pub fn delete_font_dir(font_name: &str) -> Result<(), Error> {
    use std::fs::remove_file;
    let parent = get_font_dir(font_name);
    if !parent.exists() {
        return Ok(());
    }
    let mut p = parent.clone();
    p.push(FONT_CONFIG_FILENAME);
    if p.exists() {
        remove_file(&p).unwrap();
    }
    let mut p = parent.clone();
    p.push(FONT_KERNING_FILENAME);
    if p.exists() {
        remove_file(&p).unwrap();
    }
    let mut p = parent.clone();
    p.push(format!("{}.ttf", font_name));
    if p.exists() {
        remove_file(&p).unwrap();
    }
    let mut p = parent.clone();
    p.push(format!("{}.woff2", font_name));
    if p.exists() {
        remove_file(&p).unwrap();
    }
    let mut p = parent.clone();
    p.push(".DS_Store");
    if p.exists() {
        std::fs::remove_file(&p).unwrap();
    }
    let mut p = parent.clone();
    p.push("glyph_data");
    if p.exists() {
        std::fs::remove_dir_all(&p).unwrap();
    }
    match std::fs::remove_dir(&parent) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Font(FontError {
            msg: format!("{:?}", e),
        })),
    }
}

pub fn delete_glyph_set(glyph_set: &str) {
    use std::fs::remove_file;
    let parent = get_glyph_set_dir(glyph_set);
    if !parent.exists() {
        return;
    }
    let mut entries = parent.read_dir().unwrap();
    while let Some(entry) = entries.next() {
        let entry = entry.unwrap();
        let p = entry.path();
        if p.extension().unwrap().to_str().unwrap() == "lua" {
            remove_file(&p).unwrap();
        }
    }
    std::fs::remove_dir(&parent).unwrap();
}

//
// Config
//
pub fn get_configs_dir() -> PathBuf {
    let mut p = get_root_dir();
    p.push(CONFIGS_DIRNAME);
    p
}

pub fn get_config_p(config_name: &str) -> PathBuf {
    let mut p = get_configs_dir();
    p.push(format!("{}.json5", config_name));
    p
}

pub fn get_config_str(config_name: &str) -> String {
    let p = get_config_p(config_name);
    if !p.exists() {
        return String::new();
    }
    let mut f = match std::fs::File::open(&p) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warning: cannot open config {:?}: {:?}", p, e);
            return String::new();
        }
    };
    let mut s1 = String::new();
    if let Err(e) = f.read_to_string(&mut s1) {
        eprintln!("warning: cannot read config {:?}: {:?}", p, e);
        return String::new();
    }
    let s1 = s1.replace(r"“", "\"");
    let s1 = s1.replace("'", "\"");
    s1
}

pub fn _save_config(config_data: &str, config_name: &str) {
    if config_data.is_empty() {
        return;
    }
    let config_name = if config_name.is_empty() {
        DEFAULT_NAME
    } else {
        config_name
    };
    let p = get_config_p(config_name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(config_data.as_bytes()).unwrap();
}

//
// Evolution
//
pub fn get_evolution_dir() -> PathBuf {
    let mut p = get_root_dir();
    p.push(EVOLUTION_DIRNAME);
    p
}

pub fn get_evolution_p(evolution_name: &str) -> PathBuf {
    let evolution_name = if evolution_name.is_empty() {
        DEFAULT_NAME
    } else {
        evolution_name
    };
    let mut p = get_evolution_dir();
    p.push(format!("{}.json5", evolution_name));
    p
}

pub fn get_evolution_str(evolution_name: &str) -> String {
    let p = get_evolution_p(evolution_name);
    if !p.exists() {
        return String::new();
    }
    let mut f = match std::fs::File::open(&p) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warning: cannot open evolution config {:?}: {:?}", p, e);
            return String::new();
        }
    };
    let mut s1 = String::new();
    if let Err(e) = f.read_to_string(&mut s1) {
        eprintln!("warning: cannot read evolution config {:?}: {:?}", p, e);
        return String::new();
    }
    let s1 = s1.replace(r"“", "\"");
    let s1 = s1.replace("'", "\"");
    s1
}

pub fn _save_evolution_config(evolution_data: &str, evolution_name: &str) {
    if evolution_data.is_empty() {
        return;
    }
    let evolution_name = if evolution_name.is_empty() {
        DEFAULT_NAME
    } else {
        evolution_name
    };
    let p = get_evolution_p(evolution_name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(evolution_data.as_bytes()).unwrap();
}

//
// Kerning
//
pub fn get_kernings_dir() -> PathBuf {
    let mut p = get_root_dir();
    p.push(KERNINGS_DIRNAME);
    p
}

pub fn get_kerning_p(kerning_name: &str) -> PathBuf {
    let kerning_name = if kerning_name.is_empty() {
        DEFAULT_NAME
    } else {
        kerning_name
    };
    let mut p = get_kernings_dir();
    p.push(format!("{}.txt", kerning_name));
    p
}

pub fn get_kerning_str(kerning_name: String) -> String {
    let p = get_kerning_p(&kerning_name);
    if !p.exists() {
        return String::new();
    }
    let mut f = match std::fs::File::open(&p) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warning: cannot open kerning {:?}: {:?}", p, e);
            return String::new();
        }
    };
    let mut s1 = String::new();
    if let Err(e) = f.read_to_string(&mut s1) {
        eprintln!("warning: cannot read kerning {:?}: {:?}", p, e);
        return String::new();
    }
    let s1 = s1.replace(r"“", "\"");
    let s1 = s1.replace("'", "\"");
    s1
}

//
// Glyph data
//
pub fn get_glyph_sets_dir() -> PathBuf {
    let mut p = get_root_dir();
    p.push(GLYPH_SETS_DIRNAME);
    p
}

pub fn get_glyph_set_dir(glyph_set: &str) -> PathBuf {
    let mut p = get_glyph_sets_dir();
    p.push(glyph_set);
    p
}
