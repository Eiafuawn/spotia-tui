use std::{fmt, string::ToString};

use rspotify::model::SimplifiedPlaylist;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,

    // Home Actions
    MoveUp,
    MoveDown,
    EnterEditing,
    QuitEditing,
    Save,
    // Download Actions
    EnterDownloader,
    SelectPlaylist(String, usize),
    Downloading(String),
    DownloadFinished,
    // Manage Actions
    EnterManager,
    SelectFolder(String),
}
