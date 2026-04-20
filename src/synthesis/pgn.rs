use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

/// PGN (Portable Game Notation) 记录结构
#[derive(Debug, Clone)]
pub struct PgnGame {
    pub headers: Vec<(String, String)>,
    pub moves: Vec<String>,
    pub result: Option<String>,
}

impl PgnGame {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            moves: Vec::new(),
            result: None,
        }
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        if let Some(existing) = self.headers.iter_mut().find(|(k, _)| k == key) {
            existing.1 = value.to_string();
        } else {
            self.headers.push((key.to_string(), value.to_string()));
        }
    }

    pub fn get_header(&self, key: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn add_move(&mut self, move_str: &str) {
        self.moves.push(move_str.to_string());
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();

        // 写入头部信息
        for (key, value) in &self.headers {
            output.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        output.push('\n');

        // 写入走法
        let mut move_text = String::new();
        for (i, mv) in self.moves.iter().enumerate() {
            if i % 2 == 0 {
                if !move_text.is_empty() {
                    move_text.push(' ');
                }
                move_text.push_str(&format!("{}. {}", i / 2 + 1, mv));
            } else {
                move_text.push_str(&format!(" {}", mv));
            }
        }

        if let Some(result) = &self.result {
            move_text.push_str(&format!(" {}", result));
        }

        output.push_str(&move_text);
        output
    }
}

/// 从文件加载 PGN 游戏
pub fn load_pgn(path: &Path) -> Result<Vec<PgnGame>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("无法读取 PGN 文件: {:?}", path))?;

    parse_pgn(&content)
}

/// 解析 PGN 字符串
pub fn parse_pgn(content: &str) -> Result<Vec<PgnGame>> {
    let mut games = Vec::new();
    let mut current_game = PgnGame::new();
    let mut in_moves = false;

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() {
            if in_moves && !current_game.moves.is_empty() {
                games.push(current_game.clone());
                current_game = PgnGame::new();
                in_moves = false;
            }
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            // 解析头部信息
            if let Some(header) = parse_pgn_header(line) {
                current_game.set_header(&header.0, &header.1);
                in_moves = false;
            }
        } else {
            // 解析走法
            in_moves = true;
            parse_pgn_moves(&mut current_game, line)?;
        }
    }

    // 添加最后一个游戏
    if !current_game.moves.is_empty() || !current_game.headers.is_empty() {
        games.push(current_game);
    }

    Ok(games)
}

/// 保存游戏为 PGN 格式
pub fn save_pgn(games: &[PgnGame], path: &Path) -> Result<()> {
    let mut output = String::new();
    for game in games {
        output.push_str(&game.to_string());
        output.push_str("\n\n");
    }

    fs::write(path, output).with_context(|| format!("无法写入 PGN 文件: {:?}", path))?;
    Ok(())
}

/// 添加单个游戏结果到 PGN 文件
pub fn append_game_to_pgn(game: &PgnGame, path: &Path) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("无法打开 PGN 文件: {:?}", path))?;

    writeln!(file, "{}", game.to_string())?;
    writeln!(file)?;

    Ok(())
}

fn parse_pgn_header(line: &str) -> Option<(String, String)> {
    if !line.starts_with('[') || !line.ends_with(']') {
        return None;
    }

    let content = &line[1..line.len() - 1];
    let parts: Vec<&str> = content.splitn(2, '"').collect();

    if parts.len() == 2 {
        let key = parts[0].trim().to_string();
        let value = parts[1].trim_end_matches('"').trim().to_string();
        Some((key, value))
    } else {
        None
    }
}

fn parse_pgn_moves(game: &mut PgnGame, line: &str) -> Result<()> {
    // 移除注释
    let line = remove_comments(line);

    // 分割成 tokens
    let tokens: Vec<&str> = line.split_whitespace().collect();

    for token in tokens {
        if token == "1-0" || token == "0-1" || token == "1/2-1/2" || token == "*" {
            game.result = Some(token.to_string());
        } else if token.contains('.') {
            // 跳过回合编号
            continue;
        } else if !token.is_empty() {
            game.add_move(token);
        }
    }

    Ok(())
}

fn remove_comments(line: &str) -> String {
    let mut result = String::new();
    let mut in_brace = false;
    let mut in_semicolon = false;

    for ch in line.chars() {
        if in_brace {
            if ch == '}' {
                in_brace = false;
            }
        } else if in_semicolon {
            if ch == '\n' {
                in_semicolon = false;
            }
        } else {
            match ch {
                '{' => in_brace = true,
                ';' => in_semicolon = true,
                _ => result.push(ch),
            }
        }
    }

    result
}

/// 将中国象棋走法转换为 PGN 格式
pub fn move_to_pgn(move_str: &str, _turn: usize) -> String {
    // 简化实现：直接使用走法字符串
    // 实际应用中可能需要更复杂的转换逻辑
    move_str.to_string()
}

/// 从 PGN 格式解析为中国象棋走法
pub fn pgn_to_move(pgn_move: &str) -> Result<String> {
    // 简化实现：直接返回走法字符串
    // 实际应用中可能需要解析和验证
    Ok(pgn_move.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pgn_header() {
        let header = parse_pgn_header("[Event \"Test Game\"]");
        assert_eq!(header, Some(("Event".to_string(), "Test Game".to_string())));
    }

    #[test]
    fn test_pgn_game_to_string() {
        let mut game = PgnGame::new();
        game.set_header("Event", "Test");
        game.set_header("White", "Player1");
        game.set_header("Black", "Player2");
        game.add_move("炮二平五");
        game.add_move("马8进7");
        game.result = Some("1-0".to_string());

        let output = game.to_string();
        assert!(output.contains("[Event \"Test\"]"));
        assert!(output.contains("1. 炮二平五 马8进7"));
        assert!(output.contains("1-0"));
    }
}
