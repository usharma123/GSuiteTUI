# tui-suite

A terminal UI for Google Calendar and Gmail.

## Installation

```bash
npm install -g tui-suite
```

Or use directly with npx:

```bash
npx tui-suite
```

## Features

- **Calendar View**: Weekly calendar with Google Calendar sync
- **Email Inbox**: View and read Gmail messages
- **Compose Email**: Write and send emails with markdown support
- **Notes Editor**: Built-in markdown editor

## Setup

1. Create a Google Cloud project and enable Calendar and Gmail APIs
2. Create OAuth 2.0 credentials (Desktop app)
3. Create config file at `~/.config/term-workspace/config.json`:

```json
{
  "google_client_id": "YOUR_CLIENT_ID",
  "google_client_secret": "YOUR_CLIENT_SECRET",
  "calendar_id": "primary"
}
```

4. Run `tui-suite` and use the command palette (Ctrl+P) to login

## Keybindings

- `Tab` - Cycle between scenes (Editor, Calendar, Inbox)
- `Ctrl+P` - Open command palette
- `Ctrl+S` - Save notes

### Calendar
- `h/l` or `Left/Right` - Navigate days
- `H/L` - Navigate weeks
- `j/k` or `Up/Down` - Scroll time
- `g` - Jump to now

### Inbox
- `j/k` - Navigate emails
- `Enter` - Open email
- `q/Esc` - Close/back
- `r` - Refresh

### Compose
- `Tab` - Cycle fields
- `Ctrl+D` - Send email

## Supported Platforms

- macOS (Intel & Apple Silicon)
- Linux (x64 & ARM64)
- Windows (x64)

## License

MIT
