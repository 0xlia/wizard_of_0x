use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use gameserver::{AppState, ErrorBody, EventView, GameView, LobbyView, PlayerView, build_app};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tower::ServiceExt;

fn app() -> Router {
    build_app(AppState::new())
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn send<T: DeserializeOwned>(app: &Router, req: Request<Body>) -> (StatusCode, T) {
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let parsed = serde_json::from_slice::<T>(&bytes)
        .unwrap_or_else(|e| panic!("decode failed ({e}): {}", String::from_utf8_lossy(&bytes)));
    (status, parsed)
}

async fn create_lobby(app: &Router, name: &str, n: usize) -> LobbyView {
    let (status, view) = send::<LobbyView>(
        app,
        json_request(
            Method::POST,
            "/lobby",
            json!({ "playername": name, "num_players": n }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    view
}

fn names(view: &LobbyView) -> Vec<String> {
    view.players.iter().map(|p| p.name.clone()).collect()
}

#[tokio::test]
async fn create_lobby_returns_view() {
    let app = app();
    let view = create_lobby(&app, "Nyuchen", 3).await;
    assert_eq!(view.num_players, 3);
    assert_eq!(view.players.len(), 1);
    assert_eq!(view.players[0].name, "Nyuchen");
    assert!(!view.players[0].is_ai);
    assert!(!view.started);
    assert!(!view.id.is_empty());
}

#[tokio::test]
async fn create_lobby_validates_num_players() {
    let app = app();
    for n in [0usize, 1, 2, 7, 100] {
        let (status, body) = send::<ErrorBody>(
            &app,
            json_request(
                Method::POST,
                "/lobby",
                json!({ "playername": "x", "num_players": n }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "n={n} should be rejected");
        assert!(body.error.contains("num_players"));
    }
}

#[tokio::test]
async fn create_lobby_rejects_empty_name() {
    let app = app();
    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            "/lobby",
            json!({ "playername": "  ", "num_players": 3 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.error.contains("playername"));
}

#[tokio::test]
async fn get_lobby_reflects_join() {
    let app = app();
    let created = create_lobby(&app, "host", 3).await;

    let (status, joined) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/join", created.id),
            json!({ "playername": "guest" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(names(&joined), vec!["host", "guest"]);
    assert!(joined.players.iter().all(|p| !p.is_ai));

    let (status, fetched) =
        send::<LobbyView>(&app, get_request(&format!("/lobby/{}", created.id))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched, joined);
}

#[tokio::test]
async fn get_unknown_lobby_returns_404() {
    let (status, body) = send::<ErrorBody>(&app(), get_request("/lobby/DOESNT")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.error, "lobby not found");
}

#[tokio::test]
async fn join_unknown_lobby_returns_404() {
    let (status, body) = send::<ErrorBody>(
        &app(),
        json_request(
            Method::POST,
            "/lobby/MISSING/join",
            json!({ "playername": "x" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.error, "lobby not found");
}

#[tokio::test]
async fn join_rejects_empty_name() {
    let app = app();
    let created = create_lobby(&app, "host", 3).await;
    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/join", created.id),
            json!({ "playername": "" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.error.contains("playername"));
}

#[tokio::test]
async fn join_rejects_duplicate_name() {
    let app = app();
    let created = create_lobby(&app, "twin", 3).await;
    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/join", created.id),
            json!({ "playername": "twin" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.error.contains("twin"));
}

#[tokio::test]
async fn join_rejects_when_full() {
    let app = app();
    let created = create_lobby(&app, "a", 3).await;
    for name in ["b", "c"] {
        let (status, _) = send::<LobbyView>(
            &app,
            json_request(
                Method::POST,
                &format!("/lobby/{}/join", created.id),
                json!({ "playername": name }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/join", created.id),
            json!({ "playername": "d" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "lobby is full");
}

#[tokio::test]
async fn lobbies_are_isolated() {
    let app = app();
    let lobby_a = create_lobby(&app, "alpha", 3).await;
    let lobby_b = create_lobby(&app, "bravo", 4).await;
    assert_ne!(lobby_a.id, lobby_b.id);

    let (status, joined) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/join", lobby_a.id),
            json!({ "playername": "alpha2" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(joined.players.len(), 2);

    let (_, fetched_b) =
        send::<LobbyView>(&app, get_request(&format!("/lobby/{}", lobby_b.id))).await;
    assert_eq!(names(&fetched_b), vec!["bravo"]);
    assert_eq!(fetched_b.num_players, 4);
}

#[tokio::test]
async fn add_ai_marks_player_as_ai_and_increments_slot() {
    let app = app();
    let created = create_lobby(&app, "human", 3).await;

    let (status, after_one) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/add_ai", created.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(after_one.players.len(), 2);
    assert_eq!(
        after_one.players[1],
        PlayerView {
            name: "BOT_1".into(),
            is_ai: true
        }
    );

    let (status, after_two) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/add_ai", created.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        after_two.players[2],
        PlayerView {
            name: "BOT_2".into(),
            is_ai: true
        }
    );
}

#[tokio::test]
async fn add_ai_rejected_when_full() {
    let app = app();
    let full = fill_lobby(&app, 3).await;
    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/add_ai", full.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "lobby is full");
}

#[tokio::test]
async fn add_ai_rejected_after_start() {
    let app = app();
    let full = fill_lobby(&app, 3).await;
    let (status, _) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", full.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/add_ai", full.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "game already started");
}

#[tokio::test]
async fn lobby_filled_with_ais_can_start() {
    let app = app();
    let created = create_lobby(&app, "solo", 3).await;
    for _ in 0..2 {
        let (status, _) = send::<LobbyView>(
            &app,
            json_request(
                Method::POST,
                &format!("/lobby/{}/add_ai", created.id),
                json!({}),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, started) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", created.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(started.started);
    assert_eq!(started.players.iter().filter(|p| p.is_ai).count(), 2);
    assert_eq!(started.players.iter().filter(|p| !p.is_ai).count(), 1);
}

async fn fill_lobby(app: &Router, n: usize) -> LobbyView {
    let mut lobby = create_lobby(app, "p0", n).await;
    for i in 1..n {
        let (status, view) = send::<LobbyView>(
            app,
            json_request(
                Method::POST,
                &format!("/lobby/{}/join", lobby.id),
                json!({ "playername": format!("p{i}") }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        lobby = view;
    }
    lobby
}

#[tokio::test]
async fn start_game_succeeds_when_full() {
    let app = app();
    let full = fill_lobby(&app, 3).await;
    assert!(!full.started);

    let (status, started) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", full.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(started.started);
    assert_eq!(started.id, full.id);
}

#[tokio::test]
async fn start_game_rejects_when_not_full() {
    let app = app();
    let created = create_lobby(&app, "lonely", 3).await;
    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", created.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.error.contains("lobby not full"));
}

#[tokio::test]
async fn start_game_rejects_when_already_started() {
    let app = app();
    let full = fill_lobby(&app, 3).await;
    let (status, _) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", full.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", full.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "game already started");
}

#[tokio::test]
async fn start_unknown_lobby_returns_404() {
    let (status, body) = send::<ErrorBody>(
        &app(),
        json_request(Method::POST, "/lobby/NOPE/start", json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.error, "lobby not found");
}

#[tokio::test]
async fn join_after_start_is_rejected_as_full() {
    // The lobby fills exactly when the game starts; a late join should hit "lobby is full".
    let app = app();
    let full = fill_lobby(&app, 3).await;
    let (status, _) = send::<LobbyView>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", full.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send::<ErrorBody>(
        &app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/join", full.id),
            json!({ "playername": "latecomer" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "lobby is full");
}

async fn fill_with_ais_and_start(app: &Router, human: &str) -> LobbyView {
    let created = create_lobby(app, human, 3).await;
    for _ in 0..2 {
        let (s, _) = send::<LobbyView>(
            app,
            json_request(
                Method::POST,
                &format!("/lobby/{}/add_ai", created.id),
                json!({}),
            ),
        )
        .await;
        assert_eq!(s, StatusCode::OK);
    }
    let (s, started) = send::<LobbyView>(
        app,
        json_request(
            Method::POST,
            &format!("/lobby/{}/start", created.id),
            json!({}),
        ),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(started.started);
    started
}

#[tokio::test]
async fn state_returns_game_view_for_player() {
    let app = app();
    let started = fill_with_ais_and_start(&app, "human").await;

    let (status, view) = send::<GameView>(
        &app,
        get_request(&format!("/lobby/{}/state?player=human", started.id)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view.lobby_id, started.id);
    assert_eq!(view.your_name, "human");
    assert_eq!(view.your_index, 0);
    assert_eq!(view.your_hand.len(), 1, "round 1 deals one card per player");
    assert_eq!(view.round, 1);
    assert_eq!(view.players.len(), 3);
    // after AI driver runs, we must never be sitting in choose_trumpf with an AI chooser
    if view.phase == "choose_trumpf" {
        let idx = view.current_dealer_index;
        assert!(
            !view.players[idx].is_ai,
            "AI chooser should have been auto-resolved"
        );
    }
    // for an all-AI-chooser scenario the only possible resting phases are prediction or choose_trumpf (human chooser)
    assert!(
        matches!(view.phase.as_str(), "prediction" | "choose_trumpf"),
        "unexpected phase: {}",
        view.phase
    );
}

#[tokio::test]
async fn state_rejects_unknown_player() {
    let app = app();
    let started = fill_with_ais_and_start(&app, "human").await;
    let (status, body) = send::<ErrorBody>(
        &app,
        get_request(&format!("/lobby/{}/state?player=ghost", started.id)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body.error, "player not in this lobby");
}

#[tokio::test]
async fn state_rejects_before_start() {
    let app = app();
    let created = create_lobby(&app, "host", 3).await;
    let (status, body) = send::<ErrorBody>(
        &app,
        get_request(&format!("/lobby/{}/state?player=host", created.id)),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "game not started");
}

#[tokio::test]
async fn event_log_records_round_progress() {
    let app = app();
    let started = fill_with_ais_and_start(&app, "host").await;
    let state = fetch_state(&app, &started.id, "host").await;
    // RoundStarted should always appear first
    assert!(
        matches!(
            state.log.first(),
            Some(EventView::RoundStarted { round: 1, .. })
        ),
        "first event must be RoundStarted round 1: {:?}",
        state.log.first()
    );
    // Bots predict ahead of host; we should see Predicted entries for indices 1 and 2
    let predicted: Vec<usize> = state
        .log
        .iter()
        .filter_map(|e| match e {
            EventView::Predicted { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert!(
        predicted.contains(&1) && predicted.contains(&2),
        "AI predictions should be logged: {:?}",
        predicted
    );
}

#[tokio::test]
async fn round_one_human_drives_to_round_two() {
    // Human "host" at index 0, two AIs. Round 1: dealer=host, so AI driver
    // does nothing if the top card is a Wizard (host is chooser) — host must
    // resolve via /choose_trumpf. Then host predicts, AIs auto-predict, host
    // leads or trails depending on dealer rotation. We drive only host's
    // actions; AIs fall to drive_ai_actions in each handler.
    let app = app();
    let started = fill_with_ais_and_start(&app, "host").await;
    let id = started.id.clone();

    let mut state = fetch_state(&app, &id, "host").await;
    let mut safety = 200;
    while state.round == 1 && state.phase != "game_over" {
        safety -= 1;
        assert!(safety > 0, "round 1 didn't advance; stuck in {:?}", state);

        match state.phase.as_str() {
            "choose_trumpf" => {
                if state.current_dealer_index == state.your_index {
                    state = post_json_as::<GameView>(
                        &app,
                        &format!("/lobby/{id}/choose_trumpf"),
                        json!({"player": "host", "suit": "blue"}),
                    )
                    .await;
                } else {
                    // AI should already have resolved; just re-fetch
                    state = fetch_state(&app, &id, "host").await;
                }
            }
            "prediction" => {
                if state.current_player_index == state.your_index {
                    state = post_json_as::<GameView>(
                        &app,
                        &format!("/lobby/{id}/predict"),
                        json!({"player": "host", "prediction": 0}),
                    )
                    .await;
                } else {
                    state = fetch_state(&app, &id, "host").await;
                }
            }
            "playing" => {
                if state.current_player_index == state.your_index {
                    let pick = state.your_legal_card_indices[0];
                    state = post_json_as::<GameView>(
                        &app,
                        &format!("/lobby/{id}/play"),
                        json!({"player": "host", "card_index": pick}),
                    )
                    .await;
                } else {
                    state = fetch_state(&app, &id, "host").await;
                }
            }
            other => panic!("unexpected phase in round 1: {other}"),
        }
    }

    // Round 1 finished. We should be in round 2 in either choose_trumpf or prediction.
    assert_eq!(state.round, 2, "should have advanced into round 2");
    assert!(matches!(
        state.phase.as_str(),
        "choose_trumpf" | "prediction"
    ));
    // Host hasn't acted yet in round 2 — their prediction is still None
    assert!(state.predictions[state.your_index].is_none());
    // Tricks_won reset for round 2
    assert!(state.tricks_won.iter().all(|&t| t == 0));
    // Points are integers, but at least one player has non-zero points after round 1
    assert!(state.points.iter().any(|&p| p != 0), "scoring must apply");
}

async fn fetch_state(app: &Router, id: &str, player: &str) -> GameView {
    let (status, view) = send::<GameView>(
        app,
        get_request(&format!("/lobby/{id}/state?player={player}")),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "fetch_state status");
    view
}

async fn post_json_as<T: DeserializeOwned>(app: &Router, uri: &str, body: Value) -> T {
    let (status, view) = send::<T>(app, json_request(Method::POST, uri, body)).await;
    assert_eq!(status, StatusCode::OK, "POST {uri} returned {status}");
    view
}

#[tokio::test]
async fn cors_preflight_allows_any_origin() {
    let req = Request::builder()
        .method(Method::OPTIONS)
        .uri("/lobby")
        .header("origin", "http://example.test")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let res = app().oneshot(req).await.unwrap();
    assert!(res.status().is_success());
    let headers = res.headers();
    assert_eq!(
        headers
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok()),
        Some("*")
    );
}
