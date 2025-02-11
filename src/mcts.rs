use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use tokio::sync::Semaphore;
use rand::distributions::{Distribution};
use rand::thread_rng;
use rand_distr::Dirichlet;

const C_PUCT: f64 = 5.0;
const VIRTUAL_LOSS: u32 = 3;

struct LeafNode {
    p: f64,
    q: f64,
    n: u32,
    v: f64,
    u: f64,
    w: f64,
    parent: Option<Arc<Mutex<LeafNode>>>,
    children: HashMap<String, Arc<Mutex<LeafNode>>>,
    state: String, // fen
}

impl LeafNode {
    fn new(in_parent: Option<Arc<Mutex<LeafNode>>>, in_prior_p: f64, in_state: String) -> Self {
        LeafNode {
            p: in_prior_p,
            q: 0.0,
            n: 0,
            v: 0.0,
            u: 0.0,
            w: 0.0,
            parent: in_parent,
            children: HashMap::new(),
            state: in_state,
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn get_q_plus_u(&self, c_puct: f64) -> f64 {
        let u = c_puct * self.p * (self.parent.as_ref().unwrap().lock().unwrap().n as f64).sqrt() / (1.0 + self.n as f64);
        self.q + u
    }

    fn select(&self, c_puct: f64) -> (&String, Arc<Mutex<LeafNode>>) {
        let mut max_node = None;
        let mut max_value = f64::MIN;

        for (action, node) in &self.children {
            let node_locked = node.lock().unwrap();
            let q_plus_u = node_locked.get_q_plus_u(c_puct);
            if q_plus_u > max_value {
                max_value = q_plus_u;
                max_node = Some((action, node.clone()));
            }
        }

        max_node.unwrap()
    }

    fn expand(&mut self, moves: Vec<String>, action_probs: Vec<f64>) {
        let mut tot_p = 1e-8;
        for (action, &mov_p) in moves.iter().zip(action_probs.iter()) {
            let in_state = game::sim_do_action(action, &self.state);
            // 使用 Arc::clone 创建当前节点的强引用
            let new_node = Arc::new(Mutex::new(LeafNode::new(Some(Arc::clone(&self.parent.as_ref().unwrap())), mov_p, in_state)));
            self.children.insert(action.clone(), new_node);
            tot_p += mov_p;
        }
        for (_, n) in self.children.iter_mut() {
            n.lock().unwrap().p /= tot_p;
        }
    }

    fn back_up_value(&mut self, value: f64) {
        self.n += 1;
        self.w += value;
        self.v = value;
        self.q = self.w as f64 / self.n as f64;
        self.u = C_PUCT * self.p * (self.parent.as_ref().unwrap().lock().unwrap().n as f64).sqrt() / (1.0 + self.n as f64);
    }
}

struct MCTS {
    noise_eps: f64,
    dirichlet_alpha: f64,
    p_: f64,
    root: Arc<Mutex<LeafNode>>,
    c_puct: f64,
    forward: fn(&Vec<f64>) -> (Vec<f64>, Vec<f64>),
    node_lock: HashMap<String, Mutex<()>>,
    virtual_loss: i32,
    now_expanding: HashSet<String>,  // 修改为 HashSet<String>
    expanded: HashSet<String>,       // 修改为 HashSet<String>
    
}

impl MCTS {
    fn new(in_state: String, in_forward: fn(&Vec<f64>) -> (Vec<f64>, Vec<f64>), search_threads: usize) -> Self {
        let noise_eps = 0.25;
        let dirichlet_alpha = 0.3;
        let dirichlet_dist = Dirichlet::new(&vec![dirichlet_alpha, dirichlet_alpha]).unwrap().sample(&mut thread_rng());
        let p_ = (1.0 - noise_eps) * 1.0 + noise_eps * dirichlet_dist[0];
        let root = Arc::new(Mutex::new(LeafNode::new(None, p_, in_state)));
        MCTS {
            noise_eps,
            dirichlet_alpha,
            p_,
            root,
            c_puct: 5.0,
            forward: in_forward,
            node_lock: HashMap::new(),
            virtual_loss: 3,
            now_expanding: HashSet::new(),
            expanded: HashSet::new(),
        }
    }

    fn reload(&mut self) {
        self.root = Arc::new(Mutex::new(LeafNode::new(
            None,
            self.p_.clone(),
            "RNBAKABNR/9/1C5C1/P1P1P1P1P/9/9/p1p1p1p1p/1c5c1/9/rnbakabnr".to_string(),
        )));
        self.expanded = HashSet::new();
    }
    fn Q(&self, move_: &str) -> f32 {
        let mut ret = 0.0;
        let mut find = false;

        for (a, n) in &self.root.lock().unwrap().children.iter() {
            if move_ == a {
                ret = n.Q;
                find = true;
                break;
            }
        }

        if !find {
            println!("{} not exist in the child", move_);
        }

        ret
    }
    fn update_tree(&mut self, act: &str) {
        // 先获取新的根节点，并释放锁
        let new_root = {
            let root_node = self.root.lock().unwrap();
            self.expanded.remove(&root_node.state);
            root_node.children.get(act).cloned().unwrap_or_else(|| {
                panic!("Action {} not found in child nodes", act);
            })
        };

        // 修改 self.root
        self.root = new_root;
        self.expanded = HashSet::new();
    }

    async fn tree_search(&mut self, node: Arc<Mutex<LeafNode>>, current_player: &str, restrict_round: i32) -> f64 {
        // self.running_simulation_num.fetch_add(1, Ordering::SeqCst);
        // let permit = self.sem.acquire().await.unwrap();
        let value = self.start_tree_search(node, current_player, restrict_round).await;
        // self.running_simulation_num.fetch_sub(1, Ordering::SeqCst);
        // drop(permit);
        value
    }

    async fn start_tree_search(&mut self, node: Arc<Mutex<LeafNode>>, current_player: &str, restrict_round: i32) -> f64 {
        let now_expanding = &self.now_expanding;
        let node_id = &node.lock().unwrap().state;  // 获取节点ID

        while now_expanding.contains(node_id) { // 使用节点ID
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }

        if !self.expanded.contains(node_id) { // 使用节点ID
            self.now_expanding.insert(node_id.to_string()); // 使用节点ID

            let positions = game::generate_inputs(&node.lock().unwrap().state, current_player);
            let (action_probs, value) = (self.forward)(&positions);

            let moves = game::get_legal_moves(&node.lock().unwrap().state, current_player);
            node.lock().unwrap().expand(moves, action_probs);
            self.expanded.insert(node_id.to_string()); // 使用节点ID

            self.now_expanding.remove(node_id); // 使用节点ID

            return value[0] * -1.0;
        } else {
            let last_state = node.lock().unwrap().state.clone();
            let (action, selected_node) = node.lock().unwrap().select(self.c_puct);
            let current_player = if current_player == "b" { "w" } else { "b" };
            let restrict_round = if game::is_kill_move(&last_state, &selected_node.lock().unwrap().state) == 0 { restrict_round + 1 } else { 0 };

            selected_node.lock().unwrap().n += self.virtual_loss as u32;
            selected_node.lock().unwrap().w += -self.virtual_loss as f64;

            let value = if selected_node.lock().unwrap().state.contains('K') && selected_node.lock().unwrap().state.contains('k') {
                if restrict_round >= 60 {
                    0.0
                } else {
                    // 使用 Box::pin 包装递归调用
                    Box::pin(self.start_tree_search(selected_node.clone(), current_player, restrict_round)).await
                }
            } else if !selected_node.lock().unwrap().state.contains('K') {
                if current_player == "b" { 1.0 } else { -1.0 }
            } else if current_player == "b" { -1.0 } else { 1.0 };

            selected_node.lock().unwrap().n += -self.virtual_loss as u32;
            selected_node.lock().unwrap().w += self.virtual_loss as f64;

            selected_node.lock().unwrap().back_up_value(value);
            return value * -1.0;
        }
    }

    async fn prediction_worker(&mut self) {
        // Placeholder for prediction worker logic
    }

    async fn main(&mut self, current_player: &str, restrict_round: i32, playouts: usize) {
        let node = self.root.clone();

        if !self.expanded.contains(&node.lock().unwrap().id) {  // 使用节点ID
            let positions = game::generate_inputs(&node.lock().unwrap().state, current_player);
            let (action_probs, value) = (self.forward)(&positions);

            let moves = game::get_legal_moves(&node.lock().unwrap().state, current_player);
            node.lock().unwrap().expand(moves, action_probs);
            self.expanded.insert(node.lock().unwrap().id);  // 使用节点ID
        }

        let mut handles = vec![];
        for _ in 0..playouts {
            // 克隆 Arc<Mutex<MCTS>> 而不是内部的 MCTS
            let node_clone = node.clone();
            let current_player_clone = current_player.to_string();
            let handle=tokio::spawn(async move {
                self.tree_search(node_clone,&current_player_clone, restrict_round).await
            });
            handles.push(handle);
        }
   

        for handle in handles {
            if let Ok(value) = handle.await {
                    println!("Simulation result: {}", value);
            }
        }
    }

}

mod game {
    pub fn sim_do_action(action: &str, state: &str) -> String {
        // Placeholder for sim_do_action logic
        state.to_string()
    }

    pub fn generate_inputs(state: &str, current_player: &str) -> Vec<f64> {
        // Placeholder for generate_inputs logic
        vec![0.0]
    }

    pub fn flip_policy(action_probs: Vec<f64>) -> Vec<f64> {
        // Placeholder for flip_policy logic
        action_probs
    }

    pub fn get_legal_moves(state: &str, current_player: &str) -> Vec<String> {
        // Placeholder for get_legal_moves logic
        vec!["move1".to_string()]
    }

    pub fn is_kill_move(last_state: &str, node_state: &str) -> i32 {
        // Placeholder for is_kill_move logic
        0
    }

    pub fn is_black_turn(current_player: &str) -> bool {
        // Placeholder for is_black_turn logic
        current_player == "b"
    }

    pub fn try_flip(state: &str, current_player: &str, is_black: bool) -> (String, String) {
        // Placeholder for try_flip logic
        (state.to_string(), current_player.to_string())
    }
}

#[tokio::main]
async fn main() {
    let state = "1NBAKABNR/R8/1C5C1/P1P1P1P1P/9/9/p1p1p1p1p/1c5c1/9/rnbakabnr".to_string();
    let current_player = "b";
    let (new_state, new_player) = game::try_flip(&state, current_player, game::is_black_turn(current_player));
    println!("{} {}", new_state, new_player);

    let mcts = Arc::new(Mutex::new(MCTS::new(state, game::forward, 4)));
    mcts.lock().await.main(current_player, 0, 100).await;
}

fn forward(features: &Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    // Placeholder for forward logic
    (vec![0.0], vec![0.0])
}
