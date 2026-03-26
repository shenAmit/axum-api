use crate::realtime::Realtime;
use crate::ws::protocol::{ClientMsg, ServerMsg};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Query};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    #[serde(rename = "userId")]
    pub user_id: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(q): Query<WsQuery>,
    Extension(rt): Extension<Realtime>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, q.user_id, rt))
}

async fn handle_socket(socket: WebSocket, user_id: String, rt: Realtime) {
    // Mark user online in Redis.
    let _ = rt.set_online(&user_id).await;

    let (ws_sender, mut ws_receiver) = socket.split();

    // We cannot clone the websocket sender, so forward PubSub -> WS through a channel.
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
    // Commands sent to the PubSub task (ex: subscribe to a room channel).
    let (ps_cmd_tx, mut ps_cmd_rx) = mpsc::unbounded_channel::<String>();

    // PubSub subscription for this user (DMs) + any joined rooms.
    let mut pubsub = match rt.redis_client.get_async_pubsub().await {
        Ok(conn) => conn,
        Err(_) => {
            let _ = out_tx.send(Message::Text(
                serde_json::to_string(&ServerMsg::System {
                    message: "Redis unavailable".to_string(),
                })
                .unwrap()
                .into(),
            ));
            return;
        }
    };

    let user_chan = format!("chan:user:{}", user_id);
    if pubsub.subscribe(&user_chan).await.is_err() {
        return;
    }

    // Task: own pubsub, handle subscribe commands, forward PubSub -> WS channel.
    //
    // We cannot hold a `pubsub.on_message()` stream and also call `subscribe()` later
    // (both require mutable access). So we poll for subscribe commands and use
    // `pubsub.get_message()` with a short timeout.
    let pubsub_task = {
        let out_tx = out_tx.clone();
        tokio::spawn(async move {
            loop {
                while let Ok(room_chan) = ps_cmd_rx.try_recv() {
                    let _ = pubsub.subscribe(room_chan).await;
                }

                // Create a short-lived stream so we can `subscribe()` again next loop.
                let mut stream = pubsub.on_message();
                match tokio::time::timeout(std::time::Duration::from_millis(250), stream.next()).await
                {
                    Ok(Some(msg)) => {
                        let msg: redis::Msg = msg;
                        let payload: redis::RedisResult<String> = msg.get_payload();
                        if let Ok(text) = payload {
                            let _ = out_tx.send(Message::Text(text.into()));
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {} // timeout
                }
            }
        })
    };

    // Task: write outgoing messages to websocket.
    let write_task = tokio::spawn(async move {
        let mut ws_sender = ws_sender;
        while let Some(msg) = out_rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Receive websocket messages -> publish to Redis.
    while let Some(Ok(msg)) = ws_receiver.next().await {
        let Message::Text(text) = msg else { continue };

        let parsed = match serde_json::from_str::<ClientMsg>(&text) {
            Ok(p) => p,
            Err(_) => {
                let sys = ServerMsg::System {
                    message: "Invalid message format".to_string(),
                };
                if let Ok(json) = serde_json::to_string(&sys) {
                    let _ = rt.publish_user(&user_id, &json).await;
                }
                continue;
            }
        };

        match parsed {
            ClientMsg::Dm { to, body } => {
                let dm = ServerMsg::Dm {
                    from: user_id.clone(),
                    body,
                };
                if let Ok(json) = serde_json::to_string(&dm) {
                    // deliver to receiver + sender (so sender sees their own message)
                    let _ = rt.publish_user(&to, &json).await;
                    let _ = rt.publish_user(&user_id, &json).await;
                }
            }
            ClientMsg::JoinRoom { room } => {
                let room = room.trim().to_string();
                if room.is_empty() {
                    continue;
                }

                let _ = rt.join_room(&room, &user_id).await;
                let room_chan = format!("chan:room:{}", room);
                let _ = ps_cmd_tx.send(room_chan);

                let sys = ServerMsg::System {
                    message: format!("Joined room {room}"),
                };
                if let Ok(json) = serde_json::to_string(&sys) {
                    let _ = rt.publish_user(&user_id, &json).await;
                }
            }
            ClientMsg::RoomMsg { room, body } => {
                let room = room.trim().to_string();
                if room.is_empty() {
                    continue;
                }
                let rm = ServerMsg::RoomMsg {
                    room: room.clone(),
                    from: user_id.clone(),
                    body,
                };
                if let Ok(json) = serde_json::to_string(&rm) {
                    let _ = rt.publish_room(&room, &json).await;
                }
            }
        }
    }

    pubsub_task.abort();
    write_task.abort();
    let _ = rt.set_offline(&user_id).await;
}

