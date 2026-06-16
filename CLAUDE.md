# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repo layout

- `backend/` — Cargo workspace (edition 2024, resolver 3). Workspace lints set `clippy::pedantic = "warn"`, so new code is expected to be pedantic-clean.
  - `packages/gamelogic/` — pure game-state library. No I/O, no async. The domain types (`WizardGame`, `Player`, `Card`, `Suit`, `GamePhase`, `AddPlayerError`) live in [backend/packages/gamelogic/src/gamelogic.rs](backend/packages/gamelogic/src/gamelogic.rs).
  - `packages/gameserver/` — Axum HTTP + WebSocket server that wraps `gamelogic`. The router and handlers live in [backend/packages/gameserver/src/lib.rs](backend/packages/gameserver/src/lib.rs); [main.rs](backend/packages/gameserver/src/main.rs) just wires up tracing, picks the frontend dir, and binds the listener.
- `frontend/` — static vanilla HTML/JS/CSS (no build step). Served by the gameserver as a `ServeDir` fallback. Entry point is [frontend/index.html](frontend/index.html) + [frontend/index.js](frontend/index.js).

## Common commands

All `cargo` commands run from `backend/`.

```bash
# Run the server (serves API + frontend on 0.0.0.0:3000)
cargo run -p gameserver

# Override the frontend directory (default "../frontend", resolved relative to cwd)
FRONTEND_DIR=/abs/path/to/frontend cargo run -p gameserver

# Tracing filter (default: "gameserver=debug,tower_http=info")
RUST_LOG=gameserver=trace cargo run -p gameserver

# Tests
cargo test                                  # all packages
cargo test -p gameserver                    # server only (includes integration tests)
cargo test -p gamelogic                     # gamelogic unit tests
cargo test -p gameserver --test lobby       # single integration test file
cargo test -p gameserver lobbies_are_isolated   # single test by name

# Lints (workspace enables clippy::pedantic at warn level)
cargo clippy --workspace --all-targets
cargo fmt --all
```

The frontend has no toolchain — open `http://localhost:3000` after starting the server and the `ServeDir` fallback serves it.

## Architecture

**Single in-process source of truth.** `AppState` holds an `Arc<DashMap<String, WizardGame>>` keyed by random 5-char lobby IDs ([build_app in lib.rs](backend/packages/gameserver/src/lib.rs)). Nothing is persisted — restarting the server drops all lobbies.

**Two-layer split:**
- `gamelogic` owns all rules and state transitions. It exposes mutators (`add_player`, `start_game`, `choose_trumpf`) and read-only accessors. It returns typed errors (`AddPlayerError`); it never knows about HTTP.
- `gameserver` translates between HTTP/JSON and `gamelogic`. The `*View` structs (`LobbyView`, `GameView`, `CardView`, `PlayerView`) are the wire format and convert enums to lowercase strings (`Suit::Red` → `"red"`, `GamePhase::ChooseTrumpf` → `"choose_trumpf"`). Domain types are deliberately not `Serialize` — views exist to keep the wire shape stable while the domain evolves.

**Game phases.** `GamePhase` is `NotStarted → ChooseTrumpf? → Prediction → Playing → RoundEnd → GameOver`. `ChooseTrumpf` is only entered when the revealed trumpf card is a Wizard (value 14); otherwise rounds go straight to `Prediction`. Today only `NotStarted → ChooseTrumpf/Prediction` is wired through to the server — Prediction/Playing/RoundEnd action endpoints don't exist yet.

**AI players.** Added via `POST /lobby/{id}/add_ai`, named `BOT_1`, `BOT_2`, …. After every state-mutating handler that could leave an AI as the current actor, the server calls `drive_ai_actions` to auto-resolve their turns. Right now that only covers `ChooseTrumpf` (picks a random suit); extend this function when adding new phase actions so AI turns don't stall the game.

**Realtime.** `GET /lobby/{id}/ws?player=<name>` upgrades to a WebSocket. On connect, the server sends one `{"type":"state","state":GameView}` snapshot for that player and then holds the socket open. There is no broadcast/push loop yet — the frontend currently polls `GET /lobby/{id}` on a 2s timer during the lobby phase and only opens the socket once `started` flips true. When adding action endpoints, broadcast updated `GameView`s through the existing socket rather than adding more polling.

**CORS** is wide-open (`Any` origin/methods/headers) because the frontend is also served by the same process in dev, but external clients may hit it during development.

## Testing notes

Integration tests in [backend/packages/gameserver/tests/lobby.rs](backend/packages/gameserver/tests/lobby.rs) drive the router via `tower::ServiceExt::oneshot` against an in-memory `AppState` — no network. Use the existing `send::<T>`, `json_request`, `create_lobby`, and `fill_lobby` helpers when adding new endpoint tests; they keep tests one-liner-ish and consistent.

`gamelogic` has `#[cfg(test)]` unit tests inline in [gamelogic.rs](backend/packages/gamelogic/src/gamelogic.rs) for pure-state behavior (player adds, dealing). Prefer adding logic tests there and HTTP-shape tests in the gameserver integration file.
