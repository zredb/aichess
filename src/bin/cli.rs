use std::path::PathBuf;
use std::io::{self, Write};

use anyhow::Result;
use clap::{Parser, Subcommand};

use aichess::{
    alpha_zero, BurnTrainer, CChess, PlayerId, LearningConfig, 
    MCTSConfig, RolloutConfig, ActionSelection, Exploration, Fpu, PolicyNoise, 
    ValueTarget, Game, MCTS, PolicyWithCache, NetConfig, MAX_NUM_ACTIONS,
    AlphaZeroTrainer, HasTurnOrder,
};
use aichess::pos::{moves::Move, position::Position};
use aichess::synthesis::pgn::{PgnGame, load_pgn, append_game_to_pgn};

#[derive(Parser, Debug)]
#[command(name = "aichess-cli")]
#[command(author, version, about = "中国象棋AI命令行工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 训练模型
    Train {
        /// 日志和模型保存目录
        #[arg(short = 'd', long, default_value = "./logs")]
        log_dir: PathBuf,

        /// 训练迭代次数
        #[arg(short, long, default_value_t = 10)]
        iterations: usize,

        /// 每次训练的自对弈游戏数
        #[arg(short, long, default_value_t = 100)]
        games_per_train: usize,

        /// 保留的历史游戏数
        #[arg(long, default_value_t = 500)]
        games_to_keep: usize,

        /// 批次大小
        #[arg(short = 'b', long, default_value_t = 64)]
        batch_size: i64,

        /// 训练轮数
        #[arg(short, long, default_value_t = 5)]
        epochs: usize,

        /// MCTS探索次数
        #[arg(long, default_value_t = 800)]
        num_explores: usize,

        /// 工作线程数（目前强制为0）
        #[arg(short = 'w', long, default_value_t = 0)]
        workers: usize,

        /// 学习率
        #[arg(short = 'r', long, default_value_t = 0.001)]
        learning_rate: f64,

        /// 策略损失权重
        #[arg(long, default_value_t = 1.0)]
        policy_weight: f32,

        /// 价值损失权重
        #[arg(long, default_value_t = 1.0)]
        value_weight: f32,

        /// 随机种子
        #[arg(long, default_value_t = 42)]
        seed: u64,

        /// 神经网络隐藏层大小
        #[arg(long, default_value_t = 256)]
        hidden_size: usize,

        /// 神经网络残差块数量
        #[arg(long, default_value_t = 7)]
        num_blocks: usize,

        /// 根节点 Gumbel 噪声强度（0 表示关闭）
        #[arg(long, default_value_t = 0.0)]
        gumbel_scale: f32,

        /// 搜索蔑视（和棋惩罚，建议 0.0~0.1）
        #[arg(long, default_value_t = 0.0)]
        contempt: f32,

        /// 短深度将死搜索深度（0 表示关闭）
        #[arg(long, default_value_t = 4)]
        mate_search_depth: u8,

        /// Search-Contempt 退火迭代数（0 表示不退火）
        #[arg(long, default_value_t = 10)]
        contempt_anneal_iters: usize,

        /// Progressive Simulation 混合权重（0 表示关闭）
        #[arg(long, default_value_t = 0.15)]
        progressive_weight: f32,

        /// Progressive Simulation 退火访问数尺度
        #[arg(long, default_value_t = 24)]
        progressive_visits: usize,

        /// MCTS 评估微批大小
        #[arg(long, default_value_t = 8)]
        eval_batch_size: usize,
    },

    /// 两个模型对弈
    Play {
        /// 第一个模型路径
        #[arg(short, long)]
        model1: PathBuf,

        /// 第二个模型路径
        #[arg(short, long)]
        model2: PathBuf,

        /// 对弈局数
        #[arg(short, long, default_value_t = 10)]
        games: usize,

        /// MCTS探索次数
        #[arg(short, long, default_value_t = 800)]
        num_explores: usize,

        /// 根节点 Gumbel 噪声强度（0 表示关闭）
        #[arg(long, default_value_t = 0.0)]
        gumbel_scale: f32,

        /// 搜索蔑视（和棋惩罚）
        #[arg(long, default_value_t = 0.0)]
        contempt: f32,

        /// 短深度将死搜索深度（0 表示关闭）
        #[arg(long, default_value_t = 0)]
        mate_search_depth: u8,

        /// Progressive Simulation 混合权重（0 表示关闭）
        #[arg(long, default_value_t = 0.0)]
        progressive_weight: f32,

        /// Progressive Simulation 退火访问数尺度
        #[arg(long, default_value_t = 24)]
        progressive_visits: usize,

        /// MCTS 评估微批大小
        #[arg(long, default_value_t = 8)]
        eval_batch_size: usize,

        /// 是否打印棋盘
        #[arg(short, long)]
        verbose: bool,
    },

    /// 人机对弈
    Human {
        /// 模型路径
        #[arg(short, long)]
        model: PathBuf,

        /// 玩家执棋颜色 (red/black)
        #[arg(short, long, default_value = "red")]
        color: String,

        /// MCTS探索次数
        #[arg(short, long, default_value_t = 800)]
        num_explores: usize,

        /// 根节点 Gumbel 噪声强度（0 表示关闭）
        #[arg(long, default_value_t = 0.0)]
        gumbel_scale: f32,

        /// 搜索蔑视（和棋惩罚）
        #[arg(long, default_value_t = 0.0)]
        contempt: f32,

        /// 短深度将死搜索深度（0 表示关闭）
        #[arg(long, default_value_t = 0)]
        mate_search_depth: u8,

        /// Progressive Simulation 混合权重（0 表示关闭）
        #[arg(long, default_value_t = 0.0)]
        progressive_weight: f32,

        /// Progressive Simulation 退火访问数尺度
        #[arg(long, default_value_t = 24)]
        progressive_visits: usize,

        /// MCTS 评估微批大小
        #[arg(long, default_value_t = 8)]
        eval_batch_size: usize,

        /// 是否打印棋盘
        #[arg(short, long)]
        verbose: bool,

        /// 保存游戏到 PGN 文件
        #[arg(long)]
        save_pgn_file: Option<PathBuf>,
    },

    /// 查看或转换 PGN 文件
    Pgn {
        /// PGN 文件路径
        #[arg(short, long)]
        file: PathBuf,

        /// 操作类型 (show/convert)
        #[arg(short, long, default_value = "show")]
        action: String,
    },

    /// 批量评测搜索参数（同一对模型）
    Eval {
        /// 第一个模型路径
        #[arg(short, long)]
        model1: PathBuf,

        /// 第二个模型路径
        #[arg(short, long)]
        model2: PathBuf,

        /// 每组参数的对弈局数
        #[arg(short, long, default_value_t = 20)]
        games: usize,

        /// 候选 MCTS 探索次数（逗号分隔）
        #[arg(long, value_delimiter = ',', default_values_t = vec![800])]
        explores: Vec<usize>,

        /// 候选 Gumbel scale（逗号分隔）
        #[arg(long, value_delimiter = ',', default_values_t = vec![0.0])]
        gumbel_scales: Vec<f32>,

        /// 候选 contempt（逗号分隔）
        #[arg(long, value_delimiter = ',', default_values_t = vec![0.0])]
        contempts: Vec<f32>,

        /// 候选 mate depth（逗号分隔）
        #[arg(long, value_delimiter = ',', default_values_t = vec![0])]
        mate_depths: Vec<u8>,

        /// 候选 progressive weight（逗号分隔）
        #[arg(long, value_delimiter = ',', default_values_t = vec![0.0])]
        progressive_weights: Vec<f32>,

        /// 候选 progressive visits（逗号分隔）
        #[arg(long, value_delimiter = ',', default_values_t = vec![24])]
        progressive_visits: Vec<usize>,

        /// 候选 eval batch size（逗号分隔）
        #[arg(long, value_delimiter = ',', default_values_t = vec![8])]
        eval_batch_sizes: Vec<usize>,

        /// 评测结果 CSV 输出路径
        #[arg(long, default_value = "./eval_results.csv")]
        out_csv: PathBuf,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Train {
            log_dir,
            iterations,
            games_per_train,
            games_to_keep,
            batch_size,
            epochs,
            num_explores,
            workers,
            learning_rate,
            policy_weight,
            value_weight,
            seed,
            hidden_size,
            num_blocks,
            gumbel_scale,
            contempt,
            mate_search_depth,
            contempt_anneal_iters,
            progressive_weight,
            progressive_visits,
            eval_batch_size,
        } => {
            train_model(
                log_dir,
                iterations,
                games_per_train,
                games_to_keep,
                batch_size,
                epochs,
                num_explores,
                workers,
                learning_rate,
                policy_weight,
                value_weight,
                seed,
                hidden_size,
                num_blocks,
                gumbel_scale,
                contempt,
                mate_search_depth,
                contempt_anneal_iters,
                progressive_weight,
                progressive_visits,
                eval_batch_size,
            )?;
        }
        Commands::Play {
            model1,
            model2,
            games,
            num_explores,
            gumbel_scale,
            contempt,
            mate_search_depth,
            progressive_weight,
            progressive_visits,
            eval_batch_size,
            verbose,
        } => {
            play_models(
                model1,
                model2,
                games,
                num_explores,
                gumbel_scale,
                contempt,
                mate_search_depth,
                progressive_weight,
                progressive_visits,
                eval_batch_size,
                verbose,
            )?;
        }
        Commands::Human {
            model,
            color,
            num_explores,
            gumbel_scale,
            contempt,
            mate_search_depth,
            progressive_weight,
            progressive_visits,
            eval_batch_size,
            verbose,
            save_pgn_file,
        } => {
            play_human(
                model,
                color,
                num_explores,
                gumbel_scale,
                contempt,
                mate_search_depth,
                progressive_weight,
                progressive_visits,
                eval_batch_size,
                verbose,
                save_pgn_file,
            )?;
        }
        Commands::Pgn { file, action } => {
            handle_pgn(file, &action)?;
        }
        Commands::Eval {
            model1,
            model2,
            games,
            explores,
            gumbel_scales,
            contempts,
            mate_depths,
            progressive_weights,
            progressive_visits,
            eval_batch_sizes,
            out_csv,
        } => {
            eval_search_configs(
                model1,
                model2,
                games,
                &explores,
                &gumbel_scales,
                &contempts,
                &mate_depths,
                &progressive_weights,
                &progressive_visits,
                &eval_batch_sizes,
                &out_csv,
            )?;
        }
    }

    Ok(())
}

fn build_learning_config(
    log_dir: PathBuf,
    iterations: usize,
    games_per_train: usize,
    games_to_keep: usize,
    batch_size: i64,
    epochs: usize,
    num_explores: usize,
    workers: usize,
    learning_rate: f64,
    policy_weight: f32,
    value_weight: f32,
    seed: u64,
    gumbel_scale: f32,
    contempt: f32,
    mate_search_depth: u8,
    contempt_anneal_iters: usize,
    progressive_weight: f32,
    progressive_visits: usize,
    eval_batch_size: usize,
) -> LearningConfig {
    let action = if gumbel_scale > 0.0 {
        ActionSelection::Gumbel {
            scale: gumbel_scale,
        }
    } else {
        ActionSelection::NumVisits
    };

    LearningConfig {
        seed,
        logs: log_dir,
        lr_schedule: vec![(0, learning_rate)],
        weight_decay: 0.0,
        num_iterations: iterations,
        num_epochs: epochs,
        batch_size,
        policy_weight,
        value_weight,
        games_to_keep,
        games_per_train,
        rollout_cfg: RolloutConfig {
            num_workers: workers,
            num_explores,
            random_actions_until: 0,
            sample_actions_until: 0,
            stop_games_when_solved: true,
            value_target: ValueTarget::Z,
            action,
            mcts_cfg: MCTSConfig {
                exploration: Exploration::PolynomialUct { c: 1.25 },
                solve: false,
                correct_values_on_solve: false,
                select_solved_nodes: false,
                auto_extend: false,
                fpu: Fpu::Const(0.0),
                root_policy_noise: PolicyNoise::Equal { weight: 0.25 },
                contempt,
                mate_search_depth,
                progressive_simulation_weight: progressive_weight,
                progressive_simulation_visits: progressive_visits,
                eval_batch_size,
            },
            contempt_anneal_iters,
        },
    }
}

fn train_model(
    log_dir: PathBuf,
    iterations: usize,
    games_per_train: usize,
    games_to_keep: usize,
    batch_size: i64,
    epochs: usize,
    num_explores: usize,
    _workers: usize,
    learning_rate: f64,
    policy_weight: f32,
    value_weight: f32,
    seed: u64,
    hidden_size: usize,
    num_blocks: usize,
    gumbel_scale: f32,
    contempt: f32,
    mate_search_depth: u8,
    contempt_anneal_iters: usize,
    progressive_weight: f32,
    progressive_visits: usize,
    eval_batch_size: usize,
) -> Result<()> {
    println!("🚀 开始训练模型...");
    println!("📁 日志目录: {:?}", log_dir);
    println!("🔄 迭代次数: {}", iterations);
    println!("🎮 每轮游戏数: {}", games_per_train);
    println!("📦 批次大小: {}", batch_size);
    println!("📊 学习率: {}", learning_rate);
    println!("🧠 网络结构: hidden_size={}, num_blocks={}", hidden_size, num_blocks);
    println!("🎲 Gumbel scale: {}", gumbel_scale);
    println!("⚖️  Search contempt: {}", contempt);
    println!("♟️  Mate search depth: {}", mate_search_depth);
    println!("📉 Contempt anneal iters: {}", contempt_anneal_iters);
    println!("🎯 Progressive weight: {}", progressive_weight);
    println!("📏 Progressive visits: {}", progressive_visits);
    println!("🧩 Eval batch size: {}", eval_batch_size);
    println!("⚠️  注意: 使用单线程模式 (workers=0)");

    // 强制使用单线程以避免 Sync 问题
    let cfg = build_learning_config(
        log_dir.clone(),
        iterations,
        games_per_train,
        games_to_keep,
        batch_size,
        epochs,
        num_explores,
        0, // 强制单线程
        learning_rate,
        policy_weight,
        value_weight,
        seed,
        gumbel_scale,
        contempt,
        mate_search_depth,
        contempt_anneal_iters,
        progressive_weight,
        progressive_visits,
        eval_batch_size,
    );

    let device = Default::default();
    let model_config = NetConfig::new(hidden_size, num_blocks);
    let mut trainer = BurnTrainer::new(model_config, device);
    
    let report = alpha_zero::<CChess, _, MAX_NUM_ACTIONS>(&cfg, &mut trainer)?;

    println!("\n✅ 训练完成!");
    println!("📈 总迭代次数: {}", report.iterations.len());
    
    for iter in &report.iterations {
        println!(
            "\n📊 迭代 {}: ",
            iter.iteration
        );
        println!(
            "   游戏数={}, 新鲜步数={}, 回放游戏数={}, 回放步数={}, 去重步数={}",
            iter.games_played,
            iter.fresh_steps,
            iter.replay_games,
            iter.replay_steps,
            iter.deduplicated_steps
        );
        if let Some(total_loss) = iter.training.total_loss {
            println!(
                "   损失: 策略={:.4}, 价值={:.4}, 总计={:.4}",
                iter.training.policy_loss.unwrap_or(0.0),
                iter.training.value_loss.unwrap_or(0.0),
                total_loss
            );
        }
    }

    // 生成训练曲线图
    if let Err(e) = plot_training_curves(&report, &log_dir) {
        eprintln!("⚠️  生成训练曲线图失败: {}", e);
    }

    println!("\n💾 模型已保存到: {:?}", log_dir.join("models"));

    Ok(())
}

fn play_models(
    model1_path: PathBuf,
    model2_path: PathBuf,
    games: usize,
    num_explores: usize,
    gumbel_scale: f32,
    contempt: f32,
    mate_search_depth: u8,
    progressive_weight: f32,
    progressive_visits: usize,
    eval_batch_size: usize,
    _verbose: bool,
) -> Result<()> {
    println!("🎮 开始模型对弈...");
    println!("🤖 模型1: {:?}", model1_path);
    println!("🤖 模型2: {:?}", model2_path);
    println!("🔢 对弈局数: {}", games);
    println!("🔍 MCTS探索次数: {}", num_explores);

    use aichess::BurnBackend;
    let device: <BurnBackend as burn::prelude::Backend>::Device = Default::default();
    let model_config = NetConfig::new(256, 7);
    
    // 加载两个模型
    let trainer = BurnTrainer::new(model_config.clone(), device.clone());
    let mut policy1 = trainer.load_policy(&model1_path)?;
    let mut policy2 = trainer.load_policy(&model2_path)?;

    let cfg = SearchConfig {
        num_explores,
        gumbel_scale,
        contempt,
        mate_search_depth,
        progressive_weight,
        progressive_visits,
        eval_batch_size,
    };
    let stats = run_match_series(&mut policy1, &mut policy2, games, cfg, _verbose)?;

    println!("\n📊 对弈结果统计:");
    println!(
        "   模型1 胜: {} ({:.1}%)",
        stats.wins_p1,
        stats.wins_p1 as f64 / games as f64 * 100.0
    );
    println!(
        "   模型2 胜: {} ({:.1}%)",
        stats.wins_p2,
        stats.wins_p2 as f64 / games as f64 * 100.0
    );
    println!(
        "   和棋: {} ({:.1}%)",
        stats.draws,
        stats.draws as f64 / games as f64 * 100.0
    );

    Ok(())
}

#[derive(Debug)]
enum GameResult {
    Win(PlayerId),
    Draw,
}

#[derive(Clone, Copy, Debug)]
struct SearchConfig {
    num_explores: usize,
    gumbel_scale: f32,
    contempt: f32,
    mate_search_depth: u8,
    progressive_weight: f32,
    progressive_visits: usize,
    eval_batch_size: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct MatchStats {
    wins_p1: usize,
    wins_p2: usize,
    draws: usize,
}

fn play_single_game<P1, P2>(
    policy1: &mut P1,
    policy2: &mut P2,
    num_explores: usize,
    gumbel_scale: f32,
    contempt: f32,
    mate_search_depth: u8,
    progressive_weight: f32,
    progressive_visits: usize,
    eval_batch_size: usize,
    verbose: bool,
    p1_is_red: bool,
) -> Result<GameResult>
where
    P1: aichess::Policy<CChess, MAX_NUM_ACTIONS>,
    P2: aichess::Policy<CChess, MAX_NUM_ACTIONS>,
{
    let mut game = CChess::new();
    let mcts_cfg = MCTSConfig {
        exploration: Exploration::PolynomialUct { c: 1.25 },
        solve: false,
        correct_values_on_solve: false,
        select_solved_nodes: false,
        auto_extend: false,
        fpu: Fpu::Const(0.0),
        root_policy_noise: PolicyNoise::None,
        contempt,
        mate_search_depth,
        progressive_simulation_weight: progressive_weight,
        progressive_simulation_visits: progressive_visits,
        eval_batch_size,
    };
    let action_selection = if gumbel_scale > 0.0 {
        ActionSelection::Gumbel {
            scale: gumbel_scale,
        }
    } else {
        ActionSelection::NumVisits
    };

    let mut turn = 0;
    while !game.is_over() && turn < CChess::MAX_TURNS {
        if verbose {
            println!("\n回合 {}", turn + 1);
            game.print();
        }

        // 选择当前玩家的策略
        let current_player = game.player();
        let use_policy1 = if p1_is_red {
            current_player == PlayerId::Red
        } else {
            current_player == PlayerId::Black
        };

        let action = if use_policy1 {
            let mut cached = PolicyWithCache::with_capacity(100, policy1);
            let mut mcts = MCTS::with_capacity(num_explores + 1, mcts_cfg, &mut cached, game.clone());
            mcts.explore_n(num_explores);
            mcts.best_action(action_selection)
        } else {
            let mut cached = PolicyWithCache::with_capacity(100, policy2);
            let mut mcts = MCTS::with_capacity(num_explores + 1, mcts_cfg, &mut cached, game.clone());
            mcts.explore_n(num_explores);
            mcts.best_action(action_selection)
        };

        if verbose {
            println!("走法: {:?}", action);
        }

        game.step(&action);
        turn += 1;
    }

    if verbose {
        println!("\n最终局面:");
        game.print();
    }

    // 判断结果
    let winner = get_winner(&game);
    Ok(match winner {
        Some(w) => GameResult::Win(w),
        None => GameResult::Draw,
    })
}

fn run_match_series<P1, P2>(
    policy1: &mut P1,
    policy2: &mut P2,
    games: usize,
    cfg: SearchConfig,
    verbose: bool,
) -> Result<MatchStats>
where
    P1: aichess::Policy<CChess, MAX_NUM_ACTIONS>,
    P2: aichess::Policy<CChess, MAX_NUM_ACTIONS>,
{
    let mut stats = MatchStats::default();
    for game_idx in 0..games {
        if verbose {
            println!("\n--- 第 {} 局 ---", game_idx + 1);
        }
        let result = play_single_game(
            policy1,
            policy2,
            cfg.num_explores,
            cfg.gumbel_scale,
            cfg.contempt,
            cfg.mate_search_depth,
            cfg.progressive_weight,
            cfg.progressive_visits,
            cfg.eval_batch_size,
            verbose,
            game_idx % 2 == 0,
        )?;
        match result {
            GameResult::Win(player) => {
                if player == PlayerId::Red {
                    stats.wins_p1 += 1;
                    if verbose {
                        println!("🏆 红方(模型1)获胜!");
                    }
                } else {
                    stats.wins_p2 += 1;
                    if verbose {
                        println!("🏆 黑方(模型2)获胜!");
                    }
                }
            }
            GameResult::Draw => {
                stats.draws += 1;
                if verbose {
                    println!("🤝 和棋!");
                }
            }
        }
    }
    Ok(stats)
}

fn eval_search_configs(
    model1_path: PathBuf,
    model2_path: PathBuf,
    games: usize,
    explores: &[usize],
    gumbel_scales: &[f32],
    contempts: &[f32],
    mate_depths: &[u8],
    progressive_weights: &[f32],
    progressive_visits: &[usize],
    eval_batch_sizes: &[usize],
    out_csv: &PathBuf,
) -> Result<()> {
    use std::io::Write;
    use aichess::BurnBackend;
    let device: <BurnBackend as burn::prelude::Backend>::Device = Default::default();
    let model_config = NetConfig::new(256, 7);
    let trainer = BurnTrainer::new(model_config, device);
    let mut policy1 = trainer.load_policy(&model1_path)?;
    let mut policy2 = trainer.load_policy(&model2_path)?;

    println!("🔬 开始批量评测，共 {} 组参数", explores.len()
        * gumbel_scales.len()
        * contempts.len()
        * mate_depths.len()
        * progressive_weights.len()
        * progressive_visits.len()
        * eval_batch_sizes.len());
    println!("每组对弈 {} 局\n", games);

    if let Some(parent) = out_csv.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut csv = std::fs::File::create(out_csv)?;
    writeln!(
        csv,
        "idx,explores,gumbel_scale,contempt,mate_search_depth,progressive_weight,progressive_visits,eval_batch_size,games,wins_p1,wins_p2,draws,p1_win_rate,p2_win_rate,draw_rate"
    )?;

    let mut idx = 0usize;
    for &num_explores in explores {
        for &gumbel_scale in gumbel_scales {
            for &contempt in contempts {
                for &mate_search_depth in mate_depths {
                    for &progressive_weight in progressive_weights {
                        for &progressive_visits in progressive_visits {
                            for &eval_batch_size in eval_batch_sizes {
                            idx += 1;
                            let cfg = SearchConfig {
                                num_explores,
                                gumbel_scale,
                                contempt,
                                mate_search_depth,
                                progressive_weight,
                                progressive_visits,
                                eval_batch_size,
                            };
                            let stats =
                                run_match_series(&mut policy1, &mut policy2, games, cfg, false)?;
                            let p1 = stats.wins_p1 as f64 / games as f64 * 100.0;
                            let p2 = stats.wins_p2 as f64 / games as f64 * 100.0;
                            let draw = stats.draws as f64 / games as f64 * 100.0;
                            println!(
                                "#{idx:02} explore={} gumbel={:.2} contempt={:.3} mate={} prog_w={:.2} prog_v={} eval_bs={} | p1={:.1}% p2={:.1}% draw={:.1}%",
                                num_explores,
                                gumbel_scale,
                                contempt,
                                mate_search_depth,
                                progressive_weight,
                                progressive_visits,
                                eval_batch_size,
                                p1,
                                p2,
                                draw
                            );
                            writeln!(
                                csv,
                                "{},{},{:.6},{:.6},{},{:.6},{},{},{},{},{},{},{:.6},{:.6},{:.6}",
                                idx,
                                num_explores,
                                gumbel_scale,
                                contempt,
                                mate_search_depth,
                                progressive_weight,
                                progressive_visits,
                                eval_batch_size,
                                games,
                                stats.wins_p1,
                                stats.wins_p2,
                                stats.draws,
                                p1,
                                p2,
                                draw
                            )?;
                            }
                        }
                    }
                }
            }
        }
    }
    println!("\n💾 评测结果已写入: {:?}", out_csv);

    Ok(())
}

fn get_winner(game: &CChess) -> Option<PlayerId> {
    // 检查是否将帅被吃
    let mut has_red_king = false;
    let mut has_black_king = false;
    
    let fen_str = game.state().fen_str();
    for (piece, _) in aichess::fen::fen2_coords(fen_str) {
        match piece {
            'K' => has_red_king = true,
            'k' => has_black_king = true,
            _ => {}
        }
    }

    match (has_red_king, has_black_king) {
        (true, false) => Some(PlayerId::Red),
        (false, true) => Some(PlayerId::Black),
        (false, false) => None,
        (true, true) => {
            // 检查是否无合法走法
            let position = Position::from_fen(&game.state());
            if position.gen_legal_moves().is_empty() {
                Some(game.player().prev())
            } else {
                None
            }
        }
    }
}

fn play_human(
    model_path: PathBuf,
    color: String,
    num_explores: usize,
    gumbel_scale: f32,
    contempt: f32,
    mate_search_depth: u8,
    progressive_weight: f32,
    progressive_visits: usize,
    eval_batch_size: usize,
    _verbose: bool,
    save_pgn_file: Option<PathBuf>,
) -> Result<()> {
    println!("🎮 开始人机对弈...");
    println!("🤖 AI模型: {:?}", model_path);
    println!("👤 玩家颜色: {}", color);
    println!("🔍 MCTS探索次数: {}", num_explores);
    if let Some(ref pgn_path) = save_pgn_file {
        println!("💾 游戏将保存到: {:?}", pgn_path);
    }

    use aichess::BurnBackend;
    let device: <BurnBackend as burn::prelude::Backend>::Device = Default::default();
    let model_config = NetConfig::new(256, 7);
    let trainer = BurnTrainer::new(model_config, device);
    let mut ai_policy = trainer.load_policy(&model_path)?;

    let player_color = if color.to_lowercase() == "red" || color.to_lowercase() == "r" {
        PlayerId::Red
    } else {
        PlayerId::Black
    };

    println!("\n提示:");
    println!("  - 输入走法编号 (如 0, 1, 2)");
    println!("  - 或输入 ICCS 坐标 (如 h2e2)");
    println!("  - 输入 'quit' 或 'q' 退出游戏\n");

    let mut game = CChess::new();
    let mut pgn_game = PgnGame::new();
    
    // 设置 PGN 头部信息
    pgn_game.set_header("Event", "人机对弈");
    pgn_game.set_header("White", if player_color == PlayerId::Red { "Human" } else { "AI" });
    pgn_game.set_header("Black", if player_color == PlayerId::Black { "Human" } else { "AI" });
    
    let mcts_cfg = MCTSConfig {
        exploration: Exploration::PolynomialUct { c: 1.25 },
        solve: false,
        correct_values_on_solve: false,
        select_solved_nodes: false,
        auto_extend: false,
        fpu: Fpu::Const(0.0),
        root_policy_noise: PolicyNoise::None,
        contempt,
        mate_search_depth,
        progressive_simulation_weight: progressive_weight,
        progressive_simulation_visits: progressive_visits,
        eval_batch_size,
    };
    let action_selection = if gumbel_scale > 0.0 {
        ActionSelection::Gumbel {
            scale: gumbel_scale,
        }
    } else {
        ActionSelection::NumVisits
    };

    let mut turn = 0;
    while !game.is_over() && turn < CChess::MAX_TURNS {
        println!("\n=== 回合 {} ===", turn + 1);
        game.print();

        let current_player = game.player();
        println!("当前玩家: {:?}", current_player);

        if current_player == player_color {
            // 人类玩家走棋
            print!("请输入你的走法: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.to_lowercase() == "quit" || input.to_lowercase() == "q" {
                println!("游戏结束");
                return Ok(());
            }

            // 解析人类输入
            let action = parse_human_move(&game, input)?;
            let move_str = format!("{:?}", action);
            pgn_game.add_move(&move_str);
            game.step(&action);
        } else {
            // AI走棋
            println!("🤔 AI思考中...");
            let mut cached = PolicyWithCache::with_capacity(100, &mut ai_policy);
            let mut mcts = MCTS::with_capacity(num_explores + 1, mcts_cfg, &mut cached, game.clone());
            mcts.explore_n(num_explores);
            let action = mcts.best_action(action_selection);
            
            let move_str = format!("{:?}", action);
            pgn_game.add_move(&move_str);
            println!("AI走法: {}", move_str);
            game.step(&action);
        }

        turn += 1;
    }

    // 游戏结束
    println!("\n=== 游戏结束 ===");
    game.print();

    let winner = get_winner(&game);
    match winner {
        Some(w) => {
            pgn_game.result = Some(if w == player_color { "1-0" } else { "0-1" }.to_string());
            if w == player_color {
                println!("🎉 恭喜你获胜!");
            } else {
                println!("😔 AI获胜，再接再厉!");
            }
        }
        None => {
            pgn_game.result = Some("1/2-1/2".to_string());
            println!("🤝 和棋!");
        }
    }

    // 保存 PGN 文件
    if let Some(pgn_path) = save_pgn_file {
        match append_game_to_pgn(&pgn_game, &pgn_path) {
            Ok(_) => println!("💾 游戏已保存到: {:?}", pgn_path),
            Err(e) => eprintln!("❌ 保存 PGN 失败: {}", e),
        }
    }

    Ok(())
}

fn parse_human_move(game: &CChess, input: &str) -> Result<Move> {
    // 获取所有合法走法
    let legal_moves: Vec<Move> = {
        let position = Position::from_fen(&game.state());
        position.gen_legal_moves()
    };

    if legal_moves.is_empty() {
        anyhow::bail!("没有合法走法");
    }

    // 尝试解析为数字索引
    if let Ok(idx) = input.parse::<usize>() {
        if idx < legal_moves.len() {
            return Ok(legal_moves[idx]);
        }
    }

    // 尝试解析为 ICCS 格式 (如 "h2e2")
    if input.len() == 4 {
        let chars: Vec<char> = input.chars().collect();
        if let Some(move_found) = find_move_by_iccs(&legal_moves, &chars) {
            return Ok(*move_found);
        }
    }

    // 显示所有合法走法
    println!("\n合法走法列表:");
    for (i, m) in legal_moves.iter().enumerate() {
        println!("  {}: {}", i, m);
    }
    
    anyhow::bail!("无法解析走法 '{}', 请输入走法编号或ICCS坐标(如h2e2)", input);
}

/// 根据 ICCS 坐标查找走法
fn find_move_by_iccs<'a>(moves: &'a [Move], coords: &[char]) -> Option<&'a Move> {
    // 简化实现：比较起始和目标位置的字符表示
    moves.iter().find(|m| {
        let from_str = format!("{:x}", m.from);
        let to_str = format!("{:x}", m.to);
        let input_str: String = coords.iter()
            .map(|c| c.to_lowercase().next().unwrap())
            .collect();
        format!("{}{}", from_str, to_str) == input_str
    })
}

fn handle_pgn(file: PathBuf, action: &str) -> Result<()> {
    match action {
        "show" => {
            println!("📄 加载 PGN 文件: {:?}", file);
            let games = load_pgn(&file)?;
            println!("✅ 找到 {} 个游戏\n", games.len());

            for (i, game) in games.iter().enumerate() {
                println!("=== 游戏 {} ===", i + 1);
                for (key, value) in &game.headers {
                    println!("{}: {}", key, value);
                }
                println!("\n走法:");
                let mut move_text = String::new();
                for (j, mv) in game.moves.iter().enumerate() {
                    if j % 2 == 0 {
                        if !move_text.is_empty() {
                            move_text.push('\n');
                        }
                        move_text.push_str(&format!("{}. {}", j / 2 + 1, mv));
                    } else {
                        move_text.push_str(&format!("  {}", mv));
                    }
                }
                println!("{}", move_text);
                if let Some(result) = &game.result {
                    println!("\n结果: {}", result);
                }
                println!();
            }
        }
        "convert" => {
            println!("⚙️  转换功能开发中...");
            // TODO: 实现格式转换功能
        }
        _ => {
            anyhow::bail!("未知操作: {}。支持的操作: show, convert", action);
        }
    }

    Ok(())
}

/// 绘制训练曲线图
fn plot_training_curves(
    report: &aichess::AlphaZeroReport,
    log_dir: &PathBuf,
) -> Result<()> {
    use plotters::prelude::*;

    if report.iterations.is_empty() {
        return Ok(());
    }

    let chart_path = log_dir.join("training_curves.png");
    
    // 准备数据
    let iterations: Vec<usize> = report.iterations.iter().map(|i| i.iteration).collect();
    let policy_losses: Vec<f32> = report.iterations.iter()
        .map(|i| i.training.policy_loss.unwrap_or(0.0))
        .collect();
    let value_losses: Vec<f32> = report.iterations.iter()
        .map(|i| i.training.value_loss.unwrap_or(0.0))
        .collect();
    let total_losses: Vec<f32> = report.iterations.iter()
        .map(|i| i.training.total_loss.unwrap_or(0.0))
        .collect();

    // 创建图表
    let root = BitMapBackend::new(&chart_path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("训练损失曲线", ("sans-serif", 30).into_font())
        .margin(50)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(
            0..iterations.len().saturating_sub(1),
            0.0f32..total_losses.iter().cloned().fold(0.0f32, f32::max).max(1.0),
        )?;

    chart.configure_mesh()
        .x_desc("迭代次数")
        .y_desc("损失值")
        .x_label_style(("sans-serif", 15).into_font())
        .y_label_style(("sans-serif", 15).into_font())
        .draw()?;

    // 绘制策略损失
    chart.draw_series(LineSeries::new(
        iterations.iter().zip(policy_losses.iter()).map(|(&x, &y)| (x, y)),
        &RED,
    ))?
    .label("策略损失")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // 绘制价值损失
    chart.draw_series(LineSeries::new(
        iterations.iter().zip(value_losses.iter()).map(|(&x, &y)| (x, y)),
        &BLUE,
    ))?
    .label("价值损失")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // 绘制总损失
    chart.draw_series(LineSeries::new(
        iterations.iter().zip(total_losses.iter()).map(|(&x, &y)| (x, y)),
        &GREEN,
    ))?
    .label("总损失")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 15).into_font())
        .draw()?;

    root.present()?;

    println!("📊 训练曲线图已保存到: {:?}", chart_path);

    Ok(())
}
