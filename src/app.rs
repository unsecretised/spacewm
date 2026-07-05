use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use objc2::rc::Retained;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};

use crate::hotkeys::EventTapHandle;
use crate::subscriber::{Action, ActionSender};
use crate::{config::Config, windows::RawWindow};

pub struct AppState {
    running_apps: Vec<Arc<Retained<NSRunningApplication>>>,
    workspaces: HashMap<usize, Workspace>,
    config: Config,
    event_tap_handle: EventTapHandle,
}

impl AppState {
    pub fn init(sender: ActionSender) -> AppState {
        let workspaces = RawWindow::get_open_windows()
            .iter()
            .enumerate()
            .map(|(i, b)| (i, Workspace(WorkspaceType::SingleWindow(b.clone()))))
            .collect();

        let config = Config::load_config().unwrap_or_else(|| {
            Config::create_default_config(Config::get_config_path());
            Config::default()
        });

        let running_apps = NSWorkspace::sharedWorkspace()
            .runningApplications()
            .to_vec()
            .iter()
            .map(|x| Arc::new(x.clone()))
            .collect();

        let event_tap_handle = config.register_hotkeys(sender);

        AppState {
            workspaces,
            config,
            running_apps,
            event_tap_handle,
        }
    }

    pub async fn start_thread_handler_service() {}
}

#[derive(Debug, Clone)]
pub struct Workspace(pub WorkspaceType);

#[derive(Debug, Clone)]
pub enum WorkspaceType {
    Empty,
    SingleWindow(RawWindow),
    FullWorkspace {
        focussed_window: RawWindow,
        spare_window: RawWindow,
    },
}
