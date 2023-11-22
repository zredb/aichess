use resvg::usvg::{fontdb, Node, NodeKind, Options, TreeParsing, TreeTextToPath};
use resvg::{tiny_skia, usvg};
use std::path::Path;
use usvg::Tree;

fn main() -> anyhow::Result<()> {
    let file_data = include_str!("board.svg");

    //从resvg中没有找到合适的插入节点的方法， 直接用字符串方式写入
    Ok(())
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
