use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "insta",
    version,
    about = "Fast Instagram media downloader for Windows with CLI and Web UI"
)]
pub struct Cli {
    #[arg(short, long, value_name = "URL")]
    pub download: Option<String>,

    #[arg(long)]
    pub ui: bool,

    #[arg(short, long, default_value_t = 2022)]
    pub port: u16,

    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    #[arg(short, long, value_name = "COOKIE")]
    pub cookie: Option<String>,
}
