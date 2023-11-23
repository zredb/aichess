use std::fs;
use std::path::Path;

use resvg::{tiny_skia, usvg};
use resvg::usvg::{fontdb, Options, TreeParsing, TreeTextToPath};

use aichess::{FILE_LEFT, file_x, RANK_TOP, rank_y};
use aichess::fen::fen2_coords;

const PIECE_SIZE: usize = 50;
const HALF_PIECE_SIZE: usize = 25;
const LOC_STRING: &str = "<use href=\"#board\" transform=\"translate(0,0)\" />";

fn main() -> anyhow::Result<()> {
    let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
    let piece_locs = fen2_coords(fen);
    let mut template = include_str!("board.svg").to_string();
    for (piece, loc) in piece_locs {
        let s = build_char_loc_str(piece, loc);
        insert_substring(&mut template, LOC_STRING, &s);
    }
    fs::write("chess_test.svg", &template).unwrap();
    svg2png(&template, "chess_test.png").expect("TODO: panic message");

    Ok(())
    //从resvg中没有找到合适的插入节点的方法， 直接用字符串方式写入
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

fn svg2png<P: AsRef<Path>>(svg_str: &str, dst: P) -> anyhow::Result<()> {
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
