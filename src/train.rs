use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use rand::Rng;
use numpy::ndarray::{Array2, Array1};
use std::fs::File;
use std::io::{self, Write};

struct GameBoard {
    state: String,
    current_player: char,
    restrict_round: usize,
    round: usize,
}

impl GameBoard {
    fn new() -> Self {
        GameBoard {
            state: "RNBAKABNR/9/1C5C1/P1P1P1P1P/9/9/p1p1p1p1p/1c5c1/9/rnbakabnr".to_string(),
            current_player: 'w',
            restrict_round: 0,
            round: 0,
        }
    }

    fn reload(&mut self) {
        self.state = "RNBAKABNR/9/1C5C1/P1P1P1P1P/9/9/p1p1p1p1p/1c5c1/9/rnbakabnr".to_string();
        self.current_player = 'w';
        self.restrict_round = 0;
        self.round = 0;
    }

    // 其他方法...
}


struct SilverBullet {
    epochs: usize,
    playout_counts: usize,
    temperature: f32,
    batch_size: usize,
    game_batch: usize,
    game_loop: usize,
    top_steps: usize,
    top_temperature: f32,
    eta: f32,
    learning_rate: f32,
    lr_multiplier: f32,
    buffer_size: usize,
    data_buffer: VecDeque<(Array2<f32>, Array2<f32>, f32)>,
    game_borad: GameBoard,
    policy_value_netowrk: PolicyValueNet,
    search_threads: usize,
    mcts: MCTS,
    exploration: bool,
    resign_threshold: f32,
    global_step: usize,
    kl_targ: f32,
    log_file: File,
    human_color: char,
    queue: Arc<Mutex<VecDeque<QueueItem>>>,
    running_simulation_num: Arc<Mutex<usize>>,
}

impl SilverBullet {
    fn new(playout: usize, in_batch_size: usize, exploration: bool, in_search_threads: usize, processor: &str, num_gpus: usize, res_block_nums: usize, human_color: char) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let running_simulation_num = Arc::new(Mutex::new(1));

        SilverBullet {
            epochs: 5,
            playout_counts: playout,
            temperature: 1.0,
            batch_size: in_batch_size,
            game_batch: 40000,
            game_loop: 2000000,
            top_steps: 30,
            top_temperature: 1.0,
            eta: 0.03,
            learning_rate: 0.001,
            lr_multiplier: 1.0,
            buffer_size: 30000,
            data_buffer: VecDeque::with_capacity(30000),
            game_borad: GameBoard::new(),
            policy_value_netowrk: PolicyValueNet::new(),
            search_threads: in_search_threads,
            mcts: MCTS::new(&"RNBAKABNR/9/1C5C1/P1P1P1P1P/9/9/p1p1p1p1p/1c5c1/9/rnbakabnr".to_string(), PolicyValueNet::forward, in_search_threads),
            exploration,
            resign_threshold: -0.8,
            global_step: 0,
            kl_targ: 0.025,
            log_file: File::create("log_file.txt").unwrap(),
            human_color,
            queue,
            running_simulation_num,
        }
    }

    fn policy_update(&mut self) {
        let mini_batch: Vec<_> = self.data_buffer.iter().cloned().take(self.batch_size).collect();
        let state_batch: Vec<_> = mini_batch.iter().map(|data| data.0.clone()).collect();
        let mcts_probs_batch: Vec<_> = mini_batch.iter().map(|data| data.1.clone()).collect();
        let winner_batch: Vec<_> = mini_batch.iter().map(|data| data.2).collect();

        let start_time = std::time::Instant::now();
        let (old_probs, old_v) = self.mcts.forward(&Array2::from_shape_vec((state_batch.len(), state_batch[0].len_of(Axis(1))), state_batch.into_iter().flatten().collect()).unwrap());
        for _ in 0..self.epochs {
            let (accuracy, loss) = self.policy_value_netowrk.train_step(&Array2::from_shape_vec((state_batch.len(), state_batch[0].len_of(Axis(1))), state_batch.clone().into_iter().flatten().collect()).unwrap(), &Array2::from_shape_vec((mcts_probs_batch.len(), mcts_probs_batch[0].len_of(Axis(1))), mcts_probs_batch.clone().into_iter().flatten().collect()).unwrap(), &Array1::from_vec(winner_batch.clone()), self.learning_rate * self.lr_multiplier);
            let (new_probs, new_v) = self.mcts.forward(&Array2::from_shape_vec((state_batch.len(), state_batch[0].len_of(Axis(1))), state_batch.clone().into_iter().flatten().collect()).unwrap());

            let mut kl_tmp = Array2::<f32>::zeros((old_probs.nrows(), old_probs.ncols()));
            for ((old, new), kl) in old_probs.outer_iter().zip(new_probs.outer_iter()).zip(kl_tmp.outer_iter_mut()) {
                for ((o, n), k) in old.iter().zip(new.iter()).zip(kl.iter_mut()) {
                    *k = *o * ((*o + 1e-10) / (*n + 1e-10)).ln();
                }
            }

            let mut kl_lst = Vec::new();
            for line in kl_tmp.outer_iter() {
                let all_value: Vec<_> = line.iter().filter(|&&x| !x.is_nan() && !x.is_infinite()).cloned().collect();
                kl_lst.push(all_value.iter().sum());
            }
            let kl = kl_lst.iter().sum::<f32>() / kl_lst.len() as f32;

            if kl > self.kl_targ * 4.0 {
                break;
            }
        }
        self.global_step += 1;
        self.policy_value_netowrk.save(self.global_step);
        println!("train using time {} s", start_time.elapsed().as_secs_f32());

        if kl > self.kl_targ * 2.0 && self.lr_multiplier > 0.1 {
            self.lr_multiplier /= 1.5;
        } else if kl < self.kl_targ / 2.0 && self.lr_multiplier < 10.0 {
            self.lr_multiplier *= 1.5;
        }

        let old_v_flat = old_v.to_vec();
        let winner_batch_array = Array1::from_vec(winner_batch);
        let explained_var_old = 1.0 - old_v_flat.iter().zip(winner_batch_array.iter()).map(|(&o, &w)| (w - o).powi(2)).sum::<f32>() / winner_batch_array.iter().map(|&w| (w - winner_batch_array.mean().unwrap()).powi(2)).sum::<f32>();
        let new_v_flat = new_v.to_vec();
        let explained_var_new = 1.0 - new_v_flat.iter().zip(winner_batch_array.iter()).map(|(&o, &w)| (w - o).powi(2)).sum::<f32>() / winner_batch_array.iter().map(|&w| (w - winner_batch_array.mean().unwrap()).powi(2)).sum::<f32>();

        println!("kl:{:.5},lr_multiplier:{:.3},loss:{},accuracy:{},explained_var_old:{:.3},explained_var_new:{:.3}", kl, self.lr_multiplier, 0.0, 0.0, explained_var_old, explained_var_new);
        writeln!(self.log_file, "kl:{:.5},lr_multiplier:{:.3},loss:{},accuracy:{},explained_var_old:{:.3},explained_var_new:{:.3}", kl, self.lr_multiplier, 0.0, 0.0, explained_var_old, explained_var_new).unwrap();
        self.log_file.flush().unwrap();
    }

    async fn run(&mut self) {
        let mut batch_iter = 0;
        loop {
            if batch_iter >= self.game_loop {
                break;
            }
            batch_iter += 1;
            let (play_data, episode_len) = self.selfplay().await;
            println!("batch i:{}, episode_len:{}", batch_iter, episode_len);
            let mut extend_data = Vec::new();
            for (state, mcts_prob, winner) in play_data {
                let states_data = game::state_to_positions(&state);
                extend_data.push((states_data, mcts_prob, winner));
            }
            self.data_buffer.extend(extend_data);
            if self.data_buffer.len() > self.batch_size {
                self.policy_update();
            }
        }
    }

    fn get_action(&mut self, state: &str, temperature: f32) -> (String, Vec<(Vec<String>, Vec<f32>)>, f32) {
        self.mcts.main(state, self.game_borad.current_player, self.game_borad.restrict_round, self.playout_counts);

        let actions_visits: Vec<_> = self.mcts.root.lock().unwrap().child.iter().map(|(act, nod)| (act.clone(), nod.N)).collect();
        let (actions, visits): (Vec<_>, Vec<_>) = actions_visits.into_iter().unzip();
        let probs = game::softmax(1.0 / temperature * visits.iter().map(|&v| v as f32).collect::<Vec<_>>());
        let mut move_probs = Vec::new();

        move_probs.push((actions.clone(), probs.clone()));

        let act = if self.exploration {
            let new_p: Vec<_> = probs.iter().map(|&p| 0.75 * p + 0.25 * rand::thread_rng().gen_range(0.0..1.0)).collect();
            let sum: f32 = new_p.iter().sum();
            let new_p: Vec<_> = new_p.iter().map(|&p| p / sum).collect();
            let act_idx = rand::thread_rng().gen_range(0..actions.len());
            actions[act_idx].clone()
        } else {
            actions[0].clone()
        };

        let win_rate = self.mcts.Q(&act);
        self.mcts.update_tree(&act);

        (act, move_probs, win_rate)
    }

    fn check_end(&self) -> (bool, char) {
        if !self.game_borad.state.contains('K') || !self.game_borad.state.contains('k') {
            if !self.game_borad.state.contains('K') {
                println!("Green is Winner");
                return (true, 'b');
            }
            if !self.game_borad.state.contains('k') {
                println!("Red is Winner");
                return (true, 'w');
            }
        } else if self.game_borad.restrict_round >= 60 {
            println!("TIE! No Winners!");
            return (true, 't');
        } else {
            return (false, ' ');
        }
    }

    async fn selfplay(&mut self) -> (Vec<(String, Array2<f32>, f32)>, usize) {
        self.game_borad.reload();
        let mut states = Vec::new();
        let mut mcts_probs = Vec::new();
        let mut current_players = Vec::new();
        let mut actions = Vec::new();
        let mut z = None;
        let mut game_over = false;
        let mut winner = ' ';
        let start_time = std::time::Instant::now();

        while !game_over {
            let (action, probs, win_rate) = self.get_action(&self.game_board.state, self.temperature).await;
            let (state, player) = (self.game_board.state.clone(), self.game_board.current_player);

            states.push(state);
            mcts_probs.push(probs);
            current_players.push(player);
            actions.push(action);

            self.game_board.state = "new_state"; // 模拟更新状态
            self.game_board.round += 1;
            self.game_board.current_player = if self.game_board.current_player == 'b' { 'w' } else { 'b' };

            if self.game_board.state.find('K').is_none() || self.game_board.state.find('k').is_none() {
                if self.game_board.state.find('K').is_none() {
                    winner = "b";
                } else if self.game_board.state.find('k').is_none() {
                    winner = "w";
                }
                z = Some(vec![if player == *winner { 1.0 } else { -1.0 }; current_players.len()]);
                game_over = true;
            } else if self.game_board.restrict_round >= 60 {
                z = Some(vec![0.0; current_players.len()]);
                game_over = true;
            }
        }

        self.mcts.reload();
        (states.iter().zip(mcts_probs.iter()).zip(z.unwrap().iter()).map(|((s, p), z)| (s.clone(), p.clone(), *z)).collect(), states.len())
    }
}

#[tokio::main]
async fn main() {
    use clap::Parser;

    #[derive(Parser, Debug)]
    #[command(name = "aichess")]
    struct Args {
        #[arg(short, long, default_value = "train")]
        mode: String,

        #[arg(short, long, default_value = "1")]
        ai_count: usize,

        #[arg(short, long, default_value = "mcts")]
        ai_function: String,

        #[arg(short, long, default_value = "1600")]
        train_playout: usize,

        #[arg(short, long, default_value = "512")]
        batch_size: usize,

        #[arg(short, long, default_value = "1600")]
        play_playout: usize,

        #[arg(short, long, default_value = "3")]
        delay: f32,

        #[arg(short, long, default_value = "3")]
        end_delay: f32,

        #[arg(short, long, default_value = "16")]
        search_threads: usize,

        #[arg(short, long, default_value = "gpu")]
        processor: String,

        #[arg(short, long, default_value = "1")]
        num_gpus: usize,

        #[arg(short, long, default_value = "7")]
        res_block_nums: usize,

        #[arg(short, long, default_value = "b")]
        human_color: char,
    }

    let args = Args::parse();

    if args.mode == "train" {
        let silver_bullet = SilverBullet::new(
            args.train_playout,
            args.batch_size,
            true,
            args.search_threads,
            &args.processor,
            args.num_gpus,
            args.res_block_nums,
            args.human_color,
        );
        silver_bullet.run().await;
    } else if args.mode == "play" {
        // 实现游戏模式
    }
}
