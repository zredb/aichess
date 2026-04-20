use eframe::egui;

use crate::gui::views::{TrainingView, PlayView, HumanView, PgnView, View};

/// AIChess 主应用
pub struct ChessApp {
    // 当前视图
    pub current_view: View,
    
    // 各视图状态
    pub training_view: TrainingView,
    pub play_view: PlayView,
    pub human_view: HumanView,
    pub pgn_view: PgnView,
    
    // 全局状态
    pub status_message: String,
}

impl ChessApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 设置中文字体支持
        Self::setup_fonts(cc);
        
        log::info!("Creating ChessApp...");
        
        let app = Self {
            current_view: View::Training,
            training_view: TrainingView::new(),
            play_view: PlayView::new(),
            human_view: HumanView::new(),
            pgn_view: PgnView::new(),
            status_message: "就绪".to_string(),
        };
        
        log::info!("ChessApp created successfully");
        app
    }
    
    /// 设置支持中文的字体
    fn setup_fonts(cc: &eframe::CreationContext<'_>) {
        use egui::{FontData, FontDefinitions, FontFamily};
        use std::sync::Arc;
        
        let mut fonts = FontDefinitions::default();
        
        // 优先使用编译时嵌入的字体（更可靠）
        // 如果嵌入的字体文件不存在，则回退到运行时加载
        
        #[cfg(target_os = "windows")]
        {
            // 运行时加载字体
            let font_paths = [
                "C:/Windows/Fonts/msyh.ttc",      // 微软雅黑
                "C:/Windows/Fonts/msyhbd.ttc",    // 微软雅黑 Bold
                "C:/Windows/Fonts/simsun.ttc",    // 宋体
                "C:/Windows/Fonts/simhei.ttf",    // 黑体
            ];
            
            for path in &font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        Arc::new(FontData::from_owned(font_data)),
                    );
                    log::info!("✓ Loaded Chinese font from: {}", path);
                    break;
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let font_paths = [
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Medium.ttc",
            ];
            
            for path in &font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        Arc::new(FontData::from_owned(font_data)),
                    );
                    log::info!("✓ Loaded Chinese font from: {}", path);
                    break;
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let font_paths = [
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
                "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            ];
            
            for path in &font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        Arc::new(FontData::from_owned(font_data)),
                    );
                    log::info!("✓ Loaded Chinese font from: {}", path);
                    break;
                }
            }
        }
        
        // 将中文字体插入到所有字体家族的最前面（最高优先级）
        for family in fonts.families.values_mut() {
            family.insert(0, "chinese_font".to_owned());
        }
        
        // 特别确保 Proportional 和 Monospace 家族包含中文字体
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "chinese_font".to_owned());
        fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "chinese_font".to_owned());
        
        // 应用到 egui 上下文
        cc.egui_ctx.set_fonts(fonts);
        
        log::info!("✓ Chinese font setup completed");
    }
    
    /// 显示菜单栏
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("文件", |ui| {
                if ui.button("退出").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            
            ui.menu_button("视图", |ui| {
                if ui.selectable_label(self.current_view == View::Training, "训练面板").clicked() {
                    self.current_view = View::Training;
                    ui.close_menu();
                }
                if ui.selectable_label(self.current_view == View::Play, "模型对弈").clicked() {
                    self.current_view = View::Play;
                    ui.close_menu();
                }
                if ui.selectable_label(self.current_view == View::Human, "人机对弈").clicked() {
                    self.current_view = View::Human;
                    ui.close_menu();
                }
                if ui.selectable_label(self.current_view == View::Pgn, "棋谱浏览").clicked() {
                    self.current_view = View::Pgn;
                    ui.close_menu();
                }
            });
            
            ui.menu_button("帮助", |ui| {
                if ui.button("关于").clicked() {
                    ui.close_menu();
                }
            });
        });
    }
    
    /// 显示侧边栏
    fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading("AIChess");
                    ui.add_space(10.0);
                });
                
                ui.separator();
                
                // 导航按钮
                ui.selectable_value(&mut self.current_view, View::Training, "🎯 训练");
                ui.selectable_value(&mut self.current_view, View::Play, "⚔️ 对弈");
                ui.selectable_value(&mut self.current_view, View::Human, "👤 人机");
                ui.selectable_value(&mut self.current_view, View::Pgn, "📜 棋谱");
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                // 状态信息
                ui.small(format!("状态: {}", self.status_message));
            });
    }
    
    /// 显示底部状态栏
    fn bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(30.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.small(&self.status_message);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small("v0.1.0");
                    });
                });
            });
    }
    
    /// 显示关于对话框
    fn show_about(&mut self, ctx: &egui::Context) {
        egui::Window::new("关于 AIChess")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("AIChess");
                    ui.label("基于 AlphaZero 的中国象棋 AI");
                    ui.add_space(10.0);
                    ui.label("版本: 0.1.0");
                    ui.label("技术栈: Rust + Burn + MCTS");
                });
            });
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 顶部菜单栏
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.menu_bar(ui);
        });
        
        // 左侧边栏
        self.side_panel(ctx);
        
        // 底部状态栏
        self.bottom_panel(ctx);
        
        // 中央内容区
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                View::Training => {
                    self.training_view.show(ui);
                    self.status_message = "训练面板".to_string();
                }
                View::Play => {
                    self.play_view.show(ui);
                    self.status_message = "模型对弈".to_string();
                }
                View::Human => {
                    self.human_view.show(ui);
                    self.status_message = "人机对弈".to_string();
                }
                View::Pgn => {
                    self.pgn_view.show(ui);
                    self.status_message = "棋谱浏览".to_string();
                }
            }
        });
        
        // 显示关于对话框（如果需要）
        // TODO: 添加一个标志来控制是否显示
        
        // 请求重绘（用于动画和实时更新）
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
