use tokio::sync::mpsc;

use crate::calendar::CalendarState;
use crate::config::Config;
use crate::editor::EditorState;
use crate::engine::TaskResult;
use crate::mail::{ComposeState, InboxState};
use crate::storage::EventCache;
use crate::ui::{PaletteState, StatusLine};

use super::scene::Scene;

pub struct AppState {
    pub scene: Scene,
    pub editor: EditorState,
    pub calendar: CalendarState,
    pub inbox: InboxState,
    pub compose: Option<ComposeState>,
    pub palette: Option<PaletteState>,
    pub status: StatusLine,
    pub should_quit: bool,
    pub config: Config,
    pub task_tx: mpsc::UnboundedSender<TaskResult>,
}

impl AppState {
    pub fn new(config: Config, task_tx: mpsc::UnboundedSender<TaskResult>) -> Self {
        let notes_path = config.notes_path.clone();
        let scene = Scene::default();
        let mut status = StatusLine::new("");
        status.set_hint_for_scene(scene);

        // Load cached calendar events
        let mut calendar = CalendarState::new();
        if let Ok(Some(cache)) = EventCache::load() {
            calendar.events = cache.events;
            calendar.sync_token = cache.sync_token;
        }

        Self {
            scene,
            editor: EditorState::new_local(notes_path),
            calendar,
            inbox: InboxState::new(),
            compose: None,
            palette: None,
            status,
            should_quit: false,
            config,
            task_tx,
        }
    }

    pub fn open_inbox(&mut self) {
        self.scene = Scene::MailInbox;
    }

    pub fn open_palette(&mut self) {
        self.palette = Some(PaletteState::new());
    }

    pub fn close_palette(&mut self) {
        self.palette = None;
    }

    pub fn open_compose(&mut self) {
        self.compose = Some(ComposeState::new());
        self.scene = Scene::MailCompose;
    }

    pub fn close_compose(&mut self) {
        self.compose = None;
        self.scene = Scene::Editor;
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status.message = msg.into();
    }

    pub fn update_status_hint(&mut self) {
        self.status.set_hint_for_scene(self.scene);
    }
}
