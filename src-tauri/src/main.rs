#![feature(result_option_inspect)]
#![feature(try_blocks)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use futures_util::StreamExt;
use num_traits::FromPrimitive;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use tokio::time::{sleep, Instant, Sleep};
use tokio_util::sync::CancellationToken;
use tracing::log::{debug, warn};
use tracing::{debug_span, Instrument};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::Display;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use std::str::FromStr;
use std::time::Duration;

use futures_util::sink::SinkExt;
use reqwest::header::{CONNECTION, HOST, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE};
use reqwest::Body;
use reqwest::{header::USER_AGENT, Method, Url, Version};
use serde_json::{json, Value};

use tauri::{AppHandle, Manager, CustomMenuItem, SystemTray, SystemTrayMenu,SystemTrayEvent};
use tokio::net::TcpStream;
use tokio::sync::{Notify, RwLock};
use tokio::{join, pin, select, spawn};
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::tungstenite::http::request::Parts;
use tokio_tungstenite::tungstenite::Message as WebsocketMessage;
use tokio_tungstenite::WebSocketStream;

mod commands;
mod queue;
mod util;

use commands::*;
use queue::*;

use crate::util::get_mouse_position;

#[derive(Debug)]
struct GenericError<'a>(&'a str);

impl Display for GenericError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for GenericError<'_> {}

#[derive(Debug)]
enum ResponseType {
    ListQueues,
    LeaveQueue,
    JoinQueue,
    MessageQueue,
    StartQueue,
    ResetQueue,
    NagQueue,
    DeleteQueue,
    NewQueue,
}

#[derive(Debug, Default)]
struct State {
    message_number: AtomicU64,
    queues: RwLock<BTreeMap<u64, Queue>>,
    response_type: RwLock<HashMap<u64, ResponseType>>,
    websocket_tx: Mutex<
        Option<futures_util::stream::SplitSink<WebSocketStream<TcpStream>, WebsocketMessage>>,
    >,
    websocket_rx: Mutex<Option<futures_util::stream::SplitStream<WebSocketStream<TcpStream>>>>,
    reset_keep_alive: Arc<Notify>,
}

fn main() {
    // let mut ldap = LdapConn::new("ldap://zoo.lan").unwrap();
    // ldap.sasl_gssapi_bind("zacfre").unwrap();
    // let (rs, _res) = ldap.search("", ldap3::Scope::Base, "(&(objectCategory=person)(objectClass=user))", vec!["l"]).unwrap().success().unwrap();
    // for entry in rs {
    //     println!("{:?}", SearchEntry::construct(entry));
    // }
    console_subscriber::init();
    // let console_layer = console_subscriber::spawn();
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     // .with_env_filter(EnvFilter::from_default_env())
    //     .with_span_events(FmtSpan::FULL);
    //     // .finish();
    // tracing_subscriber::registry()
    //     .with(console_layer)
    //     .with(fmt_layer)
    //     .with(EnvFilter::from_default_env())
    //     .init();

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                    app.get_window("main").unwrap().show().unwrap();
                }
            SystemTrayEvent::RightClick {
                position: _,
                size: _,
                ..
            } => {
                    println!("system tray received a right click");
                }
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .setup(|app| {
            // let main_window = app.get_window("main").unwrap();

            // let monitor = main_window.current_monitor()?.unwrap();
            // main_window.set_position(LogicalPosition::new(
            //     monitor.size().width - main_window.outer_size()?.width - 20,
            //     20,
            // ))?;

            let handle = app.handle();

            tauri::async_runtime::spawn(async move {
                setup(handle).await.expect("Error in setup");
            });
            Ok(())
        })
        .manage(State::default())
        .on_page_load(|window, _| {
            tauri::async_runtime::spawn(async move {
                let state = window.state::<State>();
                let queues = state.queues.read().await;
                window
                    .emit_all(
                        "queues_updated",
                        serde_json::to_value(&*queues).expect("Couldn't serialize queues"),
                    )
                    .expect("Couldn't emit queues_updated event");
            });
        })
        .invoke_handler(tauri::generate_handler![
            get_queues,
            leave_queue,
            join_queue,
            message_queue,
            start_queue,
            reset_queue,
            delete_queue,
            nag_queue,
            new_queue,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn setup(app: AppHandle) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _ = connect(app.clone())
        .await
        .inspect_err(|e| warn!("Error when connecting to server: {:#?}", e));

    spawn(list_all_queues(app.clone()));
    spawn(ping(app.clone()));
    spawn(read_messages(app.clone()));
    spawn(keep_alive(app.clone()));
    // spawn({
    //     let app = app.clone();
    //     async move {
    //         loop {
    //             sleep(Duration::from_millis(100)).await;
    //             let window = app.clone().get_window("main").unwrap();
    //             let m = get_mouse_position(window);
    //             app.emit_all("mouse_position", m).unwrap();
    //             println!("{:#?}", m);
    //         }
    //     }
    // });

    debug!("Disconnected");

    Ok(())
}

async fn connect(app: AppHandle) -> Result<(), Box<dyn Error + Sync + Send>> {
    debug!("Attempting to initiate connection");
    let client = reqwest::Client::new();
    let negotiate_request = client
        .request(Method::GET, "http://poolq3.zoo.lan/signalr/negotiate")
        .version(Version::HTTP_11)
        .query(&[("clientProtocol", "1.4")])
        .query(&[("connectionData", json!([{"Name": "QHub"}]).to_string())])
        .header("User", "ZacFre;Zachary Freed;Zachary.Freed@softwire.com")
        .header(
            USER_AGENT,
            "SignalR.Client.Net45/2.2.0.0 (Microsoft Windows NT 6.2.9200.0)",
        )
        .build()?;

    let negotiated = client
        .execute(negotiate_request)
        .await?
        .json::<Value>()
        .await?;

    let connect_url = Url::from_str("ws://poolq3.zoo.lan/signalr/connect")?;
    let socket_addrs = connect_url.socket_addrs(|| None)?;

    let websocket_request = client
        .request(Method::GET, connect_url.clone())
        .version(Version::HTTP_11)
        .query(&[("clientProtocol", "1.4")])
        .query(&[("transport", "webSockets")])
        .query(&[(
            "connectionToken",
            negotiated.get("ConnectionToken").unwrap(),
        )])
        .query(&[("connectionData", json!([{"Name": "QHub"}]).to_string())])
        .header("User", "ZacFre;Zachary Freed;Zachary.Freed@softwire.com")
        .header(SEC_WEBSOCKET_KEY, generate_key())
        .header(SEC_WEBSOCKET_VERSION, 13i32)
        .header(HOST, "poolq3.zoo.lan")
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket")
        .header(
            USER_AGENT,
            "SignalR.Client.Net45/2.2.0.0 (Microsoft Windows NT 6.2.9200.0)",
        )
        .build()?;
    let request_parts: Parts = http::Request::<Body>::try_from(websocket_request)?
        .into_parts()
        .0;

    let stream = tokio::net::TcpStream::connect(&*socket_addrs).await?;
    let res =
        tokio_tungstenite::client_async(Request::from_parts(request_parts, ()), stream).await?;

    let (websocket_tx, websocket_rx) = res.0.split();

    let state = app.state::<State>();
    (
        *state.websocket_tx.lock().await,
        *state.websocket_rx.lock().await,
    ) = (Some(websocket_tx), Some(websocket_rx));

    Ok(())
}

#[tracing::instrument(skip(app), level = "debug")]
async fn list_all_queues(app: AppHandle) {
    let state = app.state::<State>();
    let _: Result<(), Box<dyn Error + Sync + Send>> = try {
        let message_number = &state.message_number;
        let mut websocket = state
            .websocket_tx
            .lock()
            .instrument(debug_span!("Writing to socket"))
            .await;
        let websocket = websocket
            .as_mut()
            .ok_or(GenericError("Websocket isn't initialised"))?;
        state.response_type.write().await.insert(
            message_number.load(Ordering::Relaxed),
            ResponseType::ListQueues,
        );
        websocket
            .send(WebsocketMessage::text(
                json!(
                    {
                        "I": message_number.fetch_add(1, Ordering::Relaxed),
                        "H": "QHub",
                        "M": "ListQueues",
                        "A":[]
                    }
                )
                .to_string(),
            ))
            .await?;
    };
}

#[tracing::instrument(skip(app), level = "debug")]
async fn ping(app: AppHandle) {
    let state = app.state::<State>();
    loop {
        let _: Result<(), Box<dyn Error + Sync + Send>> = try {
            let mut interval = tokio::time::interval(tokio::time::Duration::new(5, 0));
            interval.tick().instrument(debug_span!("Ping timer")).await;
            {
                let mut websocket = state
                    .websocket_tx
                    .lock()
                    .instrument(debug_span!("Writing to socket"))
                    .await;

                let websocket = websocket
                    .as_mut()
                    .ok_or(GenericError("Websocket isn't initialised"))?;

                websocket
                    .send(WebsocketMessage::Ping(vec![1, 3, 3, 7, 4, 2, 0]))
                    .await?;
            }
            list_all_queues(app.clone()).await;
        };
    }
}

async fn keep_alive(app: AppHandle) {
    let state = app.state::<State>();
    let cancel_token = state.reset_keep_alive.clone();
    loop {
        select! {
            _ = cancel_token.notified() => {}
            _ = sleep(Duration::from_secs(10)) => {
                let _ = connect(app.clone()).await;
            }
        }
    }
}

#[tracing::instrument(skip(app), level = "debug")]
async fn read_messages(app: AppHandle) {
    let state = app.state::<State>();
    loop {
        let _: Result<(), Box<dyn Error + Sync + Send>> = try {
            let mut websocket = state
                .websocket_rx
                .lock()
                .instrument(debug_span!("Writing to socket"))
                .await;

            let websocket = websocket
                .as_mut()
                .ok_or(GenericError("Websocket isn't initialised"))?;

            if let Some(message) = websocket.next().await {
                state.reset_keep_alive.clone().notify_one();
                let message = message?;
                match message {
                    WebsocketMessage::Text(body) => {
                        parse_message(app.clone(), body.parse().expect("Body not valid json"))
                            .await;
                    }
                    WebsocketMessage::Pong(_) => {}
                    _ => panic!("Unknown message type"),
                }
            } else {
                break;
            }
        };
    }
}

#[tracing::instrument(skip(app, body), level = "debug")]
async fn parse_message(app: AppHandle, body: Value) {
    let state = app.state::<State>();
    if let Some(message_id) = body.get("I") {
        let message_id = &message_id
            .as_str()
            .expect("Message id not a valid string")
            .parse()
            .expect("Invalid message id");

        let response_types = state.response_type.read().await;

        match response_types[message_id] {
            ResponseType::ListQueues => {
                let queues: BTreeMap<u64, Queue> = body["R"]
                    .as_array()
                    .expect("R is not an array")
                    .iter()
                    .map(|queue| {
                        let queue = queue_from_object(queue);
                        let id = queue.id;

                        (id, queue)
                    })
                    .collect();
                *state.queues.write().await = queues.clone();
                app.emit_all(
                    "queues_updated",
                    serde_json::to_value(queues).expect("Couldn't serialize queues"),
                )
                .expect("Couldn't emit queue update event");
            }
            _ => {}
        }

        drop(response_types);
        state.response_type.write().await.remove(message_id);
    } else if let Some(_) = body.get("C") {
        let notifications = body["M"]
            .as_array()
            .expect("Message doesn't contain notifcations");
        for notification in notifications {
            let notification_type = notification["M"]
                .as_str()
                .expect("Notification type not valid");
            match notification_type {
                "NewQueue"
                | "QueueStatusChanged"
                | "QueueMembershipChanged" => {
                    debug!("Processing changed queue");
                    let updated_queue = queue_from_object(&notification["A"][0]);
                    let queues = &mut *state.queues.write().await;
                    queues.insert(updated_queue.id, updated_queue);
                    app.emit_all(
                        "queues_updated",
                        serde_json::to_value(queues).expect("Couldn't serialize queues"),
                    )
                    .expect("Couldn't emit queue update event");
                }
                "NagQueue" => {}
                "QueueMessageSent" => {}
                _ => {}
            }
        }
    }
}

fn queue_from_object(value: &Value) -> Queue {
    let id = value["Id"].as_u64().expect(&format!("Id {:#?} not a valid number, full queue {:#?}", value["Id"], value));
    let members = value["Members"]
        .as_array()
        .expect("Members is not a valid array")
        .iter()
        .map(|member| User {
            username: member["UserName"]
                .as_str()
                .expect(format!("Username {:#?} not a string", member["UserName"]).as_str())
                .to_owned(),
            full_name: member["FullName"]
                .as_str()
                .expect("Full name not a string")
                .to_owned(),
            email: member["EmailAddress"]
                .as_str()
                .expect("Email not a string")
                .to_owned(),
        })
        .collect();
    let name = value["Name"]
        .as_str()
        .expect("Queue name not a valid string")
        .to_owned();
    let restrict_to_group = value["RestrictToGroup"]
        .as_str()
        .expect("Queue name not a valid string")
        .to_owned();
    let status = QueueStatus::from_u64(value["Status"].as_u64().expect("Status not valid integer"))
        .expect("Status not a valid value");
    let messages = value["Messages"]
        .as_array()
        .expect("Messages not valid array")
        .iter()
        .map(|message| {
            let content = message["Content"]
                .as_str()
                .expect("Message content is not a string")
                .to_owned();
            let sender = {
                let member = &message["Sender"];
                User {
                    username: member["UserName"]
                        .as_str()
                        .expect("Username not a string")
                        .to_owned(),
                    full_name: member["FullName"]
                        .as_str()
                        .expect("Full name not a string")
                        .to_owned(),
                    email: member["EmailAddress"]
                        .as_str()
                        .expect("Email not a string")
                        .to_owned(),
                }
            };

            Message { content, sender }
        })
        .collect();

    Queue {
        id,
        name,
        status,
        members,
        messages,
        restrict_to_group,
    }
}
