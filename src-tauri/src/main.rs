#![feature(result_option_inspect)]
#![feature(try_blocks)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use num_traits::FromPrimitive;
use serde::Serialize;

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

use std::{error::Error, str::FromStr};

use futures_util::sink::SinkExt;
use reqwest::header::{CONNECTION, HOST, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE};
use reqwest::Body;
use reqwest::{header::USER_AGENT, Method, Url, Version};
use serde_json::{json, Value};

use tauri::{AppHandle, Manager};
use tokio::join;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request};
use tokio_tungstenite::tungstenite::http::request::Parts;
use tokio_tungstenite::tungstenite::Message as WebsocketMessage;
use tokio_tungstenite::WebSocketStream;

mod queue;

use queue::*;

enum ResponseType {
    ListQueues,
}

struct State {
    message_number: AtomicU64,
    websocket: RwLock<Option<WebSocketStream<TcpStream>>>,
    queues: RwLock<BTreeMap<u64, Queue>>,
    response_type: RwLock<HashMap<u64, ResponseType>>,
}

fn main() {
    console_subscriber::init();
    tauri::Builder::default()
        .setup(|app| {
            // let main_window = app.get_window("main").unwrap();

            // let monitor = main_window.current_monitor()?.unwrap();
            // main_window.set_position(LogicalPosition::new(
            //     monitor.size().width - main_window.outer_size()?.width - 20,
            //     20,
            // ))?;

            app.manage(State {
                message_number: AtomicU64::new(0),
                websocket: RwLock::const_new(None),
                queues: RwLock::const_new(BTreeMap::new()),
                response_type: RwLock::const_new(HashMap::new()),
            });

            let handle = app.handle();

            tauri::async_runtime::spawn(async move {
                setup(handle).await.expect("Error in setup");
            });
            Ok(())
        })
        .on_page_load(|_, _| {
            println!("Page loaded!");
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn setup(app: AppHandle) -> Result<(), Box<dyn Error + Sync + Send>> {
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

    {
        let state = app.state::<State>();
        let mut socket = state.websocket.write().await;
        *socket = Some(res.0);
    };

    join!(
        list_all_queues(app.clone()),
        ping(app.clone()),
        read_messages(app.clone())
    );

    println!("Disconnected");

    Ok(())
}

async fn list_all_queues(app: AppHandle) {
    let state = app.state::<State>();
    let result: Result<(), Box<dyn Error + Sync + Send>> = try {
        let message_number = &state.message_number;
        if let Some(websocket) = &mut *state.websocket.write().await {
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
        }
    };
    result.expect("Error when fetching queues");
}

async fn ping(app: AppHandle) {
    let state = app.state::<State>();
    let result: Result<(), Box<dyn Error + Sync + Send>> = try {
        let mut interval = tokio::time::interval(tokio::time::Duration::new(5, 0));
        loop {
            interval.tick().await;
            println!("Queues {:#?}", state.queues.read().await);
            if let Some(websocket) = &mut *state.websocket.write().await {
                websocket
                    .send(WebsocketMessage::Ping(vec![1, 3, 3, 7, 4, 2, 0]))
                    .await?;
            };
        }
    };
    result.expect("Error when reading messages");
}

async fn read_messages(app: AppHandle) {
    let state = app.state::<State>();
    let result: Result<(), Box<dyn Error + Sync + Send>> = try {
        loop {
            if let Some(socket) = &mut *state.websocket.write().await {
                if let Some(message) = socket.next().await {
                    let message = message?;
                    println!("Message Received {:#?}", message);
                    match message {
                        WebsocketMessage::Text(body) => {
                            parse_message(app.clone(), body.parse().expect("Body not valid json"))
                                .await
                        }
                        WebsocketMessage::Pong(_) => println!("Recieved pong"),
                        _ => panic!("Unknown message type"),
                    }
                } else {
                    break;
                }
            };
        }
    };
    result.expect("Error when reading messages");
}

async fn parse_message(app: AppHandle, body: Value) {
    let state = app.state::<State>();
    let response_types = state.response_type.read().await;
    if let Some(message_id) = body.get("I") {
        println!(
            "Message ID {:#?}",
            message_id
                .as_str()
                .expect("Message id not a valid string")
                .parse::<u64>()
                .expect("Invalid message id")
        );

        let message_id = &message_id
            .as_str()
            .expect("Message id not a valid string")
            .parse()
            .expect("Invalid message id");

        match response_types[message_id] {
            ResponseType::ListQueues => {
                let queues = body["R"]
                    .as_array()
                    .expect("R is not an array")
                    .iter()
                    .map(|value| {
                        let id = value["Id"].as_u64().expect("Id not a valid number");
                        let members = value["Members"]
                            .as_array()
                            .expect("Members is not a valid array")
                            .iter()
                            .map(|member| User {
                                username: member["UserName"]
                                    .as_str()
                                    .expect(
                                        format!("Username {:#?} not a string", member["UserName"])
                                            .as_str(),
                                    )
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
                        let name = value["Name"].to_string();
                        let status = QueueStatus::from_u64(
                            value["Status"].as_u64().expect("Status not valid integer"),
                        )
                        .expect("Status not a valid value");
                        let messages = value["Messages"]
                            .as_array()
                            .expect("Messages not valid array")
                            .iter()
                            .map(|message| {
                                let content = message["Content"].to_string();
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

                        (
                            id,
                            Queue {
                                id,
                                name,
                                status,
                                members,
                                messages,
                            },
                        )
                    })
                    .collect();
                *state.queues.write().await = queues;
            }
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
            println!("Notification Type: {}", notification_type);
            match notification_type {
                "NewQueue"
                | "QueueStatusChanged"
                | "QueueMembershipChanged"
                | "QueueMessageSent" => {
                    println!("Processing changed queue");
                    let updated_queue = queue_from_object(&notification["A"][0]).await;
                    let queues = &mut *state.queues.write().await;
                    queues.insert(updated_queue.id, updated_queue);
                    app.emit_all(
                        "queues_updated",
                        serde_json::to_value(queues).expect("Couldn't serialize queues"),
                    )
                    .expect("Couldn't emit queue update event");
                }
                "NagQueue" => {}
                _ => {}
            }
        }
    }
}

async fn queue_from_object(value: &Value) -> Queue {
    let id = value["Id"].as_u64().expect("Id not a valid number");
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
    let name = value["Name"].to_string();
    let status = QueueStatus::from_u64(value["Status"].as_u64().expect("Status not valid integer"))
        .expect("Status not a valid value");
    let messages = value["Messages"]
        .as_array()
        .expect("Messages not valid array")
        .iter()
        .map(|message| {
            let content = message["Content"].to_string();
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
    }
}
