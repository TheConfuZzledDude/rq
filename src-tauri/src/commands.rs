use std::{collections::BTreeMap, error::Error, sync::atomic::Ordering};

use crate::{queue::Queue, ResponseType, State};
use futures_util::sink::SinkExt;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message as WebsocketMessage;
use tracing::log::{self, debug};
use tracing_unwrap::ResultExt;

#[tauri::command]
pub(crate) async fn get_queues(state: tauri::State<'_, State>) -> Result<BTreeMap<u64, Queue>, ()> {
    Ok(state.queues.read().await.clone())
}

#[tauri::command]
pub(crate) async fn leave_queue(state: tauri::State<'_, State>, id: u64) -> Result<(), ()> {
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::LeaveQueue,
    );

    websocket
        .send(WebsocketMessage::text(
            json! {
                {
                    "I": message_number.fetch_add(1, Ordering::Relaxed),
                    "H": "QHub",
                    "M": "LeaveQueue",
                    "A": [id]
                }
            }
            .to_string(),
        ))
        .await
        .expect("Couldn't join queue");

    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn join_queue(state: tauri::State<'_, State>, id: u64) -> Result<(), ()> {
    debug!("Joining queue...");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::JoinQueue,
    );
    debug!("Logged request type request type");

    websocket
        .send(WebsocketMessage::text(
            json! {
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "JoinQueue",
                        "A": [id]
                    }
            }
            .to_string(),
        ))
        .await
        .expect("Couldn't join queue");
    debug!("Join request sent!");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn message_queue(state: tauri::State<'_, State>, id: u64, content: &str) -> Result<(), ()> {
    debug!("Sending message to queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::MessageQueue,
    );
    debug!("Logged request type request type");

    websocket
        .send(WebsocketMessage::text(
            json! {
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "MessageQueue",
                        "A": [id, content]
                    }
            }
            .to_string(),
        ))
        .await
        .expect_or_log("Couldn't send message to queue");
    debug!("Queue message sent!");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn start_queue(state: tauri::State<'_, State>, id: u64) -> Result<(), ()> {
    debug!("Starting queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::StartQueue,
    );
    debug!("Logged request type request type");

    websocket
        .send(WebsocketMessage::text(
            json! {
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "ActivateQueue",
                        "A": [id]
                    }
            }
            .to_string(),
        ))
        .await
        .expect_or_log("Couldn't start queue");
    debug!("Queue started");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn reset_queue(state: tauri::State<'_, State>, id: u64) -> Result<(), ()> {
    debug!("Resetting queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::ResetQueue,
    );
    debug!("Logged request type request type");

    websocket
        .send(WebsocketMessage::text(
            json! {
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "DeactivateQueue",
                        "A": [id]
                    }
            }
            .to_string(),
        ))
        .await
        .expect_or_log("Couldn't reset queue");
    debug!("Queue reset");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn nag_queue(state: tauri::State<'_, State>, id: u64) -> Result<(), ()> {
    debug!("Nagging queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::NagQueue,
    );
    debug!("Logged request type request type");

    websocket
        .send(WebsocketMessage::text(
            json! {
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "NagQueue",
                        "A": [id]
                    }
            }
            .to_string(),
        ))
        .await
        .expect_or_log("Couldn't nag queue");
    debug!("Queue nagged");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn delete_queue(state: tauri::State<'_, State>, id: u64) -> Result<(), ()> {
    debug!("Deleting queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::DeleteQueue,
    );
    debug!("Logged request type request type");

    websocket
        .send(WebsocketMessage::text(
            json! {
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "CloseQueue",
                        "A": [id]
                    }
            }
            .to_string(),
        ))
        .await
        .expect_or_log("Couldn't delete queue");
    debug!("Queue deleted");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn new_queue(state: tauri::State<'_, State>, name: &str, restrict_to_group: Option<&str> ) -> Result<(), ()> {
    debug!("Starting queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        ResponseType::NewQueue,
    );
    debug!("Logged request type request type");

    websocket
        .send(WebsocketMessage::text(
            json! {
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "StartQueue",
                        "A": [name, restrict_to_group.unwrap_or("")]
                    }
            }
            .to_string(),
        ))
        .await
        .expect_or_log("Couldn't create queue");
    debug!("Queue created");
    Ok(())
}
