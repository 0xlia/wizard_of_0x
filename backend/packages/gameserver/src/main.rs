use axum::{Json, Router, debug_handler, extract::{Path, State}, http::StatusCode, routing::{get, post}};
use dashmap::DashMap;
use serde::Deserialize;
use gamelogic::gamelogic::{Player, WizardGame};
use std::sync::Arc; 

#[derive(Debug, Clone)]
struct AppState {
    gamestates: Arc<DashMap<String, WizardGame>>,
}

impl AppState {
    fn new() -> Self {
        Self { gamestates: Arc::new(DashMap::new()) }
    }
}


#[derive(Deserialize, Debug)]
struct CreateInput {
    playername: String, 
    num_players: usize, 
}

#[derive(Deserialize, Debug)]
struct JoinInput {
    playername: String, 
}

#[derive(Deserialize, Debug)]
struct PathInput {
    id: String, 
}


// PUT /lobby POST /lobby/{id}/join oder PUT /game POST /game/{id}/join?

#[tokio::main]
async fn main() {

    let state = AppState::new();

    let app = Router::new()
    .route("/", get(root_handler))
    .route("/lobby", post(create_lobby))
    .route("/lobby/{id}/join", post(join_lobby)) 
    .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

#[debug_handler]
async fn root_handler() -> &'static str {
    "Hello World\n"
}

#[debug_handler]
async fn create_lobby(State(state): State<AppState>, Json(input): Json<CreateInput>) -> Json<String> {
    let new_game: WizardGame = WizardGame::new(input.num_players, input.playername);
    let id = WizardGame::create_id();
    state.gamestates.insert(id.clone(), new_game);
    Json(id)
}

#[debug_handler]
async fn join_lobby(State(state): State<AppState>, Path(path): Path<PathInput>, Json(input): Json<JoinInput>) -> Result<(), StatusCode> {
    let mut gamestate = state.gamestates.get_mut(&path.id).ok_or(StatusCode::NOT_FOUND)?;
    let player = Player::new(input.playername);
    gamestate.add_player(player).map_err(|_| StatusCode::BAD_REQUEST)?;
    dbg!(&*gamestate);
    Ok(())
}