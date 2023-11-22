use aichess::position::{file_x, rank_y, Position};
use resvg::usvg::{fontdb, Node, NodeKind, Options, TreeParsing, TreeTextToPath};
use resvg::{tiny_skia, usvg};
use std::fs;
use std::path::Path;
const PIECE_SIZE: u32 = 50;
const HALF_PIECE_SIZE: u32 = 25;
const LOC_STRING: &str = "<use href=\"#board\" transform=\"translate(0,0)\" />";

fn main() -> anyhow::Result<()> {
    let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1";
    let board = Position::from_fen(fen);
    let piece_locs = board.piece_loc();
    let mut template = fs::read_to_string("board.svg").unwrap();
    for (piece, loc) in piece_locs {
        let s = build_char_loc_str(piece, loc);
        insert_substring(&mut template, LOC_STRING, &s);
    }
    svg2png(&template, "test.png");
    Ok(())
    //从resvg中没有找到合适的插入节点的方法， 直接用字符串方式写入
}
fn build_char_loc_str(piece: char, loc: u8) -> String {
    // <use href="#r" transform="translate(25,25)" />
    let x = file_x(loc as usize);
    let y = rank_y(loc as usize);
    format!(" <use href=\"#{piece}\" transform=\"translate({x},{y})\" />")
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
