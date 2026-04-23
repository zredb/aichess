//! AIChess GUI 应用
//! 
//! 基于 egui + eframe 构建的图形界面应用

use eframe::egui;
use aichess::gui::app::ChessApp;

fn main() -> eframe::Result<()> {
    // 设置日志输出到文件
    use std::fs::OpenOptions;
    
    // 使用当前可执行文件所在目录作为日志目录
    let log_path = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            parent.join("aichess_gui.log")
        } else {
            std::path::PathBuf::from("aichess_gui.log")
        }
    } else {
        std::path::PathBuf::from("aichess_gui.log")
    };
    
    let log_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file {:?}: {}", log_path, e);
            panic!("Cannot open log file");
        }
    };
    
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    log::info!("AIChess GUI starting...");
    log::info!("Log file: {:?}", log_path);
    
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
