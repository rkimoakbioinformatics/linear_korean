mod app;
mod compose;
mod consts;
mod font;
mod glyph;
mod glyph_units;
mod structs;
use crate::app::*;
use crate::compose::*;
use crate::font::*;
use consts::*;
use structs::*;
use std::io::Read;

pub fn compile(args: &Args) {
    *ARGS.write().unwrap() = args.clone();
    let font_bytes = std::fs::read(&args.source_filename).unwrap();
    let font = write_fonts::read::FontRef::new(&font_bytes).unwrap();
    let (mut font_tables, builder) =
        get_font_tables_and_builder(&font);
    let mut codepoints: Vec<u16> = font_tables
        .codepoint_to_glyph_id
        .keys()
        .map(|&v| v)
        .collect();
    codepoints.sort();
    /*for codepoint in codepoints.iter() {
        println!("codepoint {:x} -> glyph {}", codepoint, font_tables.codepoint_to_glyph_id.get(codepoint).unwrap());
    }*/
    /*println!("Originally {} glyphs in maxp", font_tables.maxp.num_glyphs);
    println!("{} glyphs", font_tables.glyphs.len());
    println!(
        "{} glyph names: {:?}",
        font_tables.glyph_names.len(),
        font_tables.glyph_names
    );*/
    map_composite_chosungs(&mut font_tables);
    generate_glyphs(&mut font_tables);
    //add_eng_font(&mut font_tables);
    modify_post(&mut font_tables);
    modify_maxp(&mut font_tables);
    modify_head_hhea(&mut font_tables);
    build_font(&font_tables, builder, &args.target_filename);
}

pub fn make_woff2(ttf_filename: &str) {
    let p = std::path::Path::new(ttf_filename);
    let woff2_filename = p.with_extension("woff2");
    println!("woff2 ttf: {:?}, woff2: {:?}", ttf_filename, woff2_filename);
    let output = std::process::Command::new("fonttools")
        .arg("ttLib.woff2")
        .arg("compress")
        .arg(ttf_filename)
        .arg("-o")
        .arg(&woff2_filename)
        .output()
        .expect("WOFF2 conversion failed.");
    println!("{}", std::str::from_utf8(&output.stdout).unwrap());
    println!("{}", std::str::from_utf8(&output.stderr).unwrap());
}

pub fn get_font_data(args: &Args) -> Vec<u8> {
    let mut f = std::fs::File::open(&args.target_filename).unwrap();
    let mut font_data: Vec<u8> = Vec::new();
    f.read_to_end(&mut font_data).unwrap();
    font_data
}

pub fn get_args(cli: &Cli) -> Args {
    let source_filename: String = cli.source.clone();
    let p = std::path::PathBuf::from(&source_filename);
    if !p.exists() {
        eprintln!("{} does not exist.", cli.source);
        std::process::exit(1);
    }
    let source_p: std::path::PathBuf = p.canonicalize().unwrap();
    let target_p = p.parent().unwrap().to_path_buf();
    let mut target_p = target_p.canonicalize().unwrap();
    target_p.push(format!("{}.ttf", cli.target.clone()));
    let woff2_p = target_p.with_extension("woff2");
    let source_filename = source_p.to_str().unwrap().to_string();
    let target_filename = target_p.to_str().unwrap().to_string();
    let woff2_filename = woff2_p.to_str().unwrap().to_string();
    let jung_type_v = cli.jung_type.clone();
    let mut jung_type: u8 = 0;
    jung_type_v.iter().for_each(|v| {
        if v == "underdot" {
            jung_type += UNDERDOT;
        } else if v == "underbar" {
            jung_type += UNDERBAR;
        }
    });
    let jong_type_v = cli.jong_type.clone();
    let mut jong_type: u8 = 0;
    jong_type_v.iter().for_each(|v| {
        if v == "underdot" {
            jong_type += UNDERDOT;
        } else if v == "underbar" {
            jong_type += UNDERBAR;
        }
    });
    let jung_h_ratio: f32 = cli.jung_h_ratio.unwrap_or(1.0);
    let jong_h_ratio: f32 = cli.jong_h_ratio.unwrap_or(1.0);
    let char_gap: u16 = cli.char_gap.unwrap_or(0);
    let cho_gap: u16 = cli.cho_gap.unwrap_or(0);
    let jung_gap: u16 = cli.jung_gap.unwrap_or(0);
    let jong_gap: u16 = cli.jong_gap.unwrap_or(0);
    let sw: i16 = cli.sw.unwrap_or(160);
    let text_size: u16 = cli.text_size.unwrap_or(16);
    Args {
        source_filename,
        target_filename,
        woff2_filename,
        jung_type,
        jong_type,
        jung_h_ratio,
        jong_h_ratio,
        char_gap,
        cho_gap,
        jung_gap,
        jong_gap,
        sw,
        text_size,
    }
}

#[derive(clap::Parser)]
pub struct Cli {
    #[arg(long)]
    source: String,
    #[arg(long)]
    target: String,
    #[arg(long)]
    jung_type: Vec<String>,
    #[arg(long)]
    jong_type: Vec<String>,
    #[arg(long)]
    jung_h_ratio: Option<f32>,
    #[arg(long)]
    jong_h_ratio: Option<f32>,
    #[arg(long)]
    char_gap: Option<u16>,
    #[arg(long)]
    cho_gap: Option<u16>,
    #[arg(long)]
    jung_gap: Option<u16>,
    #[arg(long)]
    jong_gap: Option<u16>,
    #[arg(long)]
    sw: Option<i16>,
    #[arg(long)]
    text_size: Option<u16>,
}

fn main() -> iced::Result {
    use clap::Parser;
    let cli = Cli::parse();
    let args = get_args(&cli);
    println!("Args: {:#?}", args);
    compile(&args);
    make_woff2(&args.target_filename);
    let font_data = get_font_data(&args);
    iced::application("Linear Korean", App::update, App::view)
        .window_size((1000.0, 800.0))
        .font(font_data)
        .run_with(move || App::new(args))
}
