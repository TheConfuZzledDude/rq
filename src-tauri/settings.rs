use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Default)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub full_name: String,

    #[serde(default)]
    pub username: String,

    #[serde(default)]
    pub groups: Vec<String>,

    pub theme: Theme,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Default)]
#[non_exhaustive]
pub(crate) enum Theme {
    Win98,
    ClassicQ3,
    #[default]
    Modern,
}
