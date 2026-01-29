use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Global
    Quit,
    CycleScene,
    OpenPalette,
    ClosePalette,
    Save,

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

    // Mail
    SendMail,

    // Palette
    PaletteUp,
    PaletteDown,
    PaletteSelect,
    PaletteType(char),
    PaletteBackspace,

    // No action
    None,
}

pub fn map_global_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') if key.modifiers.is_empty() => Action::Quit,
        KeyCode::Tab => Action::CycleScene,
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::OpenPalette,
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Save,
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
        KeyCode::Char('1') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::InsertHeading(1)
        }
        KeyCode::Char('2') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::InsertHeading(2)
        }
        KeyCode::Char('3') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
        KeyCode::Char('h') => Action::PrevWeek,
        KeyCode::Char('l') => Action::NextWeek,
        KeyCode::Char('j') => Action::ScrollDown,
        KeyCode::Char('k') => Action::ScrollUp,
        KeyCode::Char('g') => Action::JumpToNow,
        KeyCode::Enter => Action::SelectEvent,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Up => Action::ScrollUp,
        KeyCode::Down => Action::ScrollDown,
        KeyCode::Left => Action::PrevWeek,
        KeyCode::Right => Action::NextWeek,
        _ => Action::None,
    }
}

pub fn map_compose_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SendMail,
        _ => map_editor_key(key),
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
