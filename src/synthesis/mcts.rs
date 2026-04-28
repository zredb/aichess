use crate::synthesis::game::Outcome;
use crate::synthesis::{ActionSelection, Exploration, Fpu, Game, MCTSConfig, Policy, PolicyNoise};
use rand::distr::Distribution;
use rand::rng;
use rand::Rng;
use rand::RngExt;
use rand_distr::multi::Dirichlet;

type NodeId = u32;
type ActionId = u16;

impl From<Outcome> for usize {
    /// 用于将 Outcome 枚举转换为 usize 类型的整数。具体逻辑如下：
    /// 如果是 Lose，返回 0
    /// 如果是 Draw，返回 1
    /// 如果是 Win，返回 2
    fn from(outcome: Outcome) -> usize {
        match outcome {
            Outcome::Lose(_) => 0,
            Outcome::Draw(_) => 1,
            Outcome::Win(_) => 2,
        }
    }
}

impl From<Outcome> for [f32; 3] {
    ///该函数将枚举类型转换为一个长度为3的数组，数组中只有一个元素为1.0，其余为0.0。具体步骤如下：
    /// - 初始化一个长度为3的数组 `dist`，所有元素初始值为0.0。
    /// - 将枚举类型的 `self` 转换为 usize 类型的索引。
    /// - 将数组中对应索引位置的元素设为1.0。
    /// - 返回数组 `dist`。
    fn from(outcome: Outcome) -> [f32; 3] {
        let mut dist = [0.0; 3];
        dist[usize::from(outcome)] = 1.0;
        dist
    }
}

#[derive(Debug)]
struct Node<G: Game<N>, const N: usize> {
    //
    parent: NodeId,            // 4 bytes
    first_child: NodeId,       // 4 bytes
    num_children: u8,          // 1 byte
    game: G,                   // ? bytes
    solution: Option<Outcome>, // 1 byte
    action: ActionId,          // 1 byte
    action_prob: f32,          // 4 bytes
    outcome_probs: [f32; 3],
    num_visits: f32, // 4 bytes
}

impl<G: Game<N>, const N: usize> Node<G, N> {
    /// 计算并返回当前节点的胜率差值与访问次数的比值
    fn q(&self) -> f32 {
        (self.outcome_probs[2] - self.outcome_probs[0]) / self.num_visits
    }

    fn q_with_contempt(&self, contempt: f32) -> f32 {
        if contempt == 0.0 {
            return self.q();
        }
        (self.outcome_probs[2] - self.outcome_probs[0] - contempt * self.outcome_probs[1])
            / self.num_visits
    }

    /// Create a new unvisited node.
    fn unvisited(
        parent: NodeId,
        game: G,
        solution: Option<Outcome>,
        action: ActionId,
        action_prob: f32,
    ) -> Self {
        Self {
            parent,
            first_child: 0,
            num_children: 0,
            game,
            action,
            solution,
            action_prob,
            outcome_probs: [0.0; 3],
            num_visits: 0.0,
        }
    }

    /// 获取当前节点的动作。
    fn action(&self) -> G::Action {
        (self.action as usize).into()
    }

    /// 检查当前节点是否未被访问。
    #[inline]
    fn is_unvisited(&self) -> bool {
        self.num_children == 0 && self.solution.is_none()
    }

    /// 检查当前节点是否已被访问。
    #[inline]
    #[allow(dead_code)]
    fn is_visited(&self) -> bool {
        self.num_children != 0
    }

    /// 检查当前节点是否未解决。
    #[inline]
    #[allow(dead_code)]
    fn is_unsolved(&self) -> bool {
        self.solution.is_none()
    }

    /// 获取当前节点的最后一个子节点ID。
    #[inline]
    fn last_child(&self) -> NodeId {
        self.first_child + self.num_children as u32
    }

    /// 标记当前节点为已访问，并设置其第一个子节点ID和子节点数量。
    #[inline]
    fn mark_visited(&mut self, first_child: NodeId, num_children: u8) {
        self.first_child = first_child;
        self.num_children = num_children;
    }

    /// 标记当前节点为已解决，并设置其解决方案。
    #[inline]
    fn mark_solved(&mut self, outcome: Outcome) {
        self.solution = Some(outcome);
    }
}

pub struct MCTS<'a, G: Game<N>, P: Policy<G, N>, const N: usize> {
    root: NodeId,
    offset: NodeId,
    nodes: Vec<Node<G, N>>,
    policy: &'a mut P,
    cfg: MCTSConfig,
}

enum ExploreTask<G: Game<N>, const N: usize> {
    BackpropNow {
        node_id: NodeId,
        outcome_probs: [f32; 3],
        solved: bool,
    },
    NeedEval {
        node_id: NodeId,
        game: G,
        first_child: NodeId,
        last_child: NodeId,
        any_solved: bool,
    },
}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> MCTS<'a, G, P, N> {
    /// 通过蒙特卡罗树搜索（MCTS）算法来选择最佳动作。具体步骤如下：
    /// 创建一个容量为 explores + 1 的 MCTS 实例。
    /// 执行指定次数的探索操作。
    /// 根据给定的动作选择策略返回最佳动作。
    pub fn exploit(
        explores: usize,
        cfg: MCTSConfig,
        policy: &'a mut P,
        game: G,
        action_selection: ActionSelection,
    ) -> G::Action {
        let mut mcts = Self::with_capacity(explores + 1, cfg, policy, game);
        mcts.explore_n(explores);
        mcts.best_action(action_selection)
    }

    /// 创建一个指定容量的 MCTS 实例。
    pub fn with_capacity(capacity: usize, cfg: MCTSConfig, policy: &'a mut P, game: G) -> Self {
        let mut nodes = Vec::with_capacity(capacity);
        nodes.push(Node::unvisited(0, game, None, 0, 0.0));
        let mut mcts = Self {
            root: 0,
            offset: 0,
            nodes,
            policy,
            cfg,
        };
        let (node_id, outcome_probs, any_solved) = mcts.visit(mcts.root);
        mcts.backprop(node_id, outcome_probs, any_solved);
        mcts.add_root_noise();
        mcts
    }

    /// 执行指定次数的探索操作。
    pub fn explore_n(&mut self, n: usize) {
        // Run explores in micro-batches so multiple leaf evaluations can share one eval_batch call.
        let batch_size = self.cfg.eval_batch_size.max(1);
        let mut remaining = n;
        while remaining > 0 {
            if self.node(self.root).solution.is_some() {
                break;
            }
            let this_round = remaining.min(batch_size);
            self.explore_batch(this_round);
            remaining -= this_round;
        }
    }
}

impl<'a, G: Game<N>, P: Policy<G, N>, const N: usize> MCTS<'a, G, P, N> {
    /// 获取下一个节点ID。
    fn next_node_id(&self) -> NodeId {
        self.nodes.len() as NodeId + self.offset
    }

    /// 获取指定节点ID的节点引用。
    fn node(&self, node_id: NodeId) -> &Node<G, N> {
        &self.nodes[(node_id - self.offset) as usize]
    }

    /// 获取指定节点ID的可变节点引用。
    fn mut_node(&mut self, node_id: NodeId) -> &mut Node<G, N> {
        &mut self.nodes[(node_id - self.offset) as usize]
    }

    /// 获取指定节点的所有子节点引用。
    fn children_of(&self, node: &Node<G, N>) -> &[Node<G, N>] {
        &self.nodes
            [(node.first_child - self.offset) as usize..(node.last_child() - self.offset) as usize]
    }

    /// 获取指定节点ID范围内的可变节点引用。
    fn mut_nodes(&mut self, first_child: NodeId, last_child: NodeId) -> &mut [Node<G, N>] {
        &mut self.nodes[(first_child - self.offset) as usize..(last_child - self.offset) as usize]
    }

    /// 根据搜索策略生成目标策略。
    pub fn target_policy(&self, search_policy: &mut [f32; N]) {
        search_policy.fill(0.0);
        let mut total = 0.0;
        let root = self.node(self.root);
        if root.num_visits == 1.0 {
            // assert!(root.solution.is_some());
            match root.solution {
                Some(Outcome::Win(_)) => {
                    for child in self.children_of(root) {
                        let v = if let Some(Outcome::Lose(_)) = child.solution {
                            1.0
                        } else {
                            0.0
                        };
                        search_policy[child.action as usize] = v;
                        total += v;
                    }
                }
                _ => {
                    for child in self.children_of(root) {
                        search_policy[child.action as usize] = 1.0;
                        total += 1.0;
                    }
                }
            }
        } else {
            // assert!(root.num_visits > 1.0);
            for child in self.children_of(root) {
                let v = child.num_visits;
                search_policy[child.action as usize] = v;
                total += v;
            }
        }
        // assert!(total > 0.0, "{:?} {:?}", root.solution, root.num_visits);
        // 使用 iter_mut() 代替索引访问
        for val in search_policy.iter_mut() {
            *val /= total;
        }
    }

    /// 获取根节点的目标Q值。
    pub fn target_q(&self) -> [f32; 3] {
        let root = self.node(self.root);
        match root.solution {
            Some(outcome) => outcome.into(), // From trait 自动提供 Into
            None => {
                let mut outcome_probs = [0.0; 3];
                // 使用 iter_mut().enumerate() 代替索引访问
                for (i, val) in outcome_probs.iter_mut().enumerate() {
                    *val = root.outcome_probs[i] / root.num_visits;
                }
                outcome_probs
            }
        }
    }

    /// 根据配置向根节点添加噪声。
    fn add_root_noise(&mut self) {
        match self.cfg.root_policy_noise {
            PolicyNoise::None => {}
            PolicyNoise::Equal { weight } => {
                self.add_equalizing_noise(weight);
            }
            PolicyNoise::Dirichlet { alpha, weight } => {
                self.add_dirichlet_noise(&mut rng(), alpha, weight);
            }
        }
    }

    /// 向根节点添加Dirichlet噪声。
    fn add_dirichlet_noise<R: Rng>(&mut self, rng: &mut R, alpha: f32, noise_weight: f32) {
        let root = self.node(self.root);
        if root.num_children < 2 {
            return;
        }
        let first_child = root.first_child;
        let last_child = root.last_child();
        let alphas = vec![alpha; root.num_children as usize];
        let dirichlet = Dirichlet::new(&alphas).unwrap();
        let noise_probs = dirichlet.sample(rng);
        for (noise, child) in noise_probs
            .iter()
            .zip(self.mut_nodes(first_child, last_child))
        {
            child.action_prob = child.action_prob * (1.0 - noise_weight) + noise_weight * noise;
        }
    }

    /// 向根节点添加等化噪声。
    fn add_equalizing_noise(&mut self, noise_weight: f32) {
        let root = self.node(self.root);
        if root.num_children < 2 {
            return;
        }
        let first_child = root.first_child;
        let last_child = root.last_child();
        let noise = 1.0 / root.num_children as f32;
        for child in self.mut_nodes(first_child, last_child) {
            child.action_prob = child.action_prob * (1.0 - noise_weight) + noise_weight * noise;
        }
    }

    /// 根据指定的动作选择策略返回最佳动作。
    pub fn best_action(&self, action_selection: ActionSelection) -> G::Action {
        let root = self.node(self.root);
        if self.cfg.mate_search_depth > 0 {
            if let Some(action) = self.best_mate_action(root, self.cfg.mate_search_depth) {
                return action;
            }
        }
        let mut rng = rng();

        let mut best_action = None;
        let mut best_value = None;
        for child in self.children_of(root) {
            let value = match child.solution {
                Some(Outcome::Win(turns)) => Some((0.0, turns as f32)),
                None => match action_selection {
                    ActionSelection::Q => Some((1.0, -child.q_with_contempt(self.cfg.contempt))),
                    ActionSelection::NumVisits => Some((1.0, child.num_visits)),
                    ActionSelection::Gumbel { scale } => {
                        let prior_term = child.action_prob.max(1e-8).ln();
                        let gumbel = sample_gumbel(&mut rng, scale.max(0.0));
                        Some((
                            1.0,
                            -child.q_with_contempt(self.cfg.contempt) + prior_term + gumbel,
                        ))
                    }
                },
                Some(Outcome::Draw(turns)) => {
                    Some((2.0, -(turns as f32) - self.cfg.contempt.abs()))
                }
                Some(Outcome::Lose(turns)) => Some((3.0, -(turns as f32))),
            };
            if value > best_value {
                best_value = value;
                best_action = Some(child.action());
            }
        }
        best_action.unwrap()
    }

    fn best_mate_action(&self, root: &Node<G, N>, depth: u8) -> Option<G::Action> {
        let mut best_action = None;
        let mut best_outcome = None;
        for child in self.children_of(root) {
            let outcome = if let Some(solution) = child.solution {
                Some(solution.reversed())
            } else {
                forced_outcome_from(&child.game, depth.saturating_sub(1)).map(|o| o.reversed())
            };

            if let Some(outcome) = outcome {
                if !matches!(outcome, Outcome::Win(_)) {
                    continue;
                }
                if best_outcome.is_none() || Some(outcome) > best_outcome {
                    best_outcome = Some(outcome);
                    best_action = Some(child.action());
                }
            }
        }
        best_action
    }

    /// 获取指定动作的解决方案。
    pub fn solution(&self, action: &G::Action) -> Option<Outcome> {
        let action: usize = (*action).into();
        let action: ActionId = action.try_into().expect("action id exceeds ActionId");
        let root = self.node(self.root);
        for child in self.children_of(root) {
            if child.action == action {
                return child.solution;
            }
        }
        None
    }

    fn explore_batch(&mut self, n: usize) {
        let mut tasks = Vec::with_capacity(n);
        for _ in 0..n {
            if self.node(self.root).solution.is_some() {
                break;
            }
            tasks.push(self.prepare_explore_task());
        }

        let mut eval_indices = Vec::new();
        let mut eval_games = Vec::new();
        for (i, task) in tasks.iter().enumerate() {
            match task {
                ExploreTask::BackpropNow {
                    node_id,
                    outcome_probs,
                    solved,
                } => {
                    self.backprop(*node_id, *outcome_probs, *solved);
                }
                ExploreTask::NeedEval { game, .. } => {
                    eval_indices.push(i);
                    eval_games.push(game.clone());
                }
            }
        }

        if eval_games.is_empty() {
            return;
        }

        let eval_results = self.policy.eval_batch(&eval_games);
        for (result_i, task_i) in eval_indices.into_iter().enumerate() {
            let (logits, outcome_probs) = eval_results[result_i];
            if let ExploreTask::NeedEval {
                node_id,
                first_child,
                last_child,
                any_solved,
                ..
            } = tasks[task_i]
            {
                self.apply_logits(first_child, last_child, &logits);
                self.backprop(node_id, outcome_probs, any_solved);
            }
        }
    }

    fn prepare_explore_task(&mut self) -> ExploreTask<G, N> {
        let mut node_id = self.root;
        loop {
            let node = self.node(node_id);
            if let Some(outcome) = node.solution {
                return ExploreTask::BackpropNow {
                    node_id,
                    outcome_probs: outcome.into(),
                    solved: true,
                };
            } else if node.is_unvisited() {
                let (expanded_node_id, maybe_eval) = self.expand_unvisited(node_id);
                match maybe_eval {
                    Some((game, first_child, last_child, any_solved)) => {
                        return ExploreTask::NeedEval {
                            node_id: expanded_node_id,
                            game,
                            first_child,
                            last_child,
                            any_solved,
                        };
                    }
                    None => {
                        node_id = expanded_node_id;
                        continue;
                    }
                }
            } else {
                node_id = self.select_best_child(node);
            }
        }
    }

    fn expand_unvisited(
        &mut self,
        node_id: NodeId,
    ) -> (NodeId, Option<(G, NodeId, NodeId, bool)>) {
        let first_child = self.next_node_id();
        let node = self.node(node_id);
        if let Some(_outcome) = node.solution {
            return (node_id, Some((node.game.clone(), first_child, first_child, true)));
        }

        let game = node.game.clone();
        let mut num_children = 0;
        let mut any_solved = false;
        for action in game.iter_actions() {
            let mut child_game = game.clone();
            let is_over = child_game.step(&action);
            let solution = if is_over {
                any_solved = true;
                Some(child_game.reward(child_game.player()).into())
            } else {
                None
            };
            let action: usize = action.into();
            let child = Node::unvisited(
                node_id,
                child_game,
                solution,
                action.try_into().expect("action id exceeds ActionId"),
                1.0,
            );
            self.nodes.push(child);
            num_children += 1;
        }

        let node = self.mut_node(node_id);
        node.mark_visited(first_child, num_children);
        let first = node.first_child;
        let last = node.last_child();

        if self.cfg.auto_extend && num_children == 1 {
            (first, None)
        } else {
            (node_id, Some((game, first, last, any_solved)))
        }
    }

    fn apply_logits(&mut self, first_child: NodeId, last_child: NodeId, logits: &[f32; N]) {
        let mut max_logit = f32::NEG_INFINITY;
        for child in self.mut_nodes(first_child, last_child) {
            let logit = logits[child.action as usize];
            max_logit = max_logit.max(logit);
            child.action_prob = logit;
        }
        let mut total = 0.0;
        for child in self.mut_nodes(first_child, last_child) {
            child.action_prob = (child.action_prob - max_logit).exp();
            total += child.action_prob;
        }
        for child in self.mut_nodes(first_child, last_child) {
            child.action_prob /= total;
        }
    }

    /// 选择最佳子节点。
    fn select_best_child(&self, parent: &Node<G, N>) -> NodeId {
        let mut best_child_id = None;
        let mut best_value = None;
        for child_id in parent.first_child..parent.last_child() {
            let child = self.node(child_id);
            let q = self.exploit_value(parent, child);
            let u = self.explore_value(parent, child);
            let value = Some(q + u);
            if value > best_value {
                best_child_id = Some(child_id);
                best_value = value;
            }
        }
        best_child_id.unwrap()
    }

    /// 计算利用价值。
    fn exploit_value(&self, parent: &Node<G, N>, child: &Node<G, N>) -> f32 {
        if let Some(outcome) = child.solution {
            if self.cfg.select_solved_nodes {
                let mut value = outcome.reversed().value();
                if matches!(outcome, Outcome::Draw(_)) {
                    value -= self.cfg.contempt.abs();
                }
                value
            } else {
                f32::NEG_INFINITY
            }
        } else if child.num_children == 0 {
            match self.cfg.fpu {
                Fpu::Const(value) => value,
                Fpu::ParentQ => parent.q_with_contempt(self.cfg.contempt),
                Fpu::Func(fpu_fn) => (fpu_fn)(),
            }
        } else {
            let mut q = -child.q_with_contempt(self.cfg.contempt);
            let prog_weight = self.cfg.progressive_simulation_weight.clamp(0.0, 1.0);
            if prog_weight > 0.0 {
                let k = self.cfg.progressive_simulation_visits.max(1) as f32;
                let alpha = (k / (k + child.num_visits)) * prog_weight;
                let prior_value = child.action_prob * 2.0 - 1.0;
                q = q * (1.0 - alpha) + prior_value * alpha;
            }
            q
        }
    }

    fn explore_value(&self, parent: &Node<G, N>, child: &Node<G, N>) -> f32 {
        match self.cfg.exploration {
            Exploration::Uct { c } => {
                let visits = (c * parent.num_visits.ln()).sqrt();
                visits / child.num_visits.sqrt()
            }
            Exploration::PolynomialUct { c } => {
                let visits = parent.num_visits.sqrt();
                c * child.action_prob * visits / (1.0 + child.num_visits)
            }
        }
    }
    /// 蒙特卡洛树搜索（MCTS）中的节点访问逻辑。主要功能如下：
    /// 检查当前节点是否有解，如果有则直接返回结果。
    /// 遍历所有可能的动作，生成子节点并检查是否游戏结束，更新子节点的状态。
    /// 如果只有一个子节点且配置允许自动扩展，则递归访问该子节点。
    /// 使用策略网络评估动作概率，并通过softmax计算每个子节点的动作概率。
    fn visit(&mut self, node_id: NodeId) -> (NodeId, [f32; 3], bool) {
        let first_child = self.next_node_id();
        let node = self.node(node_id);
        if let Some(outcome) = node.solution {
            return (node_id, outcome.into(), true);
        }

        let game = node.game.clone();
        let mut num_children = 0;
        let mut any_solved = false;
        for action in game.iter_actions() {
            let mut child_game = game.clone();
            let is_over = child_game.step(&action);
            let solution = if is_over {
                any_solved = true;
                Some(child_game.reward(child_game.player()).into())
            } else {
                None
            };
            let action: usize = action.into();
            let child = Node::unvisited(
                node_id,
                child_game,
                solution,
                action.try_into().expect("action id exceeds ActionId"),
                1.0,
            );
            self.nodes.push(child);
            num_children += 1;
        }

        let node = self.mut_node(node_id);
        node.mark_visited(first_child, num_children);
        let first_child = node.first_child;
        let last_child = node.last_child();

        if self.cfg.auto_extend && num_children == 1 {
            self.visit(first_child)
        } else {
            let (logits, outcome_probs) = self.policy.eval(&game);

            // stable softmax
            let mut max_logit = f32::NEG_INFINITY;
            for child in self.mut_nodes(first_child, last_child) {
                let logit = logits[child.action as usize];
                max_logit = max_logit.max(logit);
                child.action_prob = logit;
            }
            let mut total = 0.0;
            for child in self.mut_nodes(first_child, last_child) {
                child.action_prob = (child.action_prob - max_logit).exp();
                total += child.action_prob;
            }
            for child in self.mut_nodes(first_child, last_child) {
                child.action_prob /= total;
            }

            (node_id, outcome_probs, any_solved)
        }
    }
    ///该函数 backprop 用于反向传播从叶子节点到根节点的评估结果。主要功能包括：
    /// 更新节点的访问次数和结果概率。
    ///如果配置为解决模式且节点已解决，则检查子节点是否全部解决并更新最佳解。
    /// 根据最佳解调整结果概率。
    fn backprop(&mut self, leaf_node_id: NodeId, mut outcome_probs: [f32; 3], mut solved: bool) {
        let mut node_id = leaf_node_id;
        loop {
            let node = self.node(node_id);
            let parent = node.parent;

            if self.cfg.solve && solved {
                // compute whether all children are solved & best solution so far
                let mut all_solved = true;
                let mut best_solution = node.solution;
                for child in self.children_of(node) {
                    let soln = child.solution.map(|o| o.reversed());
                    all_solved &= soln.is_some();
                    best_solution = best_solution.max(soln);
                }

                let correct_values = self.cfg.correct_values_on_solve;
                let node = self.mut_node(node_id);
                if let Some(Outcome::Win(in_turns)) = best_solution {
                    // at least 1 is a win, so mark this node as a win
                    node.mark_solved(Outcome::Win(in_turns));
                    if correct_values {
                        // 使用 iter_mut().zip() 进行元素操作
                        for (dest, src) in outcome_probs.iter_mut().zip(node.outcome_probs.iter()) {
                            *dest = -*src;
                        }
                        outcome_probs[2] += node.num_visits + 1.0;
                    }
                } else if best_solution.is_some() && all_solved {
                    // all children node's are proven losses or draws
                    let best_outcome = best_solution.unwrap();
                    node.mark_solved(best_outcome);
                    if correct_values {
                        for (dest, src) in outcome_probs.iter_mut().zip(node.outcome_probs.iter()) {
                            *dest = -*src;
                        }
                        if let Outcome::Draw(_) = best_outcome {
                            outcome_probs[1] += node.num_visits + 1.0;
                        } else {
                            outcome_probs[0] += node.num_visits + 1.0;
                        }
                    }
                } else {
                    solved = false;
                }
            }

            let node = self.mut_node(node_id);
            // 使用 iter_mut().zip() 代替索引访问
            for (dest, src) in node.outcome_probs.iter_mut().zip(outcome_probs.iter()) {
                *dest += *src;
            }
            node.num_visits += 1.0;
            if node_id == self.root {
                break;
            }
            outcome_probs.swap(0, 2);
            node_id = parent;
        }
    }
}

fn sample_gumbel<R: Rng>(rng: &mut R, scale: f32) -> f32 {
    if scale <= 0.0 {
        return 0.0;
    }
    let u = rng.random::<f32>().clamp(f32::EPSILON, 1.0 - f32::EPSILON);
    -(-u.ln()).ln() * scale
}

fn forced_outcome_from<G: Game<N>, const N: usize>(game: &G, depth: u8) -> Option<Outcome> {
    if game.is_over() {
        return Some(game.reward(game.player()).into());
    }
    if depth == 0 {
        return None;
    }

    let mut best = None;
    for action in game.iter_actions() {
        let mut next = game.clone();
        let is_over = next.step(&action);
        let child = if is_over {
            Some(next.reward(next.player()).into())
        } else {
            forced_outcome_from::<G, N>(&next, depth - 1)
        };

        let current_perspective = child.map(|o| o.reversed());
        if current_perspective > best {
            best = current_perspective;
        }
        if matches!(best, Some(Outcome::Win(_))) {
            break;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use rand::prelude::{SeedableRng, StdRng};

    use super::*;
    use crate::synthesis::policies::RolloutPolicy;
    use crate::synthesis::HasTurnOrder;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, std::hash::Hash, PartialOrd, Ord)]
    pub enum PlayerId {
        X,
        O,
    }

    impl HasTurnOrder for PlayerId {
        fn prev(&self) -> Self {
            self.next()
        }

        fn next(&self) -> Self {
            match self {
                PlayerId::O => PlayerId::X,
                PlayerId::X => PlayerId::O,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct Action {
        row: usize,
        col: usize,
    }

    impl From<usize> for Action {
        fn from(i: usize) -> Self {
            let row = i / 3;
            let col = i % 3;
            Self { row, col }
        }
    }

    impl Into<usize> for Action {
        fn into(self) -> usize {
            self.row * 3 + self.col
        }
    }

    #[derive(Debug, PartialEq, Eq, std::hash::Hash, Clone)]
    struct TicTacToe {
        board: [[Option<PlayerId>; 3]; 3],
        player: PlayerId,
        turn: usize,
    }

    struct ActionIterator {
        game: TicTacToe,
        i: usize,
    }

    impl Iterator for ActionIterator {
        type Item = Action;

        fn next(&mut self) -> Option<Self::Item> {
            while self.i < 9 {
                let action: Action = self.i.into();
                self.i += 1;
                if self.game.board[action.row][action.col].is_none() {
                    return Some(action);
                }
            }

            None
        }
    }

    impl TicTacToe {
        fn won(&self, player: PlayerId) -> bool {
            let p = Some(player);
            if self.board[0][0] == p && self.board[0][1] == p && self.board[0][2] == p {
                return true;
            }
            if self.board[1][0] == p && self.board[1][1] == p && self.board[1][2] == p {
                return true;
            }
            if self.board[2][0] == p && self.board[2][1] == p && self.board[2][2] == p {
                return true;
            }
            if self.board[0][0] == p && self.board[1][0] == p && self.board[2][0] == p {
                return true;
            }
            if self.board[0][1] == p && self.board[1][1] == p && self.board[2][1] == p {
                return true;
            }
            if self.board[0][2] == p && self.board[1][2] == p && self.board[2][2] == p {
                return true;
            }
            if self.board[0][0] == p && self.board[1][1] == p && self.board[2][2] == p {
                return true;
            }
            if self.board[0][2] == p && self.board[1][1] == p && self.board[2][0] == p {
                return true;
            }

            false
        }
    }

    impl Game<9> for TicTacToe {
        type PlayerId = PlayerId;
        type Action = Action;
        type ActionIterator = ActionIterator;
        type Features = [[[f32; 3]; 3]; 3];

        const MAX_NUM_ACTIONS: usize = 9;
        const MAX_TURNS: usize = 9;
        const NAME: &'static str = "TicTacToe";
        const NUM_PLAYERS: usize = 2;
        const DIMS: &'static [i64] = &[3, 3, 3];

        fn new() -> Self {
            Self {
                board: [[None; 3]; 3],
                player: PlayerId::X,
                turn: 0,
            }
        }

        fn player(&self) -> Self::PlayerId {
            self.player
        }

        fn is_over(&self) -> bool {
            self.won(self.player) || self.won(self.player.prev()) || self.turn == 9
        }

        fn reward(&self, player_id: Self::PlayerId) -> f32 {
            if self.won(player_id) {
                1.0
            } else if self.won(player_id.next()) {
                -1.0
            } else {
                0.0
            }
        }

        fn iter_actions(&self) -> Self::ActionIterator {
            ActionIterator {
                game: self.clone(),
                i: 0,
            }
        }
        fn step(&mut self, action: &Self::Action) -> bool {
            assert!(action.row < 3);
            assert!(action.col < 3);
            assert!(self.board[action.row][action.col].is_none());
            self.board[action.row][action.col] = Some(self.player);
            self.player = self.player.next();
            self.turn += 1;
            self.is_over()
        }

        fn features(&self) -> Self::Features {
            let mut s = [[[0.0; 3]; 3]; 3];
            for row in 0..3 {
                for col in 0..3 {
                    if let Some(p) = self.board[row][col] {
                        if p == self.player {
                            s[0][row][col] = 1.0;
                        } else {
                            s[1][row][col] = 1.0;
                        }
                    } else {
                        s[2][row][col] = 1.0;
                    }
                }
            }
            s
        }

        fn print(&self) {
            for row in 0..3 {
                for col in 0..3 {
                    print!(
                        "{}",
                        match self.board[row][col] {
                            Some(PlayerId::X) => "x",
                            Some(PlayerId::O) => "o",
                            None => ".",
                        }
                    );
                }
                println!();
            }
            println!();
        }
    }

    // https://en.wikipedia.org/wiki/Tic-tac-toe

    #[test]
    fn test_solve_win() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut policy = RolloutPolicy { rng: &mut rng };
        let mut game = TicTacToe::new();
        game.step(&Action { row: 0, col: 0 });
        game.step(&Action { row: 0, col: 2 });
        let mut mcts = MCTS::with_capacity(
            1601,
            MCTSConfig {
                exploration: Exploration::PolynomialUct { c: 2.0 },
                solve: true,
                fpu: Fpu::Const(f32::INFINITY),
                select_solved_nodes: true,
                correct_values_on_solve: true,
                auto_extend: true,
                root_policy_noise: PolicyNoise::None,
                contempt: 0.0,
                mate_search_depth: 0,
                progressive_simulation_weight: 0.0,
                progressive_simulation_visits: 1,
                eval_batch_size: 8,
            },
            &mut policy,
            game.clone(),
        );
        while mcts.node(mcts.root).solution.is_none() {
            mcts.explore();
        }
        let mut search_policy = [0.0; 9];
        mcts.target_policy(&mut search_policy);
        // assert_eq!(mcts.node(mcts.root).solution, Some(Outcome::Win));
        // assert_eq!(
        //     &search_policy,
        //     &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
        // );
        assert_eq!(mcts.solution(&0.into()), None);
        assert_eq!(mcts.solution(&1.into()), None);
        assert_eq!(mcts.solution(&2.into()), None);
        assert_eq!(mcts.solution(&3.into()), None);
        assert_eq!(mcts.solution(&4.into()), None);
        assert_eq!(mcts.solution(&5.into()), None);
        // assert_eq!(mcts.solution(&6.into()), Some(Outcome::Lose));
        assert_eq!(mcts.solution(&7.into()), None);
        assert_eq!(mcts.solution(&8.into()), None);
        // assert_eq!(mcts.target_q(), 1.0);
        assert_eq!(mcts.best_action(ActionSelection::Q), 6.into());
        assert_eq!(mcts.nodes.len(), 311);
    }

    #[test]
    fn test_solve_loss() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut policy = RolloutPolicy { rng: &mut rng };
        let mut game = TicTacToe::new();
        game.step(&Action { row: 0, col: 0 });
        game.step(&Action { row: 0, col: 2 });
        game.step(&Action { row: 2, col: 0 });
        let mut mcts = MCTS::with_capacity(
            1601,
            MCTSConfig {
                exploration: Exploration::PolynomialUct { c: 2.0 },
                solve: true,
                correct_values_on_solve: true,
                fpu: Fpu::Const(f32::INFINITY),
                select_solved_nodes: true,
                auto_extend: true,
                root_policy_noise: PolicyNoise::None,
                contempt: 0.0,
                mate_search_depth: 0,
                progressive_simulation_weight: 0.0,
                progressive_simulation_visits: 1,
                eval_batch_size: 8,
            },
            &mut policy,
            game.clone(),
        );
        while mcts.node(mcts.root).solution.is_none() {
            mcts.explore();
        }
        // assert_eq!(mcts.node(mcts.root).solution, Some(Outcome::Lose));
        let mut search_policy = [0.0; 9];
        mcts.target_policy(&mut search_policy);
        // assert_eq!(
        //     &search_policy,
        //     &[
        //         0.0, 0.16666667, 0.0, 0.16666667, 0.16666667, 0.16666667, 0.0, 0.16666667,
        //         0.16666667
        //     ]
        // );
        // assert_eq!(mcts.solution(&0.into()), None);
        // assert_eq!(mcts.solution(&1.into()), Some(Outcome::Win));
        // assert_eq!(mcts.solution(&2.into()), None);
        // assert_eq!(mcts.solution(&3.into()), Some(Outcome::Win));
        // assert_eq!(mcts.solution(&4.into()), Some(Outcome::Win));
        // assert_eq!(mcts.solution(&5.into()), Some(Outcome::Win));
        // assert_eq!(mcts.solution(&6.into()), None);
        // assert_eq!(mcts.solution(&7.into()), Some(Outcome::Win));
        // assert_eq!(mcts.solution(&8.into()), Some(Outcome::Win));
        // assert_eq!(mcts.target_q(), -1.0);
        // assert_eq!(mcts.best_action(ActionSelection::Q), 1.into());
        assert_eq!(mcts.nodes.len(), 69);
    }

    #[test]
    fn test_solve_draw() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut policy = RolloutPolicy { rng: &mut rng };
        let mut game = TicTacToe::new();
        game.step(&Action { row: 0, col: 0 });
        game.step(&Action { row: 1, col: 1 });
        let mut mcts = MCTS::with_capacity(
            1601,
            MCTSConfig {
                exploration: Exploration::PolynomialUct { c: 2.0 },
                solve: true,
                correct_values_on_solve: true,
                fpu: Fpu::Const(f32::INFINITY),
                select_solved_nodes: true,
                auto_extend: true,
                root_policy_noise: PolicyNoise::None,
                contempt: 0.0,
                mate_search_depth: 0,
                progressive_simulation_weight: 0.0,
                progressive_simulation_visits: 1,
                eval_batch_size: 8,
            },
            &mut policy,
            game.clone(),
        );
        while mcts.node(mcts.root).solution.is_none() {
            mcts.explore();
        }

        // assert_eq!(mcts.node(mcts.root).solution, Some(Outcome::Draw));
        let mut search_policy = [0.0; 9];
        mcts.target_policy(&mut search_policy);
        // assert_eq!(
        //     &search_policy,
        //     &[
        //         0.0, 0.14285715, 0.14285715, 0.14285715, 0.0, 0.14285715, 0.14285715, 0.14285715,
        //         0.14285715
        //     ]
        // );
        assert_eq!(mcts.solution(&0.into()), None);
        // assert_eq!(mcts.solution(&1.into()), Some(Outcome::Draw));
        // assert_eq!(mcts.solution(&2.into()), Some(Outcome::Draw));
        // assert_eq!(mcts.solution(&3.into()), Some(Outcome::Draw));
        // assert_eq!(mcts.solution(&4.into()), None);
        // assert_eq!(mcts.solution(&5.into()), Some(Outcome::Draw));
        // assert_eq!(mcts.solution(&6.into()), Some(Outcome::Draw));
        // assert_eq!(mcts.solution(&7.into()), Some(Outcome::Draw));
        // assert_eq!(mcts.solution(&8.into()), Some(Outcome::Draw));
        // assert_eq!(mcts.target_q(), 0.0);
        assert_eq!(mcts.best_action(ActionSelection::Q), 1.into());
        assert_eq!(mcts.nodes.len(), 1533);
    }

    #[test]
    fn test_add_noise() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut policy = RolloutPolicy { rng: &mut rng };
        let game = TicTacToe::new();
        let mut mcts = MCTS::with_capacity(
            1601,
            MCTSConfig {
                exploration: Exploration::PolynomialUct { c: 2.0 },
                solve: true,
                correct_values_on_solve: true,
                fpu: Fpu::Const(f32::INFINITY),
                select_solved_nodes: false,
                auto_extend: false,
                root_policy_noise: PolicyNoise::None,
                contempt: 0.0,
                mate_search_depth: 0,
                progressive_simulation_weight: 0.0,
                progressive_simulation_visits: 1,
                eval_batch_size: 8,
            },
            &mut policy,
            game.clone(),
        );
        let mut rng2 = StdRng::seed_from_u64(0);

        let mut total = 0.0;
        for child in mcts.children_of(mcts.node(mcts.root)) {
            assert!(child.action_prob > 0.0);
            total += child.action_prob;
        }
        assert!((total - 1.0).abs() < 1e-6);

        mcts.add_dirichlet_noise(&mut rng2, 1.0, 0.25);
        let mut total = 0.0;
        for child in mcts.children_of(mcts.node(mcts.root)) {
            assert!(child.action_prob > 0.0);
            total += child.action_prob;
        }
        assert!((total - 1.0).abs() < 1e-6);
    }
}
