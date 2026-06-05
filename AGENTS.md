# Repository Guidelines

## Project Overview

**aichess** is a Chinese Chess (中国象棋) AI engine written in Rust, implementing the AlphaZero algorithm (MCTS + deep neural network) using the [Burn](https://github.com/tracel-ai/burn) deep learning framework with WGPU GPU acceleration.

---

## Project Structure

```
aichess/
├── src/
│   ├── lib.rs              # Crate root; re-exports all public modules
│   ├── cchess.rs           # Core Chinese Chess game logic
│   ├── net.rs              # Neural network model definition (Net, NetConfig)
│   ├── pos/                # Board position, FEN parsing, move generation
│   │   ├── mod.rs
│   │   ├── position.rs     # Position representation
│   │   ├── moves.rs        # Move struct & Chinese notation (Display)
│   │   ├── fen.rs          # FEN encoding/decoding
│   │   └── pregen.rs       # Pre-generated move tables
│   ├── synthesis/          # AlphaZero training pipeline
│   │   ├── alpha_zero.rs   # Self-play training loop
│   │   ├── mcts.rs         # Monte Carlo Tree Search
│   │   ├── evaluator.rs    # Neural network evaluator
│   │   ├── data.rs         # Training data management & replay buffer
│   │   ├── config.rs       # Hyperparameter configuration
│   │   ├── game.rs         # Game runner
│   │   ├── pgn.rs          # PGN file read/write
│   │   └── policies/       # Policy traits, rollout, and caching
│   ├── gui/                # egui-based desktop GUI
│   │   ├── app.rs
│   │   ├── views.rs
│   │   └── widgets.rs
│   ├── tests/              # Integration test data
│   └── bin/
│       ├── cli.rs          # `aichess-cli` – training & play CLI
│       ├── gui.rs          # `aichess-gui` – desktop GUI entry point
│       └── fen2.rs         # `fen2svg` – FEN-to-SVG renderer
├── logs/                   # Default training output (models, metrics)
├── Cargo.toml
└── readme.md
```

---

## Build, Test, and Development Commands

| Command | Description |
|---|---|
| `cargo build` | Debug build of all targets |
| `cargo build --release` | Optimised release build (`opt-level=3`, LTO enabled) |
| `cargo build --bin aichess-cli --release` | Build the CLI tool only |
| `cargo build --bin aichess-gui --release` | Build the GUI tool only |
| `cargo run --bin aichess-cli -- train -i 1 -g 2 -d ./test_logs` | Quick smoke-test training run |
| `cargo run --bin aichess-gui` | Launch the desktop GUI |
| `cargo test` | Run all unit and integration tests |
| `cargo test <module>` | Run tests in a specific module, e.g. `cargo test pos` |
| `cargo clippy` | Lint the codebase |
| `cargo fmt` | Auto-format all source files |

> **Note:** Training requires a WGPU-compatible GPU. For CPU-only testing, reduce `--num-explores` and `--games-per-train` to keep runs short.

---

## Coding Style & Naming Conventions

- **Formatter:** `rustfmt` with default settings. Run `cargo fmt` before committing.
- **Linter:** `cargo clippy` — fix all warnings before opening a PR.
- **Naming:** Follow standard Rust conventions: `snake_case` for functions/variables/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- **Error handling:** Use `anyhow::Result` for application-level errors; avoid `.unwrap()` in library code.
- **Module visibility:** Keep items `pub` only when required by external consumers.
- The crate sets `#![recursion_limit = "4096"]` in `lib.rs`; do not lower this value.

---

## Testing Guidelines

- Tests live in `src/tests/` (integration data) and in inline `#[cfg(test)]` modules within each source file.
- The project uses **rstest** for parameterised test cases.
- Name test functions descriptively: `test_<what>_<scenario>`, e.g. `test_move_display_red_cannon`.
- Run the full suite with `cargo test`; run a focused subset with `cargo test <name_fragment>`.
- There is no enforced coverage threshold, but new logic in `pos/` and `synthesis/` should include at least one test.

---

## Commit & Pull Request Guidelines

Commit messages in this project follow a **Conventional Commits** style:

```
<type>(<scope>): <short summary>
```

Common types: `feat`, `fix`, `refactor`, `docs`, `chore`, `perf`.  
Common scopes: `core`, `cli`, `gui`, `cchess`, `synthesis`, `pos`.

Examples from history:
```
feat(cli): 添加AIChess命令行工具及训练可视化功能
refactor(core): 优化代码质量并集成GUI框架
docs(readme): 添加 Move 中文记谱格式文档
```

**Pull Request checklist:**
- `cargo fmt` and `cargo clippy` pass with no warnings.
- `cargo test` passes.
- PR description explains *what* changed and *why*.
- Link any related issues.

---

## Architecture Notes

- **AlphaZero loop** (`synthesis/alpha_zero.rs`): self-play → data collection → neural network training → repeat.
- **Neural network** (`net.rs`): residual CNN implemented with Burn; backend is WGPU.
- **MCTS** (`synthesis/mcts.rs`): policy/value guided tree search; exploration count is configurable via `--num-explores`.
- **Saved models** are written to `<log_dir>/models/` as `model_N.mpk` (Burn's MessagePack format). Training resumes automatically from the latest checkpoint when the same `--log-dir` is reused.
