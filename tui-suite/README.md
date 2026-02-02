# tui-suite

A terminal UI for Google Calendar, Gmail, and Drive with a built-in Markdown editor.

## Highlights
- Weekly calendar with overlap lanes, all-day strip, day selection, and a now indicator
- Gmail inbox (list + reader) and Markdown email composer
- Drive browser to search, open, and create Google Docs
- Markdown editor for local notes or Drive docs
- Built-in setup wizard for OAuth credentials

## Run It

### From source
```bash
cd tui-suite
cargo run
```

For an optimized binary:
```bash
cargo build --release
./target/release/tui-suite
```

### Via npm
```bash
npm install -g tui-suite
```

Or run without installing:
```bash
npx tui-suite
```

## Configuration & Login

### Option A: In-app setup (recommended)
1. Open the command palette (`Ctrl+P`) and run **Setup Credentials**.
2. Follow the wizard to create a Google Cloud project and enable APIs.
3. Use `o` to open browser links from the wizard.
4. Paste credentials with `Ctrl+V` on the input steps.
5. Press `Enter` to save.

On first run, the setup wizard opens automatically if credentials are missing.

### Option B: Manual config file
Create a config file at your OS config path:
- Linux: `~/.config/term-workspace/config.json`
- macOS/Windows: use the system config directory for `term-workspace`

Example:
```json
{
  "google_client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
  "google_client_secret": "YOUR_CLIENT_SECRET",
  "calendar_id": "primary",
  "notes_path": "notes.md"
}
```

Then use **Login Google** from the command palette to authenticate.

### Build-time credentials (optional)
You can embed defaults at build time:
```bash
TUI_SUITE_GOOGLE_CLIENT_ID=... \
TUI_SUITE_GOOGLE_CLIENT_SECRET=... \
cargo build --release
```

## Features

### Markdown Editor
- Local notes file (`notes_path`, default `notes.md`)
- Undo/redo (`Ctrl+Z` / `Ctrl+Y`)
- Formatting shortcuts: `Ctrl+B` bold, `Ctrl+I` italic, `Alt+1/2/3` headings
- `Ctrl+S` saves local files or updates the current Drive doc

### Calendar Week View
- Week-at-a-glance grid with hour + half-hour lines
- Overlap layout (events in lanes)
- All-day strip for multi-day events
- Selected day highlight and now indicator
- Sync with Google Calendar (`Ctrl+R` or `s`)

### Gmail Inbox
- Inbox list with unread markers
- Open and read message details
- Scroll long emails with `j/k`

### Email Composer
- To / Subject / Body fields
- Markdown body rendered to HTML on send
- `Ctrl+D` or `Ctrl+Enter` to send

### Drive Browser
- Type-to-search Drive docs
- `Enter` opens a doc in the editor
- `Ctrl+N` creates a new doc

### Google Integration
Requires enabling these APIs in Google Cloud:
- Google Calendar API
- Gmail API
- Google Drive API

## Command Palette
Open with `Ctrl+P`.

Available commands:
- Editor
- Calendar
- Inbox
- Compose Email
- Open Drive Doc
- Save
- Sync Calendar
- Undo / Redo
- Login Google
- Setup Credentials
- Quit

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `Tab` | Cycle views (Editor → Calendar → Inbox → Drive) |
| `Ctrl+P` | Command palette |
| `Ctrl+S` | Save current document |
| `Ctrl+R` | Sync calendar |
| `Alt+D` | Open Drive browser |
| `Ctrl+N` | New Drive doc |
| `Esc` | Close palette/modal |
| `q` | Quit |

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
| `h` / `l` or `←` / `→` | Previous/next day |
| `H` / `L` | Previous/next week |
| `j` / `k` or `↓` / `↑` | Scroll time down/up |
| `g` | Jump to now |
| `s` | Sync calendar |

### Inbox
| Key | Action |
|-----|--------|
| `j` / `k` or `↓` / `↑` | Navigate or scroll |
| `Enter` | Open selected email |
| `q` / `Esc` | Back |
| `r` | Refresh inbox |

### Compose
| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle fields |
| `Ctrl+D` | Send email |
| `Ctrl+Enter` | Send email (may not work in all terminals) |

### Drive Browser
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate |
| Type | Search docs |
| `Enter` | Open doc / create new doc |
| `Ctrl+N` | Create new doc |
| `Esc` | Clear search / cancel create |
| `Backspace` | Edit query |

### Setup Wizard
| Key | Action |
|-----|--------|
| `Enter` | Next step / save credentials |
| `Esc` | Back / exit on welcome |
| `o` | Open browser link (when available) |
| `Ctrl+V` | Paste into input fields |

## Architecture

```
src/
├── main.rs           # Entry point, terminal setup
├── lib.rs            # Module exports
├── error.rs          # AppError types
├── config.rs         # Configuration loading
├── app/              # Application state & input handling
├── engine/           # Async event loop & background tasks
├── ui/               # Layout, command palette, setup wizard, status bar
├── editor/           # Markdown editor with undo/redo
├── calendar/         # Week view, layout, rendering
├── mail/             # Inbox, composer, Gmail integration
├── drive/            # Drive browser and Docs integration
├── auth/             # OAuth flow & token storage
└── storage/          # Event caching
```

## Current Limitations
- Google features require OAuth setup and login
- Ctrl+Enter to send emails may not work in all terminals
- Some terminals may not support all Unicode line-drawing glyphs
