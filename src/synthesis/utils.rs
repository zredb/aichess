use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn train_dir(root: &str, tag: &str) -> std::io::Result<PathBuf> {
    let time = chrono::Local::now().format("%m-%d-%YT%H-%M-%SZ").to_string();
    Ok(Path::new(root).join(tag).join(time))
}

pub fn save_str(path: &PathBuf, filename: &str, value: &str) -> std::io::Result<()> {
    std::fs::File::create(&path.join(filename)).and_then(|mut f| f.write_all(value.as_bytes()))
}

pub fn git_hash() -> std::io::Result<String> {
    Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .and_then(|output| {
            Ok(String::from_utf8(output.stdout).expect("Command didn't produce valid utf-8"))
        })
}

pub fn git_diff() -> std::io::Result<String> {
    Command::new("git").arg("diff").output().and_then(|output| {
        Ok(String::from_utf8(output.stdout).expect("Command didn't produce valid utf-8"))
    })
}
/// 向PGN文件中添加对局结果信息。具体功能如下：
/// 写入白方和黑方的名字。
/// 根据 white_reward 判断比赛结果（白胜、黑胜或平局）并写入结果标签和最终结果。
pub fn add_pgn_result(
    pgn: &mut File,
    white_name: &String,
    black_name: &String,
    white_reward: f32,
) -> std::io::Result<()> {
    write!(pgn, "[White \"{}\"]\n", white_name)?;
    write!(pgn, "[Black \"{}\"]\n", black_name)?;
    let result = if white_reward == 1.0 {
        // white wins
        "1-0"
    } else if white_reward == -1.0 {
        // black wins
        "0-1"
    } else {
        assert_eq!(white_reward, 0.0);
        // draw
        "1/2-1/2"
    };
    write!(pgn, "[Result \"{}\"]\n", result)?;
    write!(pgn, "{}\n", result)
}
///bayeselo.exe 是一个用于处理和分析 Elo 评级系统的命令行工具。Elo 评级系统常用于计算棋类游戏（如国际象棋）中玩家的相对技能水平。具体来说，bayeselo 工具可以帮助执行以下任务：
// 计算比赛结果的概率。
// 根据比赛结果调整玩家的 Elo 评分。
// 分析比赛数据并生成统计报告。
pub fn calculate_ratings(dir: &PathBuf) -> std::io::Result<()> {
    let mut child = Command::new("bayeselo.exe")
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    write!(stdin, "readpgn results.pgn\n")?;
    write!(stdin, "elo\n")?;
    write!(stdin, "mm\n")?;
    write!(stdin, "exactdist\n")?;
    write!(stdin, "ratings >ratings\n")?;
    write!(stdin, "x\n")?;
    write!(stdin, "x\n")?;
    child.wait()?;
    Ok(())
}

pub fn plot_ratings(dir: &PathBuf) -> std::io::Result<()> {
    let _ = dir;
    Ok(())
}

pub fn rankings(dir: &PathBuf) -> std::io::Result<Vec<String>> {
    let file = File::open(dir.join("ratings"))?;
    let reader = std::io::BufReader::new(file);
    let mut names = Vec::new();
    for line in reader.lines().skip(1) {
        let l = String::from(line?.trim());
        match l.find("model_") {
            Some(start_i) => {
                let end_i = l.find(".ot").unwrap();
                names.push(String::from(l[start_i..end_i + 3].trim()));
            }
            None => {}
        }
    }
    Ok(names)
}
