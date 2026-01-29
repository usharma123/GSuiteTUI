#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scene {
    #[default]
    Editor,
    CalendarWeek,
    MailCompose,
    MailInbox,
}

impl Scene {
    pub fn next(self) -> Self {
        match self {
            Scene::Editor => Scene::CalendarWeek,
            Scene::CalendarWeek => Scene::Editor,
            Scene::MailCompose => Scene::Editor,
            Scene::MailInbox => Scene::Editor,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Scene::Editor => "Editor",
            Scene::CalendarWeek => "Calendar",
            Scene::MailCompose => "Compose Email",
            Scene::MailInbox => "Inbox",
        }
    }
}
