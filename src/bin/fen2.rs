use std::fs;
use std::path::Path;
use clap::Parser;
use resvg::{tiny_skia, usvg};
use resvg::usvg::{fontdb, Options, TreeParsing, TreeTextToPath};

use aichess::{FILE_LEFT, file_x, RANK_TOP, rank_y};
use aichess::fen::fen2_coords;

const PIECE_SIZE: usize = 50;
const HALF_PIECE_SIZE: usize = 25;
const LOC_STRING: &str = "<use href=\"#board\" transform=\"translate(0,0)\" />";
#[derive(Parser,  Debug,Copy, Clone)]
enum OutputFormat{
    Svg,
    Png,
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///输入的Fen字符串
    #[arg(short, long)]
    fen: String,
   ///输出文件格式, 目前支持svg,png,
    #[arg(short, long)]
    output_format: OutputFormat,
    /// 输出文件路径
    #[arg(short('p'), long)]
    dest_path: String,

}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.output_format{
        OutputFormat::Png=>{
            svg2png_file(&fen2svg(args.fen)?, args.dest_path)?
        },
        OutputFormat::Svg=>{

            fen2svg_file(args.fen, args.dest_path)?
        }
    }

    Ok(())

}

fn build_char_loc_str(piece: char, loc: u8) -> String {
    // <use href="#r" transform="translate(25,25)" />
    let x = (file_x(loc as usize) - FILE_LEFT) * PIECE_SIZE + HALF_PIECE_SIZE;
    let y = (rank_y(loc as usize) - RANK_TOP) * PIECE_SIZE + HALF_PIECE_SIZE;
    format!("\n        <use href=\"#{piece}\" transform=\"translate({x},{y})\" />")
}

fn insert_substring(original_string: &mut String, substring1: &str, substring2: &str) {
    if let Some(index) = original_string.find(substring1) {
        original_string.insert_str(index + substring1.len(), substring2);
    }
}
fn fen2svg(fen: &str) -> anyhow::Result<String> {
   // let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
    let piece_locs = fen2_coords(fen);
    let mut template = include_str!("board.svg").to_string();
    for (piece, loc) in piece_locs {
        let s = build_char_loc_str(piece, loc);
        insert_substring(&mut template, LOC_STRING, &s);
    }

    //fs::write(dst, &template)?;
    Ok(template)
}
pub fn fen2svg_file<P: AsRef<Path>>(fen: &str, dst: P) -> anyhow::Result<()> {
    let svg_str=fen2svg(fen)?;
    fs::write(dst, svg_str)?;
    Ok(())
}

pub fn svg2png_file<P: AsRef<Path>>(svg_str: &str, dst: P) -> anyhow::Result<()> {
    let rtree = {
        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        let mut tree = usvg::Tree::from_str(&svg_str, &Options::default()).unwrap();
        tree.convert_text(&fontdb);
        resvg::Tree::from_usvg(&tree)
    };

    let pixmap_size = rtree.size.to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    rtree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

    pixmap.save_png(dst)?;
    Ok(())
}
