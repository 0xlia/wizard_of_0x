use axum::{
    Json, Router, debug_handler,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, header, request::Parts},
    response::IntoResponse,
    routing::{get, post},
};
use dashmap::DashMap;
use gamelogic::gamelogic::{
    AddPlayerError, Card, GameEvent, GamePhase, PlayCardError, Player, PredictionError, Suit,
    WizardGame, legal_card_indices,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone)]
pub struct AppState {
    pub gamestates: Arc<DashMap<String, WizardGame>>,
    pub broadcasters: Arc<DashMap<String, broadcast::Sender<()>>>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            gamestates: Arc::new(DashMap::new()),
            broadcasters: Arc::new(DashMap::new()),
        }
    }

    fn broadcaster_for(&self, id: &str) -> broadcast::Sender<()> {
        self.broadcasters
            .entry(id.to_owned())
            .or_insert_with(|| broadcast::channel(16).0)
            .clone()
    }

    fn pulse(&self, id: &str) {
        let _ = self.broadcaster_for(id).send(());
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateInput {
    pub playername: String,
    pub num_players: usize,
}

#[derive(Deserialize, Debug)]
pub struct JoinInput {
    pub playername: String,
}

#[derive(Deserialize, Debug)]
pub struct PredictInput {
    pub player: String,
    pub prediction: usize,
}

#[derive(Deserialize, Debug)]
pub struct PlayInput {
    pub player: String,
    pub card_index: usize,
}

#[derive(Deserialize, Debug)]
pub struct ChooseTrumpfInput {
    pub player: String,
    pub suit: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub name: String,
    pub is_ai: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LobbyView {
    pub id: String,
    pub num_players: usize,
    pub players: Vec<PlayerView>,
    pub started: bool,
}

impl LobbyView {
    fn from_game(id: String, game: &WizardGame) -> Self {
        Self {
            id,
            num_players: game.num_players(),
            players: game.players().iter().map(player_view).collect(),
            started: game.is_started(),
        }
    }
}

fn player_view(p: &Player) -> PlayerView {
    PlayerView {
        name: p.name().to_owned(),
        is_ai: p.is_bot(),
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CardView {
    pub suit: String,
    pub value: usize,
}

impl From<&Card> for CardView {
    fn from(c: &Card) -> Self {
        Self {
            suit: suit_name(&c.suit).into(),
            value: c.value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PlayedCard {
    pub player_index: usize,
    pub card: CardView,
}

fn suit_name(s: &Suit) -> &'static str {
    match s {
        Suit::Red => "red",
        Suit::Yellow => "yellow",
        Suit::Green => "green",
        Suit::Blue => "blue",
    }
}

fn phase_name(p: &GamePhase) -> &'static str {
    match p {
        GamePhase::NotStarted => "not_started",
        GamePhase::ChooseTrumpf => "choose_trumpf",
        GamePhase::Prediction => "prediction",
        GamePhase::Playing => "playing",
        GamePhase::RoundEnd => "round_end",
        GamePhase::GameOver => "game_over",
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventView {
    RoundStarted {
        round: usize,
        dealer: usize,
        flipped_card: Option<CardView>,
    },
    TrumpfChosen {
        chooser: usize,
        suit: String,
    },
    Predicted {
        player: usize,
        prediction: usize,
    },
    CardPlayed {
        player: usize,
        card: CardView,
    },
    TrickWon {
        winner: usize,
    },
    RoundScored {
        round: usize,
        deltas: Vec<i32>,
    },
    GameOver,
}

impl From<&GameEvent> for EventView {
    fn from(e: &GameEvent) -> Self {
        match e {
            GameEvent::RoundStarted {
                round,
                dealer,
                flipped_card,
            } => Self::RoundStarted {
                round: *round,
                dealer: *dealer,
                flipped_card: flipped_card.as_ref().map(CardView::from),
            },
            GameEvent::TrumpfChosen { chooser, suit } => Self::TrumpfChosen {
                chooser: *chooser,
                suit: suit_name(suit).into(),
            },
            GameEvent::Predicted { player, prediction } => Self::Predicted {
                player: *player,
                prediction: *prediction,
            },
            GameEvent::CardPlayed { player, card } => Self::CardPlayed {
                player: *player,
                card: CardView::from(card),
            },
            GameEvent::TrickWon { winner } => Self::TrickWon { winner: *winner },
            GameEvent::RoundScored { round, deltas } => Self::RoundScored {
                round: *round,
                deltas: deltas.clone(),
            },
            GameEvent::GameOver => Self::GameOver,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameView {
    pub lobby_id: String,
    pub phase: String,
    pub round: usize,
    pub max_rounds: usize,
    pub trumpf: Option<CardView>,
    pub current_player_index: usize,
    pub current_trick_starter: usize,
    pub current_dealer_index: usize,
    pub current_trick: Vec<PlayedCard>,
    pub players: Vec<PlayerView>,
    pub predictions: Vec<Option<usize>>,
    pub tricks_won: Vec<usize>,
    pub points: Vec<i32>,
    pub your_name: String,
    pub your_index: usize,
    pub your_hand: Vec<CardView>,
    pub your_legal_card_indices: Vec<usize>,
    pub log: Vec<EventView>,
}

impl GameView {
    fn for_player(id: &str, game: &WizardGame, player_name: &str) -> Option<Self> {
        let your_index = game
            .players()
            .iter()
            .position(|p| p.name() == player_name)?;
        let your_legal_card_indices = if *game.game_phase() == GamePhase::Playing
            && game.current_player_index() == your_index
        {
            legal_card_indices(game, your_index)
        } else {
            Vec::new()
        };
        Some(Self {
            lobby_id: id.to_owned(),
            phase: phase_name(game.game_phase()).into(),
            round: game.round_number(),
            max_rounds: game.max_rounds(),
            trumpf: game.trumpf().map(CardView::from),
            current_player_index: game.current_player_index(),
            current_trick_starter: game.current_trick_starter(),
            current_dealer_index: game.dealer_index(),
            current_trick: game
                .current_trick()
                .iter()
                .map(|(idx, card)| PlayedCard {
                    player_index: *idx,
                    card: CardView::from(card),
                })
                .collect(),
            players: game.players().iter().map(player_view).collect(),
            predictions: game.players().iter().map(|p| p.prediction()).collect(),
            tricks_won: game.players().iter().map(|p| p.tricks_won()).collect(),
            points: game.players().iter().map(|p| p.points()).collect(),
            your_name: player_name.to_owned(),
            your_index,
            your_hand: game.players()[your_index]
                .hand()
                .iter()
                .map(CardView::from)
                .collect(),
            your_legal_card_indices,
            log: game.event_log().iter().map(EventView::from).collect(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ErrorBody {
    pub error: String,
}

type ApiError = (StatusCode, Json<ErrorBody>);

fn err(status: StatusCode, msg: impl Into<String>) -> ApiError {
    (status, Json(ErrorBody { error: msg.into() }))
}

/// Returns `true` when the request's `Origin` host (and port) matches the
/// `Host` header, i.e. the request originates from the same origin we serve.
fn same_origin(origin: &header::HeaderValue, parts: &Parts) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    // Strip the scheme ("https://example.com:3000" -> "example.com:3000").
    let origin_host = origin.split_once("://").map_or(origin, |(_, rest)| rest);

    parts
        .headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|host| host == origin_host)
}

pub fn build_app(state: AppState) -> Router {
    // Only allow same-origin requests: the Origin header's host must match the
    // request's Host header. Cross-origin browsers are rejected; non-browser
    // clients (no Origin header) are unaffected since CORS only gates browsers.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(same_origin))
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/lobby", post(create_lobby))
        .route("/lobby/{id}", get(get_lobby))
        .route("/lobby/{id}/join", post(join_lobby))
        .route("/lobby/{id}/add_bot", post(add_bot_player))
        .route("/lobby/{id}/start", post(start_game))
        .route("/lobby/{id}/state", get(get_state))
        .route("/lobby/{id}/choose_trumpf", post(post_choose_trumpf))
        .route("/lobby/{id}/predict", post(post_prediction))
        .route("/lobby/{id}/play", post(post_play_card))
        .route("/lobby/{id}/ws", get(lobby_ws))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

#[debug_handler]
async fn create_lobby(
    State(state): State<AppState>,
    Json(input): Json<CreateInput>,
) -> Result<Json<LobbyView>, ApiError> {
    if !(3..=6).contains(&input.num_players) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "num_players must be between 3 and 6",
        ));
    }
    if input.playername.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "playername must not be empty"));
    }

    let new_game = WizardGame::new(input.num_players, input.playername);
    let id = loop {
        let candidate = WizardGame::create_id();
        if !state.gamestates.contains_key(&candidate) {
            break candidate;
        }
    };
    let view = LobbyView::from_game(id.clone(), &new_game);
    state.gamestates.insert(id.clone(), new_game);
    let _ = state.broadcaster_for(&id);
    Ok(Json(view))
}

#[debug_handler]
async fn get_lobby(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<LobbyView>, ApiError> {
    let game = state
        .gamestates
        .get(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;
    Ok(Json(LobbyView::from_game(id, &game)))
}

#[debug_handler]
async fn join_lobby(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<JoinInput>,
) -> Result<Json<LobbyView>, ApiError> {
    if input.playername.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "playername must not be empty"));
    }

    let view = {
        let mut game = state
            .gamestates
            .get_mut(&id)
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;

        let player = Player::new(input.playername);
        match game.add_player(player) {
            Ok(()) => LobbyView::from_game(id.clone(), &game),
            Err(AddPlayerError::LobbyFull { .. }) => {
                return Err(err(StatusCode::CONFLICT, "lobby is full"));
            }
            Err(AddPlayerError::NameTaken { name }) => {
                return Err(err(
                    StatusCode::CONFLICT,
                    format!("name '{name}' is already taken in this lobby"),
                ));
            }
        }
    };
    state.pulse(&id);
    Ok(Json(view))
}

#[debug_handler]
async fn add_bot_player(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<LobbyView>, ApiError> {
    let view = {
        let mut game = state
            .gamestates
            .get_mut(&id)
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;

        if game.is_started() {
            return Err(err(StatusCode::CONFLICT, "game already started"));
        }

        let existing: std::collections::HashSet<String> =
            game.players().iter().map(|p| p.name().to_owned()).collect();
        let name = (1..=usize::MAX)
            .map(|i| format!("BOT_{i}"))
            .find(|n| !existing.contains(n))
            .expect("infinite sequence has a free name");

        let player = Player::new_bot(name);
        match game.add_player(player) {
            Ok(()) => LobbyView::from_game(id.clone(), &game),
            Err(AddPlayerError::LobbyFull { .. }) => {
                return Err(err(StatusCode::CONFLICT, "lobby is full"));
            }
            Err(AddPlayerError::NameTaken { name }) => {
                return Err(err(
                    StatusCode::CONFLICT,
                    format!("name '{name}' is already taken in this lobby"),
                ));
            }
        }
    };
    state.pulse(&id);
    Ok(Json(view))
}

#[debug_handler]
async fn start_game(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<LobbyView>, ApiError> {
    let view = {
        let mut game = state
            .gamestates
            .get_mut(&id)
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;

        if game.is_started() {
            return Err(err(StatusCode::CONFLICT, "game already started"));
        }
        if !game.is_full() {
            return Err(err(
                StatusCode::CONFLICT,
                format!(
                    "lobby not full ({}/{})",
                    game.players().len(),
                    game.num_players()
                ),
            ));
        }

        game.start_game();
        drive_bot_actions(&mut game);
        LobbyView::from_game(id.clone(), &game)
    };
    state.pulse(&id);
    Ok(Json(view))
}

#[debug_handler]
async fn post_choose_trumpf(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<ChooseTrumpfInput>,
) -> Result<Json<GameView>, ApiError> {
    let suit = match input.suit.as_str() {
        "red" => Suit::Red,
        "yellow" => Suit::Yellow,
        "green" => Suit::Green,
        "blue" => Suit::Blue,
        _ => {
            return Err(err(
                StatusCode::BAD_REQUEST,
                "suit must be red/yellow/green/blue",
            ));
        }
    };

    let view = {
        let mut game = state
            .gamestates
            .get_mut(&id)
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;
        let player_index = game
            .players()
            .iter()
            .position(|p| p.name() == input.player)
            .ok_or_else(|| err(StatusCode::FORBIDDEN, "player not in this lobby"))?;
        if *game.game_phase() != GamePhase::ChooseTrumpf {
            return Err(err(StatusCode::CONFLICT, "not in choose-trumpf phase"));
        }
        if game.dealer_index() != player_index {
            return Err(err(StatusCode::CONFLICT, "you are not the trumpf chooser"));
        }
        game.choose_trumpf(suit);
        drive_bot_actions(&mut game);
        GameView::for_player(&id, &game, &input.player).expect("player exists; just looked it up")
    };
    state.pulse(&id);
    Ok(Json(view))
}

#[debug_handler]
async fn post_prediction(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<PredictInput>,
) -> Result<Json<GameView>, ApiError> {
    let view = {
        let mut game = state
            .gamestates
            .get_mut(&id)
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;
        let player_index = game
            .players()
            .iter()
            .position(|p| p.name() == input.player)
            .ok_or_else(|| err(StatusCode::FORBIDDEN, "player not in this lobby"))?;

        match game.place_prediction(player_index, input.prediction) {
            Ok(()) => {}
            Err(PredictionError::WrongPhase) => {
                return Err(err(StatusCode::CONFLICT, "not in prediction phase"));
            }
            Err(PredictionError::NotYourTurn) => {
                return Err(err(StatusCode::CONFLICT, "not your turn"));
            }
            Err(PredictionError::OutOfRange { max }) => {
                return Err(err(
                    StatusCode::BAD_REQUEST,
                    format!("prediction must be between 0 and {max}"),
                ));
            }
        }
        drive_bot_actions(&mut game);
        GameView::for_player(&id, &game, &input.player).expect("player exists; just looked it up")
    };
    state.pulse(&id);
    Ok(Json(view))
}

#[debug_handler]
async fn post_play_card(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<PlayInput>,
) -> Result<Json<GameView>, ApiError> {
    let view = {
        let mut game = state
            .gamestates
            .get_mut(&id)
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;
        let player_index = game
            .players()
            .iter()
            .position(|p| p.name() == input.player)
            .ok_or_else(|| err(StatusCode::FORBIDDEN, "player not in this lobby"))?;

        match game.play_card(player_index, input.card_index) {
            Ok(()) => {}
            Err(PlayCardError::WrongPhase) => {
                return Err(err(StatusCode::CONFLICT, "not in playing phase"));
            }
            Err(PlayCardError::NotYourTurn) => {
                return Err(err(StatusCode::CONFLICT, "not your turn"));
            }
            Err(PlayCardError::BadIndex { index, hand_size }) => {
                return Err(err(
                    StatusCode::BAD_REQUEST,
                    format!("card index {index} out of range (hand has {hand_size} cards)"),
                ));
            }
            Err(PlayCardError::MustFollowSuit) => {
                return Err(err(StatusCode::BAD_REQUEST, "must follow lead suit"));
            }
        }
        drive_bot_actions(&mut game);
        GameView::for_player(&id, &game, &input.player).expect("player exists; just looked it up")
    };
    state.pulse(&id);
    Ok(Json(view))
}

/// Drives every chained action where the current actor is a Bot. Loops across
/// ChooseTrumpf, Prediction, and Playing until a human is up or the game ends.
fn drive_bot_actions(game: &mut WizardGame) {
    loop {
        match game.game_phase().clone() {
            GamePhase::ChooseTrumpf => {
                let chooser = game.dealer_index();
                if !game.players()[chooser].is_bot() {
                    break;
                }
                game.choose_trumpf(random_suit());
            }
            GamePhase::Prediction => {
                let cur = game.current_player_index();
                if !game.players()[cur].is_bot() {
                    break;
                }
                let max = game.round_number();
                let prediction = rand::rng().random_range(0..=max);
                game.place_prediction(cur, prediction)
                    .expect("AI prediction within range and on the right player's turn");
            }
            GamePhase::Playing => {
                let cur = game.current_player_index();
                if !game.players()[cur].is_bot() {
                    break;
                }
                let legal = legal_card_indices(game, cur);
                let pick = legal[rand::rng().random_range(0..legal.len())];
                game.play_card(cur, pick).expect("AI plays a legal card");
            }
            GamePhase::NotStarted | GamePhase::RoundEnd | GamePhase::GameOver => break,
        }
    }
}

fn random_suit() -> Suit {
    let suits = [Suit::Red, Suit::Yellow, Suit::Green, Suit::Blue];
    let i = rand::rng().random_range(0..suits.len());
    suits[i].clone()
}

#[derive(Deserialize, Debug)]
struct PlayerQuery {
    player: String,
}

#[debug_handler]
async fn get_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<PlayerQuery>,
) -> Result<Json<GameView>, ApiError> {
    let game = state
        .gamestates
        .get(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;
    if !game.is_started() {
        return Err(err(StatusCode::CONFLICT, "game not started"));
    }
    GameView::for_player(&id, &game, &q.player)
        .map(Json)
        .ok_or_else(|| err(StatusCode::FORBIDDEN, "player not in this lobby"))
}

#[debug_handler]
async fn lobby_ws(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<PlayerQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    let game = state
        .gamestates
        .get(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "lobby not found"))?;
    if !game.players().iter().any(|p| p.name() == q.player) {
        return Err(err(StatusCode::FORBIDDEN, "player not in this lobby"));
    }
    drop(game);

    let state = state.clone();
    Ok(ws.on_upgrade(move |socket| handle_lobby_ws(socket, state, id, q.player)))
}

async fn handle_lobby_ws(mut socket: WebSocket, state: AppState, id: String, player: String) {
    let mut rx = state.broadcaster_for(&id).subscribe();
    send_snapshot(&mut socket, &state, &id, &player).await;

    loop {
        tokio::select! {
            biased;
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {} // ignore client text; actions go through REST
                    Some(Err(_)) => break,
                }
            }
            tick = rx.recv() => {
                match tick {
                    Ok(()) | Err(broadcast::error::RecvError::Lagged(_)) => {
                        send_snapshot(&mut socket, &state, &id, &player).await;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn send_snapshot(socket: &mut WebSocket, state: &AppState, id: &str, player: &str) {
    let payload = state.gamestates.get(id).and_then(|game| {
        GameView::for_player(id, &game, player)
            .map(|view| serde_json::json!({"type": "state", "state": view}).to_string())
    });
    if let Some(text) = payload {
        let _ = socket.send(Message::Text(text.into())).await;
    }
}
