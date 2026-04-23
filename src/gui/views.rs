use eframe::egui;
use std::path::PathBuf;
use std::time::Instant;

use crate::fen::Fen;
use crate::gui::widgets::ChessBoardWidget;

/// 视图类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Training,
    Play,
    Human,
    Pgn,
}

/// 训练视图
pub struct TrainingView {
    pub log_dir: String,
    pub iterations: usize,
    pub games_per_train: usize,
    pub num_explores: usize,
    pub hidden_size: usize,
    pub num_blocks: usize,
    pub is_training: bool,
    pub progress: f32,
    pub status: String,
    // 训练数据（用于绘图）
    pub loss_data: Vec<(f64, f64)>,
    // 动画相关
    animation_start_time: Option<Instant>,
}

impl TrainingView {
    pub fn new() -> Self {
        Self {
            log_dir: "./logs".to_string(),
            iterations: 50,
            games_per_train: 100,
            num_explores: 400,
            hidden_size: 256,
            num_blocks: 7,
            is_training: false,
            progress: 0.0,
            status: "就绪".to_string(),
            loss_data: vec![],
            animation_start_time: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎯 AlphaZero 训练");
        ui.add_space(10.0);
        
        // 训练参数配置
        egui::Grid::new("training_params")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("日志目录:");
                ui.text_edit_singleline(&mut self.log_dir);
                ui.end_row();
                
                ui.label("迭代次数:");
                ui.add(egui::Slider::new(&mut self.iterations, 1..=1000));
                ui.end_row();
                
                ui.label("每轮游戏数:");
                ui.add(egui::Slider::new(&mut self.games_per_train, 10..=1000));
                ui.end_row();
                
                ui.label("MCTS 探索次数:");
                ui.add(egui::Slider::new(&mut self.num_explores, 100..=3200));
                ui.end_row();
                
                ui.label("隐藏层大小:");
                ui.add(egui::Slider::new(&mut self.hidden_size, 64..=1024).logarithmic(true));
                ui.end_row();
                
                ui.label("网络块数:");
                ui.add(egui::Slider::new(&mut self.num_blocks, 1..=20));
                ui.end_row();
            });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 训练控制
        ui.horizontal(|ui| {
            if !self.is_training {
                if ui.button("▶️ 开始训练").clicked() {
                    self.start_training();
                }
            } else {
                if ui.button("⏸️ 暂停").clicked() {
                    self.status = "已暂停".to_string();
                }
                if ui.button("⏹️ 停止").clicked() {
                    self.stop_training();
                }
            }
            
            ui.label(format!("进度: {:.1}%", self.progress * 100.0));
        });
        
        ui.add_space(10.0);
        
        // 进度条
        ui.add(egui::ProgressBar::new(self.progress)
            .text(if self.is_training { 
                format!("{:.0}%", self.progress * 100.0) 
            } else { 
                "未开始".to_string() 
            }));
        
        ui.add_space(10.0);
        
        // 训练状态和动态图形
        ui.horizontal(|ui| {
            // 如果正在训练，显示动态图形
            if self.is_training {
                self.draw_training_animation(ui);
                ui.add_space(10.0);
            }
            
            ui.label(format!("状态: {}", self.status));
        });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 训练曲线图
        ui.heading("📊 训练曲线");
        ui.add_space(10.0);
        
        if self.loss_data.is_empty() {
            ui.label("暂无训练数据");
        } else {
            use egui_plot::{Line, Plot, PlotPoints};
            
            let points: PlotPoints = self.loss_data.iter()
                .map(|&(x, y)| [x, y])
                .collect::<Vec<_>>()
                .into();
            let line = Line::new(points).name("总损失");
            
            Plot::new("loss_plot")
                .view_aspect(2.0)
                .label_formatter(|name, value| {
                    format!("{}: {:.4}", name, value.y)
                })
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        }
    }
    
    /// 绘制训练动画
    fn draw_training_animation(&mut self, ui: &mut egui::Ui) {
        // 初始化动画开始时间
        if self.animation_start_time.is_none() {
            self.animation_start_time = Some(Instant::now());
        }
        
        // 计算动画角度（每秒旋转一圈）
        let elapsed = self.animation_start_time.unwrap().elapsed();
        let angle = (elapsed.as_secs_f32() * std::f32::consts::TAU) % std::f32::consts::TAU;
        
        // 创建一个小画布来绘制动画
        let size = egui::vec2(40.0, 40.0);
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
        
        // 获取 painter
        let painter = ui.painter_at(rect);
        
        // 绘制外圈圆环
        let center = rect.center();
        let radius = 15.0;
        
        // 绘制背景圆环
        painter.circle_stroke(
            center,
            radius,
            egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 100, 100)),
        );
        
        // 绘制旋转的圆弧（表示进度）
        let arc_length = std::f32::consts::FRAC_PI_2; // 90度的弧
        let start_angle = angle;
        let end_angle = angle + arc_length;
        
        // 将圆弧分成多个小线段来绘制
        let segments = 20;
        let mut points = Vec::new();
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let a = start_angle + t * (end_angle - start_angle);
            let x = center.x + radius * a.cos();
            let y = center.y + radius * a.sin();
            points.push(egui::pos2(x, y));
        }
        
        // 绘制渐变色圆弧
        for i in 0..points.len() - 1 {
            let t = i as f32 / (points.len() - 1) as f32;
            let color = egui::Color32::from_rgb(
                (50.0 + t * 205.0) as u8,
                (150.0 + t * 105.0) as u8,
                255,
            );
            painter.line_segment(
                [points[i], points[i + 1]],
                egui::Stroke::new(3.0, color),
            );
        }
        
        // 在中心绘制一个小点
        painter.circle_filled(center, 4.0, egui::Color32::WHITE);
        
        // 请求重绘以继续动画
        ui.ctx().request_repaint();
    }
    
    fn start_training(&mut self) {
        self.is_training = true;
        self.progress = 0.0;
        self.status = "训练中...".to_string();
        self.loss_data.clear();
        self.animation_start_time = Some(Instant::now());
        
        // TODO: 在实际实现中，这里会启动后台训练任务
        // 目前只是模拟进度
    }
    
    fn stop_training(&mut self) {
        self.is_training = false;
        self.status = "已停止".to_string();
        self.animation_start_time = None;
    }
}

/// 模型对弈视图
pub struct PlayView {
    pub model1_path: String,
    pub model2_path: String,
    pub num_games: usize,
    pub num_explores: usize,
    pub is_playing: bool,
    pub current_game: usize,
    pub wins_p1: usize,
    pub wins_p2: usize,
    pub draws: usize,
}

impl PlayView {
    pub fn new() -> Self {
        Self {
            model1_path: "./models/model1.ot".to_string(),
            model2_path: "./models/model2.ot".to_string(),
            num_games: 10,
            num_explores: 400,
            is_playing: false,
            current_game: 0,
            wins_p1: 0,
            wins_p2: 0,
            draws: 0,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚔️ 模型对弈");
        ui.add_space(10.0);
        
        // 模型配置
        egui::Grid::new("play_params")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("模型 1:");
                ui.text_edit_singleline(&mut self.model1_path);
                ui.end_row();
                
                ui.label("模型 2:");
                ui.text_edit_singleline(&mut self.model2_path);
                ui.end_row();
                
                ui.label("对弈局数:");
                ui.add(egui::Slider::new(&mut self.num_games, 1..=100));
                ui.end_row();
                
                ui.label("MCTS 探索:");
                ui.add(egui::Slider::new(&mut self.num_explores, 100..=3200));
                ui.end_row();
            });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 控制按钮
        if !self.is_playing {
            if ui.button("▶️ 开始对弈").clicked() {
                self.start_play();
            }
        } else {
            if ui.button("⏹️ 停止").clicked() {
                self.stop_play();
            }
        }
        
        ui.add_space(10.0);
        
        // 统计信息
        ui.group(|ui| {
            ui.heading("📊 对弈统计");
            ui.add_space(5.0);
            ui.label(format!("当前局数: {}/{}", self.current_game, self.num_games));
            ui.label(format!("模型 1 胜: {}", self.wins_p1));
            ui.label(format!("模型 2 胜: {}", self.wins_p2));
            ui.label(format!("平局: {}", self.draws));
        });
    }
    
    fn start_play(&mut self) {
        self.is_playing = true;
        self.current_game = 0;
        self.wins_p1 = 0;
        self.wins_p2 = 0;
        self.draws = 0;
        // TODO: 启动后台对弈任务
    }
    
    fn stop_play(&mut self) {
        self.is_playing = false;
    }
}

/// 人机对弈视图
pub struct HumanView {
    pub model_path: String,
    pub player_color: String,
    pub num_explores: usize,
    pub game_status: String,
    pub chess_board: ChessBoardWidget,
    pub current_fen: Fen,
    // 用时相关
    pub red_time: std::time::Duration,
    pub black_time: std::time::Duration,
    pub last_move_time: Option<std::time::Instant>,
}

impl HumanView {
    pub fn new() -> Self {
        Self {
            model_path: "./models/model.ot".to_string(),
            player_color: "红方".to_string(),
            num_explores: 800,
            game_status: "未开始".to_string(),
            // chess_board: ChessBoardWidget::new(),  // 暂时注释掉
            chess_board: ChessBoardWidget::new().with_size(600.0, 667.0),
            // current_fen: Fen::init(),  // 暂时注释掉
            current_fen: Fen::new("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"),
            red_time: std::time::Duration::ZERO,
            black_time: std::time::Duration::ZERO,
            last_move_time: None,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // 不使用 ScrollArea，直接显示以获取更多空间
        ui.heading("👤 人机对弈");
        ui.add_space(5.0);
        
        // 配置 - 使用紧凑的 Grid
        egui::Grid::new("human_params")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("模型路径:");
                ui.text_edit_singleline(&mut self.model_path);
                ui.end_row();
                
                ui.label("玩家颜色:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.player_color, "红方".to_string(), "红方（先手）");
                    ui.radio_value(&mut self.player_color, "黑方".to_string(), "黑方（后手）");
                });
                ui.end_row();
                
                ui.label("AI 强度:");
                let explores = self.num_explores;
                ui.add(egui::Slider::new(&mut self.num_explores, 100..=3200)
                    .text(format!("{}", explores)));
                ui.end_row();
            });
        
        ui.add_space(5.0);
        
        // 开始按钮
        ui.horizontal(|ui| {
            if ui.button("▶️ 开始游戏").clicked() {
                self.game_status = "游戏进行中 - 红方先行".to_string();
                self.red_time = std::time::Duration::ZERO;
                self.black_time = std::time::Duration::ZERO;
                self.last_move_time = Some(std::time::Instant::now());
            }
            if ui.button("🔄 重新开始").clicked() {
                self.current_fen = Fen::new("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1");
                self.chess_board.clear_last_move();
                self.game_status = "游戏重新开始 - 红方先行".to_string();
                self.red_time = std::time::Duration::ZERO;
                self.black_time = std::time::Duration::ZERO;
                self.last_move_time = Some(std::time::Instant::now());
            }
            if ui.button("🆕 新游戏").clicked() {
                self.game_status = "游戏进行中".to_string();
            }
            if ui.button("↩️ 悔棋").clicked() {
                // TODO: 实现悔棋
            }
            if ui.button("💡 提示").clicked() {
                // TODO: 实现提示
            }
        });
        
        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);
        
        // 用时显示
        let current_time = if let Some(start) = self.last_move_time {
            start.elapsed()
        } else {
            std::time::Duration::ZERO
        };
        
        // 根据当前走棋方计算总用时
        let position = crate::pos::position::Position::from_fen(&self.current_fen);
        let is_red_turn = position.current_player() == crate::pos::ChessPlayer::Red;
        
        let red_display = if is_red_turn {
            self.red_time + current_time
        } else {
            self.red_time
        };
        
        let black_display = if !is_red_turn {
            self.black_time + current_time
        } else {
            self.black_time
        };
        
        // 格式化用时显示
        fn format_duration(d: std::time::Duration) -> String {
            let total_secs = d.as_secs();
            let hours = total_secs / 3600;
            let minutes = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            if hours > 0 {
                format!("{:02}:{:02}:{:02}", hours, minutes, secs)
            } else {
                format!("{:02}:{:02}", minutes, secs)
            }
        }
        
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("🔴 红方用时");
                ui.heading(format_duration(red_display));
            });
            ui.add_space(20.0);
            ui.group(|ui| {
                ui.label("⚫ 黑方用时");
                ui.heading(format_duration(black_display));
            });
        });
        
        ui.add_space(5.0);
        
        // 棋盘区域 - 直接显示，不使用嵌套布局
        ui.heading("棋盘");
        ui.add_space(5.0);
        
        // 计算期望的棋盘尺寸 - 使用固定比例
        let window_width = ui.available_width();
        // 缩小棋盘尺寸，使用窗口宽度的 50%，但不超过 600px
        let desired_width = (window_width * 0.5).max(450.0).min(600.0);
        let aspect_ratio = 502.0 / 452.0;
        let desired_height = desired_width * aspect_ratio;
        
        log::info!("Board size: {}x{} (window: {})", desired_width, desired_height, window_width);
        
        let ctx = ui.ctx().clone();
        
        // 生成当前局面的合法走法
        let position = crate::pos::position::Position::from_fen(&self.current_fen);
        let legal_moves = position.gen_legal_moves();
        self.chess_board.set_legal_moves(legal_moves);
        
        // 显示棋盘 - 使用固定尺寸
        if let Some(mv) = self.chess_board.show_with_size(ui, &ctx, &self.current_fen, desired_width, desired_height) {
            // 计算当前方的用时
            if let Some(start) = self.last_move_time {
                let elapsed = start.elapsed();
                let position = crate::pos::position::Position::from_fen(&self.current_fen);
                let is_red_turn = position.current_player() == crate::pos::ChessPlayer::Red;
                
                if is_red_turn {
                    self.red_time += elapsed;
                } else {
                    self.black_time += elapsed;
                }
            }
            
            self.game_status = format!("走法: {}", mv);
            // 记录上一步走子
            self.chess_board.set_last_move(mv);
            // 应用走法并更新局面
            let mut position = crate::pos::position::Position::from_fen(&self.current_fen);
            position.make_move(mv);
            self.current_fen = position.to_fen();
            // 重置计时器
            self.last_move_time = Some(std::time::Instant::now());
            // 清除选中状态，避免吃子后选中状态指向无效位置
            self.chess_board.clear_selection();
        }
        
        ui.add_space(10.0);
        
       
        
        // 临时调试信息
        let position = crate::pos::position::Position::from_fen(&self.current_fen);
        let legal_moves = position.gen_legal_moves();
        ui.label(format!("合法走法数: {}", legal_moves.len()));
        if let Some(selected) = self.chess_board.get_selected_square() {
            ui.label(format!("选中格子: 0x{:02X} ({})", selected, selected));
        }
    }
}

/// 棋谱浏览视图
pub struct PgnView {
    pub pgn_file: Option<PathBuf>,
    pub pgn_content: String,
    pub loaded_games: usize,
}

impl PgnView {
    pub fn new() -> Self {
        Self {
            pgn_file: None,
            pgn_content: "请选择 PGN 文件".to_string(),
            loaded_games: 0,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("📜 棋谱浏览");
        ui.add_space(10.0);
        
        // 文件选择
        ui.horizontal(|ui| {
            if ui.button("📂 选择 PGN 文件").clicked() {
                // TODO: 实现文件选择对话框
                self.load_sample_pgn();
            }
            
            if let Some(path) = &self.pgn_file {
                ui.label(format!("当前文件: {:?}", path.file_name().unwrap_or_default()));
            }
        });
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        
        // 统计信息
        ui.label(format!("已加载游戏数: {}", self.loaded_games));
        
        ui.add_space(10.0);
        
        // PGN 内容显示
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.code(&self.pgn_content);
            });
    }
    
    fn load_sample_pgn(&mut self) {
        // 示例 PGN 数据
        self.pgn_content = r#"[Event "示例游戏"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]

1. 炮二平五 马8进7
2. 马二进三 车9平8
3. 车一平二 马2进3
4. 兵七进一 卒7进1
5. 车二进六 炮8平9
6. 车二平三 炮9退1
7. 马八进七 士4进5
8. 炮八平九 炮9平7
9. 车三平四 马7进8
10. 车九平八 车1平2
1-0
"#.to_string();
        self.loaded_games = 1;
        self.pgn_file = Some(PathBuf::from("sample.pgn"));
    }
}
