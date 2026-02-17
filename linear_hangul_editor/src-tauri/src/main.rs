// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[derive(Debug, clap::Parser, serde::Serialize, serde::Deserialize)]
pub struct Cli {
    #[arg(long)]
    just_compile: bool,
}

fn main() {
    use clap::Parser;
    let cli = Cli::parse();
    if cli.just_compile {
        linhan_lib::just_compile();
    } else {
        linhan_lib::run();
    }
}
