use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Retile,
    ReloadConfig,
    MoveToWorkspace(u32),
    MoveWindowToWorkSpace(u32),
    SwapWindow,
    SwapWithGlobal,
    NewWorkspace,
}

pub type ActionSender = Sender<Action>;
