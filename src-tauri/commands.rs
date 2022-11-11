use std::sync::atomic::Ordering;

use crate::{connect, new_queue::NewQueueApp, settings::Settings, RequestType, State};
use futures_util::sink::SinkExt;
use serde_json::json;
use tauri::Manager;
use tokio::select;
use tokio_tungstenite::tungstenite::Message as WebsocketMessage;
use tracing::log::debug;

#[tauri::command]
pub(crate) async fn fetch_data(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, State>,
) -> Result<(), ()> {
    let queues = &*state.queues.read().await;
    let config = fetch_settings().await?;

    app_handle
        .emit_all(
            "data_updated",
            json!(
                    {
                        "queues": queues,
                        "config": config,
                    }
            ),
        )
        .unwrap();

    Ok(())
}

#[tauri::command]
pub(crate) async fn leave_queue(state: tauri::State<'_, State>, id: u64) -> Result<(), ()> {
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        RequestType::LeaveQueue,
    );

    select!(
    _ = websocket
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
        )) => { },
        _ = state.cancel_websockets.notified() => {return Err(())}
    );

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
        RequestType::JoinQueue,
    );
    debug!("Logged request type request type");

    select!(
    _ = websocket
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
        )) => { },
        _ = state.cancel_websockets.notified() => {return Err(())}
    );

    debug!("Join request sent!");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn message_queue(
    state: tauri::State<'_, State>,
    id: u64,
    content: &str,
) -> Result<(), ()> {
    debug!("Sending message to queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        RequestType::MessageQueue,
    );
    debug!("Logged request type request type");

    select!(
        _ = websocket
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
            ) ) => {},
        _ = state.cancel_websockets.notified() => {return Err(())}
    );
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
        RequestType::StartQueue,
    );
    debug!("Logged request type request type");

    select!(
        _ = websocket
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
            )) => {},
        _ = state.cancel_websockets.notified() => {return Err(())}
    );
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
        RequestType::ResetQueue,
    );
    debug!("Logged request type request type");

    select!(
        _ = websocket
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
            )) => {},
        _ = state.cancel_websockets.notified() => {return Err(())}
    );
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
        RequestType::NagQueue,
    );
    debug!("Logged request type request type");

    select!(
        _ = websocket
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
            )) => {},
        _ = state.cancel_websockets.notified() => { return Err(()) }
    );
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
        RequestType::DeleteQueue,
    );
    debug!("Logged request type request type");

    select!(
        _ = websocket
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
            )) => {},
        _ = state.cancel_websockets.notified() => {return Err(())}
    );
    debug!("Queue deleted");
    Ok(())
}

#[tracing::instrument(level = "debug")]
#[tauri::command]
pub(crate) async fn new_queue(
    state: tauri::State<'_, State>,
    name: &str,
    restrict_to_group: Option<&str>,
) -> Result<(), ()> {
    debug!("Starting queue");
    let mut websocket = state.websocket_tx.lock().await;
    let websocket = websocket.as_mut().ok_or(())?;
    let message_number = &state.message_number;

    debug!("Logging request type");
    state.response_type.write().await.insert(
        message_number.load(Ordering::Relaxed),
        RequestType::NewQueue,
    );
    debug!("Logged request type request type");

    select!(
        _ = websocket
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
            )) => {},
        _ = state.cancel_websockets.notified() => {return Err(())}
    );
    debug!("Queue created");
    Ok(())
}

#[tauri::command]
pub(crate) async fn fetch_settings() -> Result<Settings, ()> {
    let settings = confy::load("rq", None).map_err(|_e| ())?;

    Ok(settings)
}

#[tauri::command]
pub(crate) async fn write_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, State>,
    settings: Settings,
) -> Result<(), ()> {
    confy::store("rq", None, settings).map_err(|_e| ())?;

    state.cancel_websockets.notify_waiters();
    let _ = connect(app.clone()).await;

    Ok(())
}

#[tauri::command]
pub(crate) async fn open_settings(app: tauri::AppHandle) -> Result<(), ()> {
    let settings = confy::load::<Settings>("rq", None).map_err(|_e| ())?;

    settings.launch(app);

    Ok(())
}

#[tauri::command]
pub(crate) async fn open_new_queue(app: tauri::AppHandle) -> Result<(), ()> {
    NewQueueApp::launch(app);

    Ok(())
}
