use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::config::Config;
use crate::error::Result;

// Google Cloud Console deep-link URLs
pub const GCP_NEW_PROJECT: &str = "https://console.cloud.google.com/projectcreate";
pub const GCP_ENABLE_APIS: &str = "https://console.cloud.google.com/apis/library";
pub const GCP_OAUTH_CONSENT: &str = "https://console.cloud.google.com/apis/credentials/consent";
pub const GCP_CREDENTIALS: &str = "https://console.cloud.google.com/apis/credentials";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetupStep {
    #[default]
    Welcome,
    CreateProject,
    EnableApis,
    ConfigureConsent,
    CreateCredentials,
    InputClientId,
    InputClientSecret,
    Complete,
}

impl SetupStep {
    pub fn step_number(&self) -> usize {
        match self {
            SetupStep::Welcome => 1,
            SetupStep::CreateProject => 2,
            SetupStep::EnableApis => 3,
            SetupStep::ConfigureConsent => 4,
            SetupStep::CreateCredentials => 5,
            SetupStep::InputClientId => 6,
            SetupStep::InputClientSecret => 7,
            SetupStep::Complete => 8,
        }
    }

    pub const TOTAL_STEPS: usize = 8;

    pub fn next(&self) -> Self {
        match self {
            SetupStep::Welcome => SetupStep::CreateProject,
            SetupStep::CreateProject => SetupStep::EnableApis,
            SetupStep::EnableApis => SetupStep::ConfigureConsent,
            SetupStep::ConfigureConsent => SetupStep::CreateCredentials,
            SetupStep::CreateCredentials => SetupStep::InputClientId,
            SetupStep::InputClientId => SetupStep::InputClientSecret,
            SetupStep::InputClientSecret => SetupStep::Complete,
            SetupStep::Complete => SetupStep::Complete,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SetupStep::Welcome => SetupStep::Welcome,
            SetupStep::CreateProject => SetupStep::Welcome,
            SetupStep::EnableApis => SetupStep::CreateProject,
            SetupStep::ConfigureConsent => SetupStep::EnableApis,
            SetupStep::CreateCredentials => SetupStep::ConfigureConsent,
            SetupStep::InputClientId => SetupStep::CreateCredentials,
            SetupStep::InputClientSecret => SetupStep::InputClientId,
            SetupStep::Complete => SetupStep::InputClientSecret,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            SetupStep::Welcome => "Welcome",
            SetupStep::CreateProject => "Create Google Cloud Project",
            SetupStep::EnableApis => "Enable APIs",
            SetupStep::ConfigureConsent => "Configure OAuth Consent",
            SetupStep::CreateCredentials => "Create OAuth Credentials",
            SetupStep::InputClientId => "Enter Client ID",
            SetupStep::InputClientSecret => "Enter Client Secret",
            SetupStep::Complete => "Setup Complete",
        }
    }

    pub fn has_browser_action(&self) -> bool {
        matches!(
            self,
            SetupStep::CreateProject
                | SetupStep::EnableApis
                | SetupStep::ConfigureConsent
                | SetupStep::CreateCredentials
        )
    }

    pub fn browser_url(&self) -> Option<&'static str> {
        match self {
            SetupStep::CreateProject => Some(GCP_NEW_PROJECT),
            SetupStep::EnableApis => Some(GCP_ENABLE_APIS),
            SetupStep::ConfigureConsent => Some(GCP_OAUTH_CONSENT),
            SetupStep::CreateCredentials => Some(GCP_CREDENTIALS),
            _ => None,
        }
    }

    pub fn is_input_step(&self) -> bool {
        matches!(self, SetupStep::InputClientId | SetupStep::InputClientSecret)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupField {
    ClientId,
    ClientSecret,
}

#[derive(Debug, Clone)]
pub struct SetupWizardState {
    pub step: SetupStep,
    pub client_id: String,
    pub client_secret: String,
    pub active_field: SetupField,
    pub error: Option<String>,
}

impl Default for SetupWizardState {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupWizardState {
    pub fn new() -> Self {
        Self {
            step: SetupStep::Welcome,
            client_id: String::new(),
            client_secret: String::new(),
            active_field: SetupField::ClientId,
            error: None,
        }
    }

    pub fn advance(&mut self) -> bool {
        self.error = None;

        // Validate input on input steps
        if self.step == SetupStep::InputClientId && self.client_id.trim().is_empty() {
            self.error = Some("Client ID is required".to_string());
            return false;
        }
        if self.step == SetupStep::InputClientSecret && self.client_secret.trim().is_empty() {
            self.error = Some("Client Secret is required".to_string());
            return false;
        }

        self.step = self.step.next();
        true
    }

    pub fn go_back(&mut self) {
        self.error = None;
        self.step = self.step.prev();
    }

    pub fn open_browser(&self) -> std::result::Result<(), String> {
        if let Some(url) = self.step.browser_url() {
            open::that(url).map_err(|e| format!("Failed to open browser: {e}"))
        } else {
            Err("No URL for this step".to_string())
        }
    }

    pub fn type_char(&mut self, c: char) {
        self.error = None;
        match self.step {
            SetupStep::InputClientId => self.client_id.push(c),
            SetupStep::InputClientSecret => self.client_secret.push(c),
            _ => {}
        }
    }

    pub fn backspace(&mut self) {
        self.error = None;
        match self.step {
            SetupStep::InputClientId => {
                self.client_id.pop();
            }
            SetupStep::InputClientSecret => {
                self.client_secret.pop();
            }
            _ => {}
        }
    }

    pub fn paste(&mut self, text: &str) {
        self.error = None;
        // Clean up the pasted text (remove newlines, trim)
        let cleaned = text.trim().replace(['\n', '\r'], "");
        match self.step {
            SetupStep::InputClientId => self.client_id.push_str(&cleaned),
            SetupStep::InputClientSecret => self.client_secret.push_str(&cleaned),
            _ => {}
        }
    }

    pub fn clear_field(&mut self) {
        match self.step {
            SetupStep::InputClientId => self.client_id.clear(),
            SetupStep::InputClientSecret => self.client_secret.clear(),
            _ => {}
        }
    }

    pub fn is_complete(&self) -> bool {
        self.step == SetupStep::Complete
    }

    pub fn save_credentials(&self) -> Result<()> {
        let mut config = Config::load().unwrap_or_default();
        config.google_client_id = Some(self.client_id.trim().to_string());
        config.google_client_secret = Some(self.client_secret.trim().to_string());
        config.save()
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Center the wizard - use 80% width and 70% height
        let popup_area = centered_rect(80, 70, area);

        // Clear the area
        f.render_widget(Clear, popup_area);

        // Create main layout: title, content, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title + progress
                Constraint::Min(1),    // Content
                Constraint::Length(3), // Footer (controls)
            ])
            .split(popup_area);

        // Render title bar with step progress
        self.render_title(f, chunks[0]);

        // Render step content
        self.render_content(f, chunks[1]);

        // Render footer with controls
        self.render_footer(f, chunks[2]);
    }

    fn render_title(&self, f: &mut Frame, area: Rect) {
        let title = format!(
            "{} - Step {} of {}",
            self.step.title(),
            self.step.step_number(),
            SetupStep::TOTAL_STEPS
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        // Progress bar
        let progress = self.step.step_number() as f64 / SetupStep::TOTAL_STEPS as f64;
        let inner = block.inner(area);
        f.render_widget(block, area);

        let progress_width = ((inner.width as f64 * progress) as u16).min(inner.width);
        let progress_bar = Paragraph::new("=".repeat(progress_width as usize))
            .style(Style::default().fg(Color::Green));
        f.render_widget(progress_bar, inner);
    }

    fn render_content(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::LEFT | Borders::RIGHT);
        let inner = block.inner(area);
        f.render_widget(block, area);

        match self.step {
            SetupStep::Welcome => self.render_welcome(f, inner),
            SetupStep::CreateProject => self.render_create_project(f, inner),
            SetupStep::EnableApis => self.render_enable_apis(f, inner),
            SetupStep::ConfigureConsent => self.render_configure_consent(f, inner),
            SetupStep::CreateCredentials => self.render_create_credentials(f, inner),
            SetupStep::InputClientId => self.render_input_client_id(f, inner),
            SetupStep::InputClientSecret => self.render_input_client_secret(f, inner),
            SetupStep::Complete => self.render_complete(f, inner),
        }
    }

    fn render_welcome(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Welcome to tui-suite!",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("tui-suite needs Google API credentials to sync your"),
            Line::from("Calendar, send emails, and access Drive."),
            Line::from(""),
            Line::from("You'll create a free Google Cloud project and OAuth"),
            Line::from("credentials. This is a one-time setup."),
            Line::from(""),
            Line::from(Span::styled(
                "What you'll need:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  - A Google account"),
            Line::from("  - Access to Google Cloud Console (free)"),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] to start the setup process.",
                Style::default().fg(Color::Green),
            )),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    fn render_create_project(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Step 1: Create a Google Cloud Project",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("1. Press [o] to open Google Cloud Console"),
            Line::from("2. Sign in with your Google account"),
            Line::from("3. Enter a project name (e.g., \"tui-suite\")"),
            Line::from("4. Click \"Create\""),
            Line::from(""),
            Line::from(Span::styled(
                "Note: If you already have a Google Cloud project,",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "you can use it - just skip to the next step.",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "URL: console.cloud.google.com/projectcreate",
                Style::default().fg(Color::Blue),
            )),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    fn render_enable_apis(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Step 2: Enable Required APIs",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("1. Press [o] to open the API Library"),
            Line::from("2. Make sure your project is selected (top dropdown)"),
            Line::from("3. Search for and enable these APIs:"),
            Line::from(""),
            Line::from(Span::styled(
                "   - Google Calendar API",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "   - Gmail API",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "   - Google Drive API",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("4. Click each API, then click \"Enable\""),
            Line::from(""),
            Line::from(Span::styled(
                "URL: console.cloud.google.com/apis/library",
                Style::default().fg(Color::Blue),
            )),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    fn render_configure_consent(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Step 3: Configure OAuth Consent Screen",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("1. Press [o] to open OAuth consent screen"),
            Line::from("2. Select \"External\" user type, click Create"),
            Line::from("3. Fill in required fields:"),
            Line::from("   - App name: \"tui-suite\""),
            Line::from("   - User support email: your email"),
            Line::from("   - Developer contact: your email"),
            Line::from("4. Click \"Save and Continue\" through scopes"),
            Line::from("5. On Test Users, add your email address"),
            Line::from("6. Click \"Save and Continue\""),
            Line::from(""),
            Line::from(Span::styled(
                "Note: App stays in \"Testing\" mode - that's fine!",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "URL: console.cloud.google.com/apis/credentials/consent",
                Style::default().fg(Color::Blue),
            )),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    fn render_create_credentials(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Step 4: Create OAuth Client Credentials",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("1. Press [o] to open Credentials page"),
            Line::from("2. Click \"+ CREATE CREDENTIALS\" at the top"),
            Line::from("3. Select \"OAuth client ID\""),
            Line::from("4. Application type: \"Desktop app\""),
            Line::from("5. Name: \"tui-suite\" (or anything you like)"),
            Line::from("6. Click \"Create\""),
            Line::from(""),
            Line::from(Span::styled(
                "A popup will show your Client ID and Client Secret.",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "Keep this window open - you'll need both values next!",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "URL: console.cloud.google.com/apis/credentials",
                Style::default().fg(Color::Blue),
            )),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    fn render_input_client_id(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Instructions
                Constraint::Length(1), // Spacing
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Error or hint
                Constraint::Min(0),    // Padding
            ])
            .split(area);

        let instructions = Paragraph::new(vec![
            Line::from("Paste your OAuth Client ID from the Google Cloud Console:"),
            Line::from(Span::styled(
                "(It looks like: 123456789-abc.apps.googleusercontent.com)",
                Style::default().fg(Color::Gray),
            )),
        ]);
        f.render_widget(instructions, chunks[0]);

        // Input field with border
        let display_value = if self.client_id.len() > 50 {
            format!("...{}", &self.client_id[self.client_id.len() - 47..])
        } else {
            self.client_id.clone()
        };

        let input = Paragraph::new(format!("{}_", display_value))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Client ID"),
            )
            .style(Style::default().fg(Color::White));
        f.render_widget(input, chunks[2]);

        // Error or hint
        if let Some(ref error) = self.error {
            let error_text =
                Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
            f.render_widget(error_text, chunks[3]);
        } else {
            let hint = Paragraph::new("Press Ctrl+V to paste, Enter to continue")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(hint, chunks[3]);
        }
    }

    fn render_input_client_secret(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Instructions
                Constraint::Length(1), // Spacing
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Error or hint
                Constraint::Min(0),    // Padding
            ])
            .split(area);

        let instructions = Paragraph::new(vec![
            Line::from("Paste your OAuth Client Secret from the Google Cloud Console:"),
            Line::from(Span::styled(
                "(It's shown alongside the Client ID in the popup)",
                Style::default().fg(Color::Gray),
            )),
        ]);
        f.render_widget(instructions, chunks[0]);

        // Input field with border - mask the secret
        let masked = "*".repeat(self.client_secret.len().min(40));
        let display_value = if self.client_secret.is_empty() {
            String::new()
        } else if self.client_secret.len() > 40 {
            format!("{}...", masked)
        } else {
            masked
        };

        let input = Paragraph::new(format!("{}_", display_value))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Client Secret"),
            )
            .style(Style::default().fg(Color::White));
        f.render_widget(input, chunks[2]);

        // Error or hint
        if let Some(ref error) = self.error {
            let error_text =
                Paragraph::new(error.as_str()).style(Style::default().fg(Color::Red));
            f.render_widget(error_text, chunks[3]);
        } else {
            let hint = Paragraph::new("Press Ctrl+V to paste, Enter to continue")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(hint, chunks[3]);
        }
    }

    fn render_complete(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Setup Complete!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Your Google OAuth credentials have been saved."),
            Line::from(""),
            Line::from("You can now use tui-suite to:"),
            Line::from(Span::styled(
                "  - Sync and view your Google Calendar",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "  - Send and read emails via Gmail",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "  - Access and edit Google Drive documents",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Next: Use the Command Palette (Ctrl+P) and select",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "\"Login Google\" to authenticate with your account.",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("Press [Enter] to start using tui-suite!"),
        ];

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let controls = match self.step {
            SetupStep::Welcome => "[Enter] Start setup    [Esc] Skip (use without Google)",
            SetupStep::Complete => "[Enter] Finish",
            _ if self.step.has_browser_action() => {
                "[o] Open browser    [Enter] Continue    [Esc] Back"
            }
            _ if self.step.is_input_step() => "[Ctrl+V] Paste    [Enter] Continue    [Esc] Back",
            _ => "[Enter] Continue    [Esc] Back",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let footer = Paragraph::new(controls)
            .block(block)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
