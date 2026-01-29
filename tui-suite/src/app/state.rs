use tokio::sync::mpsc;

use crate::calendar::CalendarState;
use crate::config::Config;
use crate::editor::EditorState;
use crate::engine::TaskResult;
use crate::mail::ComposeState;
use crate::ui::{PaletteState, StatusLine};

use super::scene::Scene;

pub struct AppState {
    pub scene: Scene,
    pub editor: EditorState,
    pub calendar: CalendarState,
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
        Self {
            scene: Scene::default(),
            editor: EditorState::new(notes_path),
            calendar: CalendarState::mock(),
            compose: None,
            palette: None,
            status: StatusLine::new("Tab: switch | Ctrl+S: save | Ctrl+P: palette | q: quit"),
            should_quit: false,
            config,
            task_tx,
        }
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
        self.status = StatusLine::new(msg);
    }
}
