use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,

    // Download menu
    Downloader,

    // Manager menu
    Manager,

    // Outputs
    Idle,
    Downloading,
}
