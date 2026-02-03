use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Global
    Quit,
    CycleScene,
    OpenPalette,
    ClosePalette,
    Save,
    SyncCalendar,
    OpenDriveBrowser,
    CreateDriveDoc,
    OpenSheets,

    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Home,
    End,

    // Editor
    InsertChar(char),
    Newline,
    Backspace,
    Delete,
    Undo,
    Redo,
    ToggleBold,
    ToggleItalic,
    InsertHeading(u8),

    // Calendar
    PrevWeek,
    NextWeek,
    ScrollUp,
    ScrollDown,
    JumpToNow,
    SelectEvent,
    Search,
    SelectPrevDay,
    SelectNextDay,

    // Mail
    SendMail,
    CycleField,
    RefreshInbox,
    OpenEmail,
    CloseEmail,

    // Palette
    PaletteUp,
    PaletteDown,
    PaletteSelect,
    PaletteType(char),
    PaletteBackspace,

    // Drive browser
    DriveUp,
    DriveDown,
    DriveSelect,
    DriveType(char),
    DriveBackspace,
    DriveCancel,

    // Sheets
    SheetRefresh,
    SheetTabPrev,
    SheetTabNext,
    SheetEnterEdit,
    SheetCancelEdit,

    // No action
    None,
}

pub fn map_global_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') if key.modifiers.is_empty() => Action::Quit,
        KeyCode::Tab => Action::CycleScene,
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::OpenPalette,
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Save,
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SyncCalendar,
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => Action::OpenDriveBrowser,
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::ALT) => Action::OpenSheets,
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::CreateDriveDoc,
        KeyCode::Esc => Action::ClosePalette,
        _ => Action::None,
    }
}

pub fn map_editor_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => Action::MoveUp,
        KeyCode::Down => Action::MoveDown,
        KeyCode::Left => Action::MoveLeft,
        KeyCode::Right => Action::MoveRight,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Delete => Action::Delete,
        KeyCode::Enter => Action::Newline,
        KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Undo,
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Redo,
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::ToggleBold,
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::ToggleItalic,
        // Note: Ctrl+number combinations don't work in most terminals,
        // so we use Alt+number instead for headings
        KeyCode::Char('1') if key.modifiers.contains(KeyModifiers::ALT) => {
            Action::InsertHeading(1)
        }
        KeyCode::Char('2') if key.modifiers.contains(KeyModifiers::ALT) => {
            Action::InsertHeading(2)
        }
        KeyCode::Char('3') if key.modifiers.contains(KeyModifiers::ALT) => {
            Action::InsertHeading(3)
        }
        KeyCode::Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            Action::InsertChar(c)
        }
        _ => Action::None,
    }
}

pub fn map_calendar_key(key: KeyEvent) -> Action {
    match key.code {
        // Day navigation (h/l or arrow keys)
        KeyCode::Char('h') => Action::SelectPrevDay,
        KeyCode::Char('l') => Action::SelectNextDay,
        KeyCode::Left => Action::SelectPrevDay,
        KeyCode::Right => Action::SelectNextDay,
        // Week navigation (H/L or shift+arrow)
        KeyCode::Char('H') => Action::PrevWeek,
        KeyCode::Char('L') => Action::NextWeek,
        // Scroll (j/k or up/down arrows)
        KeyCode::Char('j') => Action::ScrollDown,
        KeyCode::Char('k') => Action::ScrollUp,
        KeyCode::Up => Action::ScrollUp,
        KeyCode::Down => Action::ScrollDown,
        // Other
        KeyCode::Char('g') => Action::JumpToNow,
        KeyCode::Enter => Action::SelectEvent,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char('s') => Action::SyncCalendar,
        _ => Action::None,
    }
}

pub fn map_compose_key(key: KeyEvent) -> Action {
    match key.code {
        // Ctrl+Enter (may not work in all terminals)
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SendMail,
        // Ctrl+D as alternative send binding (works reliably)
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SendMail,
        KeyCode::Tab | KeyCode::BackTab => Action::CycleField,
        _ => map_editor_key(key),
    }
}

pub fn map_inbox_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Enter => Action::OpenEmail,
        KeyCode::Char('q') | KeyCode::Esc => Action::CloseEmail,
        KeyCode::Char('r') => Action::RefreshInbox,
        _ => Action::None,
    }
}

pub fn map_palette_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => Action::PaletteUp,
        KeyCode::Down => Action::PaletteDown,
        KeyCode::Enter => Action::PaletteSelect,
        KeyCode::Esc => Action::ClosePalette,
        KeyCode::Backspace => Action::PaletteBackspace,
        KeyCode::Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            Action::PaletteType(c)
        }
        _ => Action::None,
    }
}

pub fn map_drive_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => Action::DriveUp,
        KeyCode::Down => Action::DriveDown,
        KeyCode::Enter => Action::DriveSelect,
        KeyCode::Esc => Action::DriveCancel,
        KeyCode::Backspace => Action::DriveBackspace,
        KeyCode::Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            Action::DriveType(c)
        }
        _ => Action::None,
    }
}

pub fn map_sheets_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => Action::MoveUp,
        KeyCode::Down => Action::MoveDown,
        KeyCode::Left => Action::MoveLeft,
        KeyCode::Right => Action::MoveRight,
        KeyCode::Enter => Action::SheetEnterEdit,
        KeyCode::Esc => Action::SheetCancelEdit,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Char('r') => Action::SheetRefresh,
        KeyCode::Char('[') => Action::SheetTabPrev,
        KeyCode::Char(']') => Action::SheetTabNext,
        KeyCode::Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            Action::InsertChar(c)
        }
        _ => Action::None,
    }
}
