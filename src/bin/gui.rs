//! AIChess GUI 应用
//! 
//! 基于 egui + eframe 构建的图形界面应用

use eframe::egui;
use aichess::gui::app::ChessApp;

fn main() -> eframe::Result<()> {
    // 设置日志
    env_logger::init();
    
    // 窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("AIChess - 中国象棋 AI"),
        ..Default::default()
    };
    
    // 运行应用
    eframe::run_native(
        "AIChess",
        options,
        Box::new(|cc| Ok(Box::new(ChessApp::new(cc)))),
    )
}
