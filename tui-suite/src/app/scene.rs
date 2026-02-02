#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scene {
    #[default]
    Editor,
    CalendarWeek,
    MailCompose,
    MailInbox,
    DriveBrowser,
    Setup,
}

impl Scene {
    pub fn next(self) -> Self {
        match self {
            Scene::Editor => Scene::CalendarWeek,
            Scene::CalendarWeek => Scene::MailInbox,
            Scene::MailInbox => Scene::DriveBrowser,
            Scene::DriveBrowser => Scene::Editor,
            Scene::MailCompose => Scene::Editor,
            // Setup is modal - doesn't participate in Tab cycling
            Scene::Setup => Scene::Setup,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Scene::Editor => "Editor",
            Scene::CalendarWeek => "Calendar",
            Scene::MailCompose => "Compose Email",
            Scene::MailInbox => "Inbox",
            Scene::DriveBrowser => "Drive",
            Scene::Setup => "Setup",
        }
    }
}
