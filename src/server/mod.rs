use std::net::SocketAddr;

use axum::{
    Router,
    extract::{
        State as AxumState,
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::info;

use crate::ai::AIPlayer;
use crate::game::GameState;
use crate::game::piece::Camp;
use crate::protocol::{ClientMessage, GameMode, ServerMessage};

#[derive(Clone)]
pub struct AppState {
    pub games: Arc<Mutex<Vec<GameSession>>>,
}

pub struct GameSession {
    pub state: GameState,
    pub mode: GameMode,
    pub ai_player: Option<AIPlayer>,
}

pub async fn run_server(port: u16) {
    let state = AppState {
        games: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("frontend"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut session_idx: Option<usize> = None;

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                let err = ServerMessage::Error {
                    message: format!("Invalid message: {}", e),
                };
                let _ = sender
                    .send(Message::Text(serde_json::to_string(&err).unwrap().into()))
                    .await;
                continue;
            }
        };

        let response = process_message(client_msg, &state, &mut session_idx).await;
        for msg in response {
            let text = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(text.into())).await.is_err() {
                return;
            }
        }
    }
}

async fn process_message(
    msg: ClientMessage,
    state: &AppState,
    session_idx: &mut Option<usize>,
) -> Vec<ServerMessage> {
    let mut games = state.games.lock().await;
    let mut responses = Vec::new();

    match msg {
        ClientMessage::NewGame { mode } => {
            let game = GameSession {
                state: GameState::new(),
                mode: mode.clone(),
                ai_player: if matches!(mode, GameMode::Pve) {
                    Some(AIPlayer::new(Camp::Red))
                } else {
                    None
                },
            };

            let idx = if let Some(i) = *session_idx {
                games[i] = game;
                i
            } else {
                games.push(game);
                *session_idx = Some(games.len() - 1);
                games.len() - 1
            };

            let g = &games[idx];
            responses.push(ServerMessage::GameStarted {
                board: g.state.board.clone(),
                current_player: g.state.current_player,
                mode,
                move_count: g.state.move_count,
            });
        }

        ClientMessage::Flip { x, y } => {
            let idx = match *session_idx {
                Some(i) if i < games.len() => i,
                _ => {
                    responses.push(ServerMessage::Error {
                        message: "No active game".into(),
                    });
                    return responses;
                }
            };

            let g = &mut games[idx];
            if g.state.game_over {
                return responses;
            }

            let cell = &g.state.board.cells[x][y];
            if cell.piece.is_none() || cell.revealed || cell.monkey.is_some() {
                responses.push(ServerMessage::Error {
                    message: "Cannot flip this cell".into(),
                });
                return responses;
            }

            g.state.board.cells[x][y].revealed = true;
            g.state.toggle_player();
            g.state.move_count += 1;
            g.state.check_winner();

            if g.state.game_over {
                responses.push(ServerMessage::GameOver {
                    winner: g.state.winner.unwrap(),
                    board: g.state.board.clone(),
                });
            } else {
                responses.push(ServerMessage::BoardUpdate {
                    board: g.state.board.clone(),
                    current_player: g.state.current_player,
                    message: format!("翻开了 ({}, {}) 的棋子", x, y),
                    move_count: g.state.move_count,
                });

                if matches!(g.mode, GameMode::Pve) && g.state.current_player == Camp::Red {
                    responses.push(ServerMessage::AiThinking);
                    let ai_result = process_ai_turn(g);
                    responses.extend(ai_result);
                }
            }
        }

        ClientMessage::Move { from_x, from_y, to_x, to_y } => {
            let idx = match *session_idx {
                Some(i) if i < games.len() => i,
                _ => {
                    responses.push(ServerMessage::Error {
                        message: "No active game".into(),
                    });
                    return responses;
                }
            };

            let g = &mut games[idx];
            if g.state.game_over {
                return responses;
            }

            let (ok, _) = g.state.board.dispose_piece(
                (from_x, from_y),
                Some((to_x, to_y)),
                g.state.current_player,
            );

            if ok {
                g.state.toggle_player();
                g.state.move_count += 1;
                g.state.check_winner();

                if g.state.game_over {
                    responses.push(ServerMessage::GameOver {
                        winner: g.state.winner.unwrap(),
                        board: g.state.board.clone(),
                    });
                } else {
                    responses.push(ServerMessage::BoardUpdate {
                        board: g.state.board.clone(),
                        current_player: g.state.current_player,
                        message: String::new(),
                        move_count: g.state.move_count,
                    });

                    if matches!(g.mode, GameMode::Pve) && g.state.current_player == Camp::Red {
                        responses.push(ServerMessage::AiThinking);
                        let ai_result = process_ai_turn(g);
                        responses.extend(ai_result);
                    }
                }
            } else {
                responses.push(ServerMessage::Error {
                    message: "Invalid move".into(),
                });
            }
        }
    }

    responses
}

fn process_ai_turn(g: &mut GameSession) -> Vec<ServerMessage> {
    let mut responses = Vec::new();

    if let Some(ai) = &g.ai_player {
        let action = ai.choose_action(&g.state.board);
        if let Some(action) = action {
            let (description, action_type, from_x, from_y, to_x, to_y) = match action.action_type {
                crate::ai::heuristic::ActionType::Flip => {
                    let (x, y) = action.self_pos;
                    g.state.board.cells[x][y].revealed = true;
                    (
                        format!("AI 翻开了 ({}, {}) 的棋子", x, y),
                        "flip".to_string(),
                        x, y, None, None,
                    )
                }
                crate::ai::heuristic::ActionType::Move => {
                    let tp = action.target_pos.unwrap();
                    let (ok, _) = g.state.board.dispose_piece(
                        action.self_pos,
                        Some(tp),
                        g.state.current_player,
                    );
                    let desc = if ok {
                        format!("AI 从 ({}, {}) 移动到 ({}, {})", action.self_pos.0, action.self_pos.1, tp.0, tp.1)
                    } else {
                        "AI 动作执行失败".to_string()
                    };
                    (desc, "move".to_string(), action.self_pos.0, action.self_pos.1, Some(tp.0), Some(tp.1))
                }
            };

            g.state.toggle_player();
            g.state.move_count += 1;
            g.state.check_winner();

            responses.push(ServerMessage::AiAction { description, action_type, from_x, from_y, to_x, to_y });

            if g.state.game_over {
                responses.push(ServerMessage::GameOver {
                    winner: g.state.winner.unwrap(),
                    board: g.state.board.clone(),
                });
            } else {
                responses.push(ServerMessage::BoardUpdate {
                    board: g.state.board.clone(),
                    current_player: g.state.current_player,
                    message: String::new(),
                    move_count: g.state.move_count,
                });
            }
        } else {
            g.state.toggle_player();
            responses.push(ServerMessage::AiAction {
                description: "AI 无可执行动作，跳过回合".to_string(),
                action_type: "skip".to_string(),
                from_x: 0, from_y: 0, to_x: None, to_y: None,
            });
            responses.push(ServerMessage::BoardUpdate {
                board: g.state.board.clone(),
                current_player: g.state.current_player,
                message: String::new(),
                move_count: g.state.move_count,
            });
        }
    }

    responses
}
