use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use plotters::backend::BitMapBackend;
use plotters::element::EmptyElement;
use plotters::prelude::{BLACK, ChartBuilder, Circle, Color, IntoFont, LineSeries, PointSeries, RED, ShapeStyle, WHITE};

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
    let file = File::open(dir)?;
    let reader = io::BufReader::new(file);

    let mut scores = vec![];
    let mut random_score = None;
    let mut mcts_scores = vec![];

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue; // skip header
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let elo = parts[2].parse::<i32>()?;
        if parts[1] == "Random" {
            random_score = Some(elo);
        } else if parts[1].contains("VanillaMCTS") {
            mcts_scores.push((parts[1].to_string(), elo));
        } else {
            let num = parts[1].split('_').nth(1).unwrap().split('.').nth(0).unwrap().parse::<usize>()?;
            scores.push((num, elo));
        }
    }

    if !scores.is_empty() {
        let root = BitMapBackend::new("ratings.png", (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Strength through training", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..scores.len(), -20..*scores.iter().map(|(_, elo)| elo).max().unwrap())?;

        chart.configure_mesh().draw()?;

        let names: Vec<usize> = scores.iter().map(|(name, _)| *name).collect();
        let elos: Vec<i32> = scores.iter().map(|(_, elo)| elo - scores[0].1).collect();

        chart.draw_series(LineSeries::new(
            names.iter().zip(elos.iter()).map(|(x, y)| (*x, *y)),
            &RED,
        ))?;

        chart.draw_series(PointSeries::of_element(
            names.iter().zip(elos.iter()).map(|(x, y)| (*x, *y)),
            5,
            ShapeStyle::from(&RED).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord) + Circle::new((0, 0), size, style)
            },
        ))?;

        for (name, elo) in mcts_scores {
            if elo - scores[0].1 < 0 {
                continue;
            }
            chart.draw_series(LineSeries::new(
                vec![(names[0], elo - scores[0].1), (names[names.len() - 1], elo - scores[0].1)],
                &BLACK.stroke_width(1).dotted(),
            ))?;

            chart.draw_text(
                &name.replace("VanillaMCTS", ""),
                (names[names.len() - 1], elo - scores[0].1),
                ("sans-serif", 15).into_font(),
                &BLACK,
            )?;
        }

        root.present()?;
    }

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
