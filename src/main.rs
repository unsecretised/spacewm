use std::sync::mpsc::channel;

use crate::{accessibility::ensure_accessibility_permission, app::AppState};

mod accessibility;
mod app;
mod config;
mod hotkeys;
mod subscriber;
mod windows;

fn main() {
    ensure_accessibility_permission();

    let (sender, receiver) = channel();

    let state = AppState::init(sender);
}
