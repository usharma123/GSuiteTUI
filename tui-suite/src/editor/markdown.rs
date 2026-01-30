use pulldown_cmark::{html, Parser};
use ratatui::prelude::*;
use ratatui::text::{Line, Span};

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Applies markdown syntax highlighting to a single line for TUI display.
/// Returns a styled Line with appropriate colors and modifiers.
pub fn style_markdown_line(line: &str) -> Line<'static> {
    let owned = line.to_string();

    // Check for code block markers
    if owned.starts_with("```") {
        return Line::from(Span::styled(
            owned,
            Style::default().fg(Color::Rgb(209, 154, 102)), // Orange for code fence
        ));
    }

    // Check for blockquotes
    if owned.starts_with("> ") {
        let content = owned.strip_prefix("> ").unwrap_or(&owned);
        return Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Rgb(86, 182, 194))), // Cyan marker
            Span::styled(
                content.to_string(),
                Style::default()
                    .fg(Color::Rgb(152, 195, 121)) // Green text
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);
    }

    // Check for horizontal rule
    if owned == "---" || owned == "***" || owned == "___" {
        return Line::from(Span::styled(
            owned,
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Check for headings (ordered by specificity)
    if owned.starts_with("#### ") {
        return style_heading(&owned, 4, Color::Rgb(198, 120, 221)); // Purple
    }
    if owned.starts_with("### ") {
        return style_heading(&owned, 3, Color::Rgb(224, 108, 117)); // Red/Pink
    }
    if owned.starts_with("## ") {
        return style_heading(&owned, 2, Color::Rgb(97, 175, 239)); // Blue
    }
    if owned.starts_with("# ") {
        return style_heading(&owned, 1, Color::Rgb(86, 182, 194)); // Cyan
    }

    // Check for unordered list items
    if owned.starts_with("- ") || owned.starts_with("* ") || owned.starts_with("+ ") {
        let (marker, rest) = owned.split_at(2);
        let mut spans = vec![
            Span::styled(marker.to_string(), Style::default().fg(Color::Rgb(229, 192, 123))), // Yellow marker
        ];
        spans.extend(parse_inline_formatting(rest));
        return Line::from(spans);
    }

    // Check for ordered list items (1. 2. etc)
    if let Some(rest) = parse_ordered_list(&owned) {
        let marker_end = owned.len() - rest.len();
        let marker = &owned[..marker_end];
        let mut spans = vec![
            Span::styled(marker.to_string(), Style::default().fg(Color::Rgb(229, 192, 123))), // Yellow marker
        ];
        spans.extend(parse_inline_formatting(rest));
        return Line::from(spans);
    }

    // Check for checkbox items
    if owned.starts_with("- [ ] ") {
        let rest = owned.strip_prefix("- [ ] ").unwrap_or("");
        let mut spans = vec![
            Span::styled("- ", Style::default().fg(Color::Rgb(229, 192, 123))),
            Span::styled("[ ] ", Style::default().fg(Color::DarkGray)),
        ];
        spans.extend(parse_inline_formatting(rest));
        return Line::from(spans);
    }
    if owned.starts_with("- [x] ") || owned.starts_with("- [X] ") {
        let rest = &owned[6..];
        let mut spans = vec![
            Span::styled("- ", Style::default().fg(Color::Rgb(229, 192, 123))),
            Span::styled("[x] ", Style::default().fg(Color::Rgb(152, 195, 121))), // Green checkmark
        ];
        spans.extend(parse_inline_formatting(rest));
        return Line::from(spans);
    }

    // For lines with inline formatting, parse into spans
    let spans = parse_inline_formatting(&owned);
    Line::from(spans)
}

/// Style a heading line with the appropriate marker and color
fn style_heading(line: &str, level: usize, color: Color) -> Line<'static> {
    let marker = "#".repeat(level) + " ";
    let content = line.strip_prefix(&marker).unwrap_or(line);
    Line::from(vec![
        Span::styled(marker, Style::default().fg(color).add_modifier(Modifier::DIM)),
        Span::styled(
            content.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

/// Try to parse an ordered list marker (e.g., "1. ", "23. ")
fn parse_ordered_list(line: &str) -> Option<&str> {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    // Must start with digit
    if i >= chars.len() || !chars[i].is_ascii_digit() {
        return None;
    }

    // Consume digits
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }

    // Must be followed by ". "
    if i + 1 < chars.len() && chars[i] == '.' && chars[i + 1] == ' ' {
        Some(&line[i + 2..])
    } else {
        None
    }
}

/// Parses inline markdown formatting (bold, italic, code, links) into styled spans.
fn parse_inline_formatting(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut current = String::new();

    while i < chars.len() {
        // Check for inline code: `code`
        if chars[i] == '`' {
            // Flush current text
            if !current.is_empty() {
                spans.push(Span::styled(current.clone(), Style::default().fg(Color::Rgb(171, 178, 191))));
                current.clear();
            }

            // Find closing backtick
            let start = i + 1;
            i += 1;
            while i < chars.len() && chars[i] != '`' {
                i += 1;
            }

            if i < chars.len() {
                let code_text: String = chars[start..i].iter().collect();
                spans.push(Span::styled(
                    format!("`{}`", code_text),
                    Style::default()
                        .fg(Color::Rgb(209, 154, 102)) // Orange
                        .bg(Color::Rgb(40, 44, 52)),   // Dark background
                ));
                i += 1;
            } else {
                // No closing backtick, treat as literal
                current.push('`');
                i = start;
            }
            continue;
        }

        // Check for links: [text](url)
        if chars[i] == '[' {
            // Find closing bracket
            let mut bracket_end = i + 1;
            while bracket_end < chars.len() && chars[bracket_end] != ']' {
                bracket_end += 1;
            }

            // Check if followed by (url)
            if bracket_end + 1 < chars.len() && chars[bracket_end + 1] == '(' {
                let mut paren_end = bracket_end + 2;
                while paren_end < chars.len() && chars[paren_end] != ')' {
                    paren_end += 1;
                }

                if paren_end < chars.len() {
                    // Valid link found
                    if !current.is_empty() {
                        spans.push(Span::styled(current.clone(), Style::default().fg(Color::Rgb(171, 178, 191))));
                        current.clear();
                    }

                    let link_text: String = chars[i + 1..bracket_end].iter().collect();
                    let url: String = chars[bracket_end + 2..paren_end].iter().collect();

                    spans.push(Span::styled("[", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled(
                        link_text,
                        Style::default()
                            .fg(Color::Rgb(97, 175, 239)) // Blue
                            .add_modifier(Modifier::UNDERLINED),
                    ));
                    spans.push(Span::styled("](", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled(
                        url,
                        Style::default().fg(Color::Rgb(152, 195, 121)), // Green
                    ));
                    spans.push(Span::styled(")", Style::default().fg(Color::DarkGray)));

                    i = paren_end + 1;
                    continue;
                }
            }
        }

        // Check for bold: **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            // Flush current text
            if !current.is_empty() {
                spans.push(Span::styled(current.clone(), Style::default().fg(Color::Rgb(171, 178, 191))));
                current.clear();
            }

            // Find closing **
            let start = i + 2;
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }

            if i + 1 < chars.len() {
                let bold_text: String = chars[start..i].iter().collect();
                spans.push(Span::styled(
                    "**",
                    Style::default().fg(Color::DarkGray),
                ));
                spans.push(Span::styled(
                    bold_text,
                    Style::default()
                        .fg(Color::Rgb(229, 192, 123)) // Yellow/Gold
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    "**",
                    Style::default().fg(Color::DarkGray),
                ));
                i += 2;
            } else {
                // No closing **, treat as literal
                current.push_str("**");
                i = start;
            }
            continue;
        }

        // Check for italic: *text* (single asterisk, not followed by another)
        if chars[i] == '*' && (i + 1 >= chars.len() || chars[i + 1] != '*') {
            // Flush current text
            if !current.is_empty() {
                spans.push(Span::styled(current.clone(), Style::default().fg(Color::Rgb(171, 178, 191))));
                current.clear();
            }

            // Find closing *
            let start = i + 1;
            i += 1;
            while i < chars.len() && chars[i] != '*' {
                i += 1;
            }

            if i < chars.len() {
                let italic_text: String = chars[start..i].iter().collect();
                spans.push(Span::styled(
                    "*",
                    Style::default().fg(Color::DarkGray),
                ));
                spans.push(Span::styled(
                    italic_text,
                    Style::default()
                        .fg(Color::Rgb(198, 120, 221)) // Purple
                        .add_modifier(Modifier::ITALIC),
                ));
                spans.push(Span::styled(
                    "*",
                    Style::default().fg(Color::DarkGray),
                ));
                i += 1;
            } else {
                // No closing *, treat as literal
                current.push('*');
                i = start;
            }
            continue;
        }

        // Check for strikethrough: ~~text~~
        if i + 1 < chars.len() && chars[i] == '~' && chars[i + 1] == '~' {
            if !current.is_empty() {
                spans.push(Span::styled(current.clone(), Style::default().fg(Color::Rgb(171, 178, 191))));
                current.clear();
            }

            let start = i + 2;
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '~' && chars[i + 1] == '~') {
                i += 1;
            }

            if i + 1 < chars.len() {
                let strike_text: String = chars[start..i].iter().collect();
                spans.push(Span::styled(
                    format!("~~{}~~", strike_text),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT),
                ));
                i += 2;
            } else {
                current.push_str("~~");
                i = start;
            }
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    // Flush remaining text with default styling
    if !current.is_empty() {
        spans.push(Span::styled(current, Style::default().fg(Color::Rgb(171, 178, 191)))); // Light gray default text
    }

    if spans.is_empty() {
        spans.push(Span::raw(String::new()));
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold_produces_strong() {
        let md = "**bold text**";
        let html = markdown_to_html(md);
        assert!(html.contains("<strong>"), "Expected <strong> in: {}", html);
        assert!(html.contains("</strong>"), "Expected </strong> in: {}", html);
    }

    #[test]
    fn test_heading_produces_h1() {
        let md = "# Heading";
        let html = markdown_to_html(md);
        assert!(html.contains("<h1>"), "Expected <h1> in: {}", html);
    }

    #[test]
    fn test_non_empty_output() {
        let md = "Some text";
        let html = markdown_to_html(md);
        assert!(!html.is_empty(), "Expected non-empty output");
        assert!(html.contains("Some text"));
    }

    #[test]
    fn test_italic() {
        let md = "*italic*";
        let html = markdown_to_html(md);
        assert!(html.contains("<em>"), "Expected <em> in: {}", html);
    }

    #[test]
    fn test_link() {
        let md = "[link](https://example.com)";
        let html = markdown_to_html(md);
        assert!(html.contains("href="), "Expected href in: {}", html);
    }
}
