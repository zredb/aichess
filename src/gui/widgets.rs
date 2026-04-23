/// 图形化棋盘组件
/// 使用现成的 board.svg 作为棋盘，支持棋子显示和交互点击

use eframe::egui;
use egui::{ColorImage, TextureHandle, TextureOptions};
use resvg::tiny_skia;
use resvg::usvg;

use crate::fen::Fen;
use crate::pos::position::Position;
use crate::pos::moves::Move;
use crate::pos::{file_x, rank_y};

/// 棋盘模板 SVG 内容（嵌入在代码中）
const BOARD_SVG: &str = include_str!("../bin/board.svg");

/// 棋子字符映射表：(大写字母, 红方名称, 黑方名称)
const PIECE_CHARS: [(char, &str, &str); 7] = [
    ('R', "車", "车"),
    ('N', "馬", "马"),
    ('B', "相", "象"),
    ('A', "仕", "士"),
    ('K', "帥", "将"),
    ('C', "炮", "炮"),
    ('P', "兵", "卒"),
];

/// 棋盘小部件
pub struct ChessBoardWidget {
    /// 当前显示的 FEN
    current_fen: Option<Fen>,
    /// 缓存的位置信息
    cached_pieces: Vec<(char, u8)>,
    /// 棋盘纹理（从 SVG 渲染）
    board_texture: Option<TextureHandle>,
    /// 棋盘尺寸（像素）
    board_size: [f32; 2],
    /// 格子尺寸
    cell_size: f32,
    /// 偏移量
    offset: [f32; 2],
    /// 选中的格子（如果有）
    selected_square: Option<usize>,
    /// 上一步走子（起点和终点）
    last_move: Option<Move>,
    /// 合法走法列表
    legal_moves: Vec<Move>,
    /// 是否显示合法走法提示
    #[allow(dead_code)]
    show_legal_hints: bool,
}

impl ChessBoardWidget {
    pub fn new() -> Self {
        Self {
            current_fen: None,
            cached_pieces: Vec::new(),
            board_texture: None,
            board_size: [452.0, 502.0],
            cell_size: 50.0,
            offset: [0.0, 0.0],
            selected_square: None,
            last_move: None,
            legal_moves: Vec::new(),
            show_legal_hints: true,
        }
    }

    /// 设置棋盘大小
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.board_size = [width, height];
        self.cell_size = width / 9.0;
        self
    }

    /// 清除选中状态
    pub fn clear_selection(&mut self) {
        self.selected_square = None;
    }

    /// 获取当前选中的格子
    pub fn get_selected_square(&self) -> Option<usize> {
        self.selected_square
    }

    /// 显示棋盘，使用指定的尺寸
    pub fn show_with_size(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, fen: &Fen, width: f32, height: f32) -> Option<Move> {
        // 使用传入的固定尺寸
        let final_width = width;
        let final_height = height;
        
        log::debug!("Using fixed board size: {}x{}", final_width, final_height);
        
        // 检查是否需要重新渲染（尺寸变化、FEN 变化或选中状态变化）
        let size_changed = (self.board_size[0] - final_width).abs() > 1.0 || 
                          (self.board_size[1] - final_height).abs() > 1.0;
        
        let fen_changed = match &self.current_fen {
            None => true,
            Some(cached) => cached.fen_str() != fen.fen_str(),
        };
        
        let needs_update = size_changed || fen_changed;

        if needs_update {
            self.board_size = [final_width, final_height];
            self.cell_size = final_width / 9.0;
            
            // 更新棋子缓存
            let position = Position::from_fen(fen);
            self.cached_pieces = position.piece_loc();
            self.current_fen = Some(fen.clone());
            
            // 清除旧纹理,强制重新创建
            self.board_texture = None;
            
            // 重新渲染棋盘纹理
            self.render_board_texture(ctx);
            
            if size_changed {
                log::info!("Board resized to {}x{}", final_width, final_height);
            }
        }
        
        // 分配棋盘区域 - 使用指定尺寸
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(final_width, final_height),
            egui::Sense::click_and_drag(),
        );
        
        log::debug!("Allocated rect: {:?}, texture exists: {}", rect, self.board_texture.is_some());
        
        // 绘制一个测试用的边框，确认区域可点击
        let stroke = egui::Stroke::new(3.0, egui::Color32::RED);
        ui.painter().line_segment([rect.left_top(), rect.right_top()], stroke);
        ui.painter().line_segment([rect.right_top(), rect.right_bottom()], stroke);
        ui.painter().line_segment([rect.right_bottom(), rect.left_bottom()], stroke);
        ui.painter().line_segment([rect.left_bottom(), rect.left_top()], stroke);

        // 绘制棋盘背景 - 使用完整的 rect 来绘制图像，确保填充整个区域
        if let Some(ref texture) = self.board_texture {
            // 使用完整的 rect 绘制图像，确保棋盘占满整个分配的区域
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            log::debug!("Drew board image in rect: {:?}", rect);
        } else {
            // 如果纹理未加载，显示占位符
            log::warn!("Board texture not available, showing placeholder");
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(238, 176, 102));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Loading...",
                egui::FontId::proportional(20.0),
                egui::Color32::BLACK,
            );
        }

        // 处理点击事件
        if response.clicked() {
            log::info!("=== CLICK DETECTED ===");
            if let Some(pos) = response.interact_pointer_pos() {
                log::debug!("Click at position: ({}, {})", pos.x, pos.y);
                log::debug!("Rect: min=({}, {}), max=({}, {})", rect.min.x, rect.min.y, rect.max.x, rect.max.y);
                
                // 将点击位置转换为棋盘坐标（考虑缩放）
                let local_x = pos.x - rect.min.x;
                let local_y = pos.y - rect.min.y;
                
                log::debug!("Local position: ({}, {})", local_x, local_y);
                
                // SVG 原始尺寸是 452x502，左上角从 (25,25) 开始
                // 计算缩放比例
                let scale_x = self.board_size[0] / 452.0;
                let scale_y = self.board_size[1] / 502.0;
                
                log::debug!("Board size: {}x{}, Scale: {}x{}", self.board_size[0], self.board_size[1], scale_x, scale_y);
                
                // 转换到原始 SVG 坐标系
                let svg_x = local_x / scale_x;
                let svg_y = local_y / scale_y;
                
                log::debug!("SVG coordinates: ({}, {})", svg_x, svg_y);
                
                // 计算行列（SVG 中每格 50px，从 25px 开始）
                let col = ((svg_x - 25.0) / 50.0).round() as i32;
                let row = ((svg_y - 25.0) / 50.0).round() as i32;

                log::debug!("Grid position: col={}, row={}", col, row);

                if col >= 0 && col < 9 && row >= 0 && row < 10 {
                    // 转换为内部坐标
                    let sq = ((row + 3) << 4) | (col + 3);
                    log::debug!("Square: 0x{:02X} ({})", sq, sq);
                    
                    // 处理点击
                    let result = self.handle_click(sq as usize);
                    
                    // 如果选中状态改变，需要重新渲染以显示绿色圆圈
                    self.board_texture = None;
                    self.render_board_texture(ctx);
                    
                    // 返回走法（如果有）
                    return result;
                } else {
                    log::debug!("Click outside board");
                }
            }
        }

        None
    }

    /// 显示棋盘（使用现成的 board.svg）
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, fen: &Fen) -> Option<Move> {
        // 获取可用空间
        let available_size = ui.available_size();
        
        log::debug!("Available size: {:?}", available_size);
        
        // 根据可用空间计算合适的棋盘大小（保持宽高比 452:502）
        let aspect_ratio = 452.0 / 502.0;
        
        // 使用预设的 board_size，如果可用空间更大则使用可用空间
        let mut final_width = self.board_size[0];
        let mut final_height = self.board_size[1];
        
        // 如果可用空间比预设尺寸大，使用可用空间
        if available_size.x > final_width && available_size.y > final_height {
            final_width = available_size.x;
            final_height = available_size.y;
        }
        
        // 保持宽高比
        if final_width / final_height > aspect_ratio {
            final_width = final_height * aspect_ratio;
        } else {
            final_height = final_width / aspect_ratio;
        }
        
        // 确保最小尺寸
        final_width = final_width.max(500.0);
        final_height = final_height.max(557.0); // 500 * 502/452
        
        log::debug!("Calculated board size: {}x{} (from available {:?})", final_width, final_height, available_size);
        
        // 检查是否需要重新渲染（尺寸变化或 FEN 变化）
        let size_changed = (self.board_size[0] - final_width).abs() > 1.0 || 
                          (self.board_size[1] - final_height).abs() > 1.0;
        
        let fen_changed = match &self.current_fen {
            None => true,
            Some(cached) => cached.fen_str() != fen.fen_str(),
        };
        
        let needs_update = size_changed || fen_changed;

        if needs_update {
            // 更新棋盘尺寸
            self.board_size = [final_width, final_height];
            self.cell_size = final_width / 9.0;
            
            let position = Position::from_fen(fen);
            self.cached_pieces = position.piece_loc();
            self.current_fen = Some(fen.clone());
            
            // 清除旧纹理,强制重新创建
            self.board_texture = None;
            
            // 重新渲染棋盘纹理
            self.render_board_texture(ctx);
            
            if size_changed {
                log::info!("Board resized to {}x{}", final_width, final_height);
            }
        }

        // 分配棋盘区域 - 使用当前计算的尺寸
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(final_width, final_height),
            egui::Sense::click(),
        );
        
        log::debug!("Allocated rect: {:?}, texture exists: {}", rect, self.board_texture.is_some());
        
        // 绘制一个测试用的边框，确认区域可点击
        let stroke = egui::Stroke::new(3.0, egui::Color32::RED);
        ui.painter().line_segment([rect.left_top(), rect.right_top()], stroke);
        ui.painter().line_segment([rect.right_top(), rect.right_bottom()], stroke);
        ui.painter().line_segment([rect.right_bottom(), rect.left_bottom()], stroke);
        ui.painter().line_segment([rect.left_bottom(), rect.left_top()], stroke);

        // 绘制棋盘背景
        if let Some(ref texture) = self.board_texture {
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            log::debug!("Drew board image");
        } else {
            // 如果纹理未加载，显示占位符
            log::warn!("Board texture not available, showing placeholder");
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(238, 176, 102));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Loading...",
                egui::FontId::proportional(20.0),
                egui::Color32::BLACK,
            );
        }

        // 处理点击事件
        if response.clicked() {
            log::info!("=== CLICK DETECTED ===");
            if let Some(pos) = response.interact_pointer_pos() {
                log::debug!("Click at position: ({}, {})", pos.x, pos.y);
                log::debug!("Rect: min=({}, {}), max=({}, {})", rect.min.x, rect.min.y, rect.max.x, rect.max.y);
                
                // 将点击位置转换为棋盘坐标（考虑缩放）
                let local_x = pos.x - rect.min.x;
                let local_y = pos.y - rect.min.y;
                
                log::debug!("Local position: ({}, {})", local_x, local_y);
                
                // SVG 原始尺寸是 452x502，左上角从 (25,25) 开始
                // 计算缩放比例
                let scale_x = self.board_size[0] / 452.0;
                let scale_y = self.board_size[1] / 502.0;
                
                log::debug!("Board size: {}x{}, Scale: {}x{}", self.board_size[0], self.board_size[1], scale_x, scale_y);
                
                // 转换到原始 SVG 坐标系
                let svg_x = local_x / scale_x;
                let svg_y = local_y / scale_y;
                
                log::debug!("SVG coordinates: ({}, {})", svg_x, svg_y);
                
                // 计算行列（SVG 中每格 50px，从 25px 开始）
                let col = ((svg_x - 25.0) / 50.0).round() as i32;
                let row = ((svg_y - 25.0) / 50.0).round() as i32;

                log::debug!("Grid position: col={}, row={}", col, row);

                if col >= 0 && col < 9 && row >= 0 && row < 10 {
                    // 转换为内部坐标
                    let sq = ((row + 3) << 4) | (col + 3);
                    log::debug!("Square: 0x{:02X} ({})", sq, sq);
                    return self.handle_click(sq as usize);
                } else {
                    log::debug!("Click outside board");
                }
            }
        }

        None
    }

    /// 渲染棋盘（使用 board.svg 动态生成棋子）
    fn render_board_texture(&mut self, ctx: &egui::Context) {
        // 根据 FEN 动态生成 SVG
        let svg_content = self.generate_svg_with_pieces();
        
        // 使用 resvg 渲染 SVG，配置字体
        let mut opt = usvg::Options::default();
        
        // 尝试加载系统字体
        #[cfg(target_os = "windows")]
        {
            // Windows 上尝试加载宋体
            if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/simsun.ttc") {
                opt.fontdb_mut().load_font_data(font_data);
                log::info!("✓ Loaded SimSun font for SVG rendering");
            }
            // 也尝试加载微软雅黑
            if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/msyh.ttc") {
                opt.fontdb_mut().load_font_data(font_data);
                log::info!("✓ Loaded Microsoft YaHei font for SVG rendering");
            }
        }
        
        if let Ok(tree) = usvg::Tree::from_str(&svg_content, &opt) {
            let width = self.board_size[0] as u32;
            let height = self.board_size[1] as u32;
            
            log::info!("Rendering board texture: {}x{}", width, height);
            log::info!("SVG original size: {}x{}", tree.size().width(), tree.size().height());
            
            if let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) {
                // 计算缩放比例，让 SVG 填充整个目标区域
                let scale_x = width as f32 / tree.size().width();
                let scale_y = height as f32 / tree.size().height();
                let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
                
                log::info!("Transform scale: {}x{}", scale_x, scale_y);
                
                resvg::render(&tree, transform, &mut pixmap.as_mut());

                // 转换为 egui ColorImage
                let pixels = pixmap.pixels();
                let mut image_data = vec![0u8; pixels.len() * 4];
                for (i, pixel) in pixels.iter().enumerate() {
                    image_data[i * 4] = pixel.red();
                    image_data[i * 4 + 1] = pixel.green();
                    image_data[i * 4 + 2] = pixel.blue();
                    image_data[i * 4 + 3] = pixel.alpha();
                }

                let color_image = ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &image_data,
                );
                
                self.board_texture = Some(ctx.load_texture(
                    "chess_board",
                    color_image,
                    TextureOptions::default(),
                ));
                
                log::info!("✓ Board texture rendered from SVG");
            } else {
                log::error!("Failed to create pixmap");
            }
        } else {
            log::error!("Failed to parse SVG");
        }
    }
    
    /// 根据 FEN 生成带棋子的 SVG
    fn generate_svg_with_pieces(&self) -> String {
        let mut svg = String::from(BOARD_SVG);
        
        // 找到 <g id="game"> 标签后面的注释位置
        if let Some(insert_pos) = svg.find("<!--        <use href=\"#r\"") {
            // 在注释前插入棋子
            let mut pieces_svg = String::new();
            
            log::info!("Generating {} pieces", self.cached_pieces.len());
            
            for (piece_char, sq) in &self.cached_pieces {
                if *sq > 0 {
                    let px = file_x(*sq as usize);
                    let py = rank_y(*sq as usize);
                    
                    // SVG 坐标：x = 25 + (px-3)*50, y = 25 + (py-3)*50
                    let x = 25.0 + ((px - 3) as f32) * 50.0;
                    let y = 25.0 + ((py - 3) as f32) * 50.0;
                    
                    // 根据棋子类型选择对应的模板
                    let template = match *piece_char {
                        'r' => "r", 'n' => "n", 'b' => "b", 'a' => "a",
                        'k' => "k", 'c' => "c", 'p' => "p",
                        'R' => "R", 'N' => "N", 'B' => "B", 'A' => "A",
                        'K' => "K", 'C' => "C", 'P' => "P",
                        _ => continue,
                    };
                    
                    pieces_svg.push_str(&format!(
                        "        <use href=\"#{}\" transform=\"translate({:.1},{:.1})\" />\n",
                        template, x, y
                    ));
                    
                    log::debug!("Piece {} at ({}, {}) -> SVG ({:.1}, {:.1})", piece_char, px, py, x, y);
                }
            }
            
            // 添加选中状态高亮
            if let Some(selected_sq) = self.selected_square {
                let px = file_x(selected_sq);
                let py = rank_y(selected_sq);
                let x = 25.0 + ((px - 3) as f32) * 50.0;
                let y = 25.0 + ((py - 3) as f32) * 50.0;
                
                // 绘制选中高亮圆圈（半透明绿色）
                pieces_svg.push_str(&format!(
                    "        <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"24\" fill=\"none\" stroke=\"#00FF00\" stroke-width=\"3\" opacity=\"0.8\" />\n",
                    x, y
                ));
            }
            
            // 添加上一步走子标记
            if let Some(last_mv) = self.last_move {
                // 只在起点标记（白色小圆点）
                let from_px = file_x(last_mv.from);
                let from_py = rank_y(last_mv.from);
                let from_x = 25.0 + ((from_px - 3) as f32) * 50.0;
                let from_y = 25.0 + ((from_py - 3) as f32) * 50.0;
                
                pieces_svg.push_str(&format!(
                    "        <circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"5\" fill=\"#FFFFFF\" opacity=\"0.8\" />\n",
                    from_x, from_y
                ));
            }
            
            svg.insert_str(insert_pos, &pieces_svg);
            log::info!("Generated SVG with {} pieces", pieces_svg.lines().count());
        } else {
            log::error!("Could not find insertion point in SVG template");
        }
        
        svg
    }

    /// 从 Position 创建 FEN 并显示
    pub fn show_from_position(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, position: &Position) -> Option<Move> {
        let fen = position.to_fen();
        self.show(ui, ctx, &fen)
    }

    /// 设置合法走法
    pub fn set_legal_moves(&mut self, moves: Vec<Move>) {
        log::debug!("Setting {} legal moves", moves.len());
        // for mv in &moves {
        //     log::debug!("  Move: {:?}", mv);
        // }
        self.legal_moves = moves;
    }

    /// 设置上一步走子
    pub fn set_last_move(&mut self, mv: Move) {
        self.last_move = Some(mv);
    }

    /// 清除上一步走子标记
    pub fn clear_last_move(&mut self) {
        self.last_move = None;
    }

    /// 处理点击事件
    fn handle_click(&mut self, sq: usize) -> Option<Move> {
        // 安全检查：确保 current_fen 存在
        let fen = match &self.current_fen {
            Some(f) => f,
            None => {
                log::warn!("No FEN available, ignoring click");
                return None;
            }
        };
        
        // 获取当前位置的棋子信息
        let position = Position::from_fen(fen);
        let piece_locs = position.piece_loc();
        
        log::debug!("Click at square: {} (0x{:02X})", sq, sq);
        log::debug!("Total pieces: {}", piece_locs.len());
        
        // 检查点击的位置是否有棋子
        let clicked_piece = piece_locs.iter().find(|(_, s)| *s as usize == sq);
        
        if let Some((piece_char, _)) = clicked_piece {
            log::debug!("Clicked on piece: {}", piece_char);
        } else {
            log::debug!("Clicked on empty square");
        }
        
        if let Some(selected) = self.selected_square {
            // 已经有选中的棋子
            log::debug!("Selected square: {}, clicking: {}", selected, sq);
            
            if selected == sq {
                // 点击的是同一个位置，取消选择
                log::debug!("Deselecting square");
                self.selected_square = None;
                return None;
            }
            
            // 检查是否是合法走法
            if let Some(mv) = self.legal_moves.iter().find(|m| m.from == selected && m.to as usize == sq) {
                log::debug!("Valid move found: {:?}", mv);
                self.selected_square = None;
                return Some(*mv);
            }
            
            log::debug!("Not a valid move, checking if it's own piece");
            
            // 如果点击的是己方棋子，则重新选择
            if let Some((piece_char, _)) = clicked_piece {
                // 通过 FEN 字符串判断当前走棋方
                let fen_str = fen.fen_str();
                let is_red_turn = fen_str.contains(" w "); // "w" 表示红方走棋
                let is_red_piece = piece_char.is_ascii_uppercase();
                
                log::debug!("Is red turn: {}, is red piece: {}", is_red_turn, is_red_piece);
                
                if is_red_turn == is_red_piece {
                    log::debug!("Selecting new piece");
                    self.selected_square = Some(sq);
                    return None;
                }
            }
            
            // 点击的不是合法走法也不是己方棋子，取消选择
            log::debug!("Invalid click, deselecting");
            self.selected_square = None;
        } else {
            // 没有选中的棋子，如果点击的是己方棋子则选择
            if let Some((piece_char, _)) = clicked_piece {
                // 通过 FEN 字符串判断当前走棋方
                let fen_str = fen.fen_str();
                let is_red_turn = fen_str.contains(" w "); // "w" 表示红方走棋
                let is_red_piece = piece_char.is_ascii_uppercase();
                
                log::debug!("First click - Is red turn: {}, is red piece: {}", is_red_turn, is_red_piece);
                
                if is_red_turn == is_red_piece {
                    log::debug!("Selecting piece: {}", piece_char);
                    self.selected_square = Some(sq);
                } else {
                    log::debug!("Not your piece to move");
                }
            }
        }
        None
    }



    /// 导出当前棋盘为 SVG 文件
    pub fn export_svg(&self, fen: &Fen, file_path: &str) -> std::io::Result<()> {
        let svg_content = self.generate_board_svg(fen);
        std::fs::write(file_path, svg_content)
    }
    fn generate_board_svg(&self, fen: &Fen) -> String {
        let position = Position::from_fen(fen);
        let piece_locs = position.piece_loc();

        let mut svg = String::new();
        
        // SVG 头部
        svg.push_str("<svg width=\"");
        svg.push_str(&(self.board_size[0] + 4.0).to_string());
        svg.push_str("\" height=\"");
        svg.push_str(&(self.board_size[1] + 4.0).to_string());
        svg.push_str("\" xmlns=\"http://www.w3.org/2000/svg\">\n");

        // 定义部分 - 使用写死字符串避免 format! 解析 #
        svg.push_str("<defs>\n");
        svg.push_str("  <filter id=\"shadow\" x=\"-20%\" y=\"-20%\" width=\"140%\" height=\"140%\">\n");
        svg.push_str("    <feDropShadow dx=\"2\" dy=\"2\" stdDeviation=\"2\" flood-color=\"#000\" flood-opacity=\"0.3\"/>\n");
        svg.push_str("  </filter>\n");
        svg.push_str("</defs>\n");
        
        // 棋盘背景
        svg.push_str("<g id=\"board\">\n");
        svg.push_str("  <rect x=\"2\" y=\"2\" width=\"");
        svg.push_str(&self.board_size[0].to_string());
        svg.push_str("\" height=\"");
        svg.push_str(&self.board_size[1].to_string());
        svg.push_str("\" fill=\"#eeb066\" rx=\"3\"/>\n");
        svg.push_str("  <rect x=\"27\" y=\"27\" width=\"400\" height=\"450\" fill=\"none\" stroke=\"#000\" stroke-width=\"2\"/>\n");

        // 绘制横线
        for i in 0..10 {
            let y = 27.0 + i as f32 * self.cell_size;
            svg.push_str("        <line x1=\"27\" y1=\"");
            svg.push_str(&format!("{:.1}", y));
            svg.push_str("\" x2=\"427\" y2=\"");
            svg.push_str(&format!("{:.1}", y));
            svg.push_str("\" stroke=\"#000\" stroke-width=\"1\"/>\n");
        }

        // 绘制竖线
        for i in 0..9 {
            let x = 27.0 + i as f32 * self.cell_size;
            if i == 0 || i == 8 {
                svg.push_str("        <line x1=\"");
                svg.push_str(&format!("{:.1}", x));
                svg.push_str("\" y1=\"27\" x2=\"");
                svg.push_str(&format!("{:.1}", x));
                svg.push_str("\" y2=\"477\" stroke=\"#000\" stroke-width=\"2\"/>\n");
            } else {
                svg.push_str("        <line x1=\"");
                svg.push_str(&format!("{:.1}", x));
                svg.push_str("\" y1=\"27\" x2=\"");
                svg.push_str(&format!("{:.1}", x));
                svg.push_str("\" y2=\"227\" stroke=\"#000\" stroke-width=\"1\"/>\n");
                svg.push_str("        <line x1=\"");
                svg.push_str(&format!("{:.1}", x));
                svg.push_str("\" y1=\"250\" x2=\"");
                svg.push_str(&format!("{:.1}", x));
                svg.push_str("\" y2=\"477\" stroke=\"#000\" stroke-width=\"1\"/>\n");
            }
        }

        // 九宫格斜线
        svg.push_str("        <line x1=\"177\" y1=\"27\" x2=\"277\" y2=\"127\" stroke=\"#000\" stroke-width=\"1\"/>\n");
        svg.push_str("        <line x1=\"277\" y1=\"27\" x2=\"177\" y2=\"127\" stroke=\"#000\" stroke-width=\"1\"/>\n");
        svg.push_str("        <line x1=\"177\" y1=\"377\" x2=\"277\" y2=\"477\" stroke=\"#000\" stroke-width=\"1\"/>\n");
        svg.push_str("        <line x1=\"277\" y1=\"377\" x2=\"177\" y2=\"477\" stroke=\"#000\" stroke-width=\"1\"/>\n");

        // 楚河汉界文字
        svg.push_str("        <text x=\"110\" y=\"258\" font-family=\"楷体, KaiTi, serif\" font-size=\"28\" fill=\"#000\">楚 河</text>\n");
        svg.push_str("        <text x=\"260\" y=\"258\" font-family=\"楷体, KaiTi, serif\" font-size=\"28\" fill=\"#000\">汉 界</text>\n");

        // 绘制炮位和兵位标记
        self.draw_position_marks(&mut svg);

        svg.push_str("</g>");

        // 绘制棋子
        for (piece_char, sq) in &piece_locs {
            if *sq > 0 {
                let x = file_x(*sq as usize);
                let y = rank_y(*sq as usize);
                let pixel_x = 27.0 + (x - 3) as f32 * self.cell_size;
                let pixel_y = 27.0 + (y - 3) as f32 * self.cell_size;

                let is_red = piece_char.is_ascii_uppercase();
                let (fill, stroke, text_color, piece_name) = if is_red {
                    ("#fff", "#c00", "#c00", self.get_piece_name(*piece_char, true))
                } else {
                    ("#fff", "#000", "#000", self.get_piece_name(*piece_char, false))
                };

                // 检查是否是选中的棋子
                let is_selected = self.selected_square.map_or(false, |s| s == *sq as usize);
                let stroke_width = if is_selected { 4.0 } else { 2.0 };
                let radius = if is_selected { 23.0 } else { 21.0 };

                svg.push_str("<g filter=\"url(#shadow)\">\n");
                svg.push_str("  <circle cx=\"");
                svg.push_str(&format!("{:.1}", pixel_x));
                svg.push_str("\" cy=\"");
                svg.push_str(&format!("{:.1}", pixel_y));
                svg.push_str("\" r=\"");
                svg.push_str(&format!("{:.1}", radius));
                svg.push_str("\" fill=\"");
                svg.push_str(fill);
                svg.push_str("\" stroke=\"");
                svg.push_str(stroke);
                svg.push_str("\" stroke-width=\"");
                svg.push_str(&format!("{:.1}", stroke_width));
                svg.push_str("\"/>\n");
                svg.push_str("  <text x=\"");
                svg.push_str(&format!("{:.1}", pixel_x));
                svg.push_str("\" y=\"");
                svg.push_str(&format!("{:.1}", pixel_y + 1.0));
                svg.push_str("\" font-family=\"楷体, KaiTi, serif\" font-size=\"28\" font-weight=\"bold\" fill=\"");
                svg.push_str(text_color);
                svg.push_str("\" text-anchor=\"middle\" dominant-baseline=\"central\">");
                svg.push_str(piece_name);
                svg.push_str("</text>\n");
                svg.push_str("</g>\n");
            }
        }

        svg.push_str("</svg>");
        svg
    }

    /// 绘制炮位和兵位标记
    fn draw_position_marks(&self, svg: &mut String) {
        let positions = vec![
            // 炮位
            (2, 7), (2, 2), (7, 7), (7, 2),
            // 兵位
            (1, 6), (1, 4), (1, 2), (1, 0), (1, 8),
            (8, 6), (8, 4), (8, 2), (8, 0), (8, 8),
        ];

        for (rank, file) in positions {
            let x = 27.0 + file as f32 * self.cell_size;
            let y = 27.0 + rank as f32 * self.cell_size;
            let size = 8.0;

            // 左上角
            if file > 0 {
                svg.push_str("<polyline points=\"");
                svg.push_str(&format!("{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                    x - size, y - size / 2.0, x - size, y - size, x - size / 2.0, y - size));
                svg.push_str("\" fill=\"none\" stroke=\"#000\" stroke-width=\"1\"/>\n");
            }
            // 右上角
            if file < 8 {
                svg.push_str("<polyline points=\"");
                svg.push_str(&format!("{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                    x + size, y - size / 2.0, x + size, y - size, x + size / 2.0, y - size));
                svg.push_str("\" fill=\"none\" stroke=\"#000\" stroke-width=\"1\"/>\n");
            }
            // 左下角
            if file > 0 {
                svg.push_str("<polyline points=\"");
                svg.push_str(&format!("{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                    x - size, y + size / 2.0, x - size, y + size, x - size / 2.0, y + size));
                svg.push_str("\" fill=\"none\" stroke=\"#000\" stroke-width=\"1\"/>\n");
            }
            // 右下角
            if file < 8 {
                svg.push_str("<polyline points=\"");
                svg.push_str(&format!("{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                    x + size, y + size / 2.0, x + size, y + size, x + size / 2.0, y + size));
                svg.push_str("\" fill=\"none\" stroke=\"#000\" stroke-width=\"1\"/>\n");
            }
        }
    }

    /// 获取棋子名称
    fn get_piece_name(&self, piece_char: char, is_red: bool) -> &'static str {
        for (upper, red_name, black_name) in PIECE_CHARS {
            if piece_char.to_ascii_uppercase() == upper {
                return if is_red { red_name } else { black_name };
            }
        }
        "?"
    }
}
