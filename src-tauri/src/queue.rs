use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    pub(crate) username: String,
    pub(crate) full_name: String,
    pub(crate) email: String,
}

#[derive(Serialize, Deserialize, FromPrimitive, Debug, Copy, Clone)]
pub(crate) enum QueueStatus {
    Open = 0,
    Started = 1,
    Closed = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Queue {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) status: QueueStatus,
    pub(crate) members: Vec<User>,
    pub(crate) messages: Vec<Message>,
    pub(crate) restrict_to_group: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Message {
    pub(crate) content: String,
    pub(crate) sender: User,
}
