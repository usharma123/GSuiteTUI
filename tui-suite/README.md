# term-workspace

A terminal-based workspace application with a Markdown editor, calendar week view, and Google integrations.

## Features

### Markdown Editor
- Full text editing with cursor navigation
- Undo/Redo support (`Ctrl+Z` / `Ctrl+Y`)
- Markdown formatting shortcuts:
  - `Ctrl+B` - Insert bold markers (`**text**`)
  - `Ctrl+I` - Insert italic markers (`*text*`)
  - `Alt+1/2/3` - Insert heading markers (`#`, `##`, `###`)
- Auto-save with `Ctrl+S`
- File saved to `notes.md` in current directory

### Calendar Week View
- Week-at-a-glance display with time grid
- **Overlap layout**: Events that overlap are displayed side-by-side in lanes
- Current time indicator (red line)
- Navigation:
  - `h` or `←` - Previous week
  - `l` or `→` - Next week
  - `j` or `↓` - Scroll time down
  - `k` or `↑` - Scroll time up
  - `g` - Jump to current time
- Displays mock events on startup (real sync requires Google credentials)

### Command Palette
- Open with `Ctrl+P`
- Fuzzy search through commands
- Available commands:
  - **Editor** - Switch to editor view
  - **Calendar** - Switch to calendar view
  - **Compose Email** - Open email composer
  - **Save** - Save current document
  - **Sync Calendar** - Sync with Google Calendar
  - **Open Drive Doc** - Search Google Docs
  - **Undo** / **Redo** - Undo/redo in editor
  - **Login Google** - Start OAuth flow
  - **Quit** - Exit application

### Email Composer
- Compose emails with To, Subject, and Body fields
- Body supports Markdown (rendered to HTML when sent)
- `Ctrl+Enter` to send (requires Google credentials)

### Drive Browser
- Browse and search your Google Docs
- `Alt+D` to open Drive view
- Type to filter, `Enter` to open
- `Ctrl+N` to create a new Google Doc

### Google Integration (requires setup)
- **Google Calendar**: Sync events with incremental sync tokens
- **Gmail**: Send emails with Markdown body (converted to HTML)
- **Drive**: Browse and create Google Docs

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `Tab` | Cycle between views |
| `Ctrl+P` | Open command palette |
| `Ctrl+S` | Save document |
| `Ctrl+R` | Sync calendar |
| `Alt+D` | Open Drive browser |
| `Ctrl+N` | New Drive doc |
| `Esc` | Close palette/modal |
| `q` | Quit application |

### Editor
| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| `Home` / `End` | Go to line start/end |
| `Enter` | New line |
| `Backspace` | Delete character |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+B` | Insert bold |
| `Ctrl+I` | Insert italic |
| `Alt+1/2/3` | Insert heading |

### Calendar
| Key | Action |
|-----|--------|
| `h` / `l` | Previous/next week |
| `j` / `k` | Scroll time down/up |
| `g` | Jump to now |
| `s` | Sync calendar |

### Command Palette
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate options |
| `Enter` | Select command |
| `Esc` | Close palette |
| Type | Filter commands |

### Drive Browser
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate |
| `Enter` | Open selected doc |
| Type | Search docs |
| `Ctrl+N` | New doc |
| `Esc` | Clear search / cancel create |

## Installation

```bash
cd tui-suite
cargo build --release
./target/release/tui-suite
```

## Google API Setup (Optional)

To enable Google Calendar sync and Gmail send:

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Enable the **Google Calendar API** and **Gmail API**
4. Create OAuth 2.0 credentials (Desktop application)
5. Create config file at `~/.config/term-workspace/config.json`:

```json
{
  "google_client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
  "google_client_secret": "YOUR_CLIENT_SECRET",
  "calendar_id": "primary",
  "notes_path": "notes.md"
}
```

6. Use Command Palette → "Login Google" to authenticate

## Architecture

```
src/
├── main.rs           # Entry point, terminal setup
├── lib.rs            # Module exports
├── error.rs          # AppError types
├── config.rs         # Configuration loading
├── app/              # Application state & input handling
├── engine/           # Async event loop & background tasks
├── ui/               # Layout, command palette, status bar
├── editor/           # Markdown editor with undo/redo
├── calendar/         # Week view with overlap layout
├── mail/             # Email composer & MIME builder
├── auth/             # OAuth flow & token storage
└── storage/          # Event caching
```

## Current Limitations

- Calendar displays mock data until Google credentials are configured
- Email sending requires Google credentials
- Single file editing (notes.md)
