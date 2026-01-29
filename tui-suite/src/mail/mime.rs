use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use uuid::Uuid;

use crate::editor::markdown::markdown_to_html;

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body_markdown: String,
}

pub fn build_mime_message(msg: &EmailMessage, from: &str) -> String {
    let boundary = format!("boundary_{}", Uuid::new_v4().simple());
    let html_body = markdown_to_html(&msg.body_markdown);
    let plain_body = &msg.body_markdown;

    // Build RFC 2822 compliant message
    let mut message = String::new();

    // Headers
    message.push_str(&format!("From: {}\r\n", from));
    message.push_str(&format!("To: {}\r\n", msg.to));
    message.push_str(&format!("Subject: {}\r\n", encode_header_value(&msg.subject)));
    message.push_str("MIME-Version: 1.0\r\n");
    message.push_str(&format!(
        "Content-Type: multipart/alternative; boundary=\"{}\"\r\n",
        boundary
    ));
    message.push_str("\r\n");

    // Plain text part
    message.push_str(&format!("--{}\r\n", boundary));
    message.push_str("Content-Type: text/plain; charset=\"UTF-8\"\r\n");
    message.push_str("Content-Transfer-Encoding: 8bit\r\n");
    message.push_str("\r\n");
    message.push_str(plain_body);
    message.push_str("\r\n");

    // HTML part
    message.push_str(&format!("--{}\r\n", boundary));
    message.push_str("Content-Type: text/html; charset=\"UTF-8\"\r\n");
    message.push_str("Content-Transfer-Encoding: 8bit\r\n");
    message.push_str("\r\n");
    message.push_str(&html_body);
    message.push_str("\r\n");

    // End boundary
    message.push_str(&format!("--{}--\r\n", boundary));

    message
}

pub fn encode_for_gmail(raw: &str) -> String {
    URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

fn encode_header_value(value: &str) -> String {
    // If ASCII only, return as-is
    if value.is_ascii() {
        return value.to_string();
    }

    // Use RFC 2047 encoded-word for non-ASCII
    let encoded = URL_SAFE_NO_PAD.encode(value.as_bytes());
    format!("=?UTF-8?B?{}?=", encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_required_headers() {
        let msg = EmailMessage {
            to: "recipient@example.com".to_string(),
            subject: "Test Subject".to_string(),
            body_markdown: "Hello **world**".to_string(),
        };

        let mime = build_mime_message(&msg, "sender@example.com");

        assert!(mime.contains("From: sender@example.com"), "Missing From header");
        assert!(mime.contains("To: recipient@example.com"), "Missing To header");
        assert!(mime.contains("Subject: Test Subject"), "Missing Subject header");
    }

    #[test]
    fn test_has_both_parts() {
        let msg = EmailMessage {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body_markdown: "Hello **world**".to_string(),
        };

        let mime = build_mime_message(&msg, "sender@example.com");

        assert!(
            mime.contains("Content-Type: text/plain"),
            "Missing text/plain part"
        );
        assert!(
            mime.contains("Content-Type: text/html"),
            "Missing text/html part"
        );
        assert!(
            mime.contains("Hello **world**"),
            "Missing plain text content"
        );
        assert!(
            mime.contains("<strong>world</strong>"),
            "Missing HTML content"
        );
    }

    #[test]
    fn test_base64url_encoding_no_special_chars() {
        let long_string = "Long string ".repeat(100);
        let test_strings = vec![
            "Hello, World!",
            "Test with special chars: +/=",
            "Unicode: \u{1F600}",
            long_string.as_str(),
        ];

        for s in test_strings {
            let encoded = encode_for_gmail(s);
            assert!(
                !encoded.contains('+'),
                "Encoded string should not contain '+': {}",
                encoded
            );
            assert!(
                !encoded.contains('/'),
                "Encoded string should not contain '/': {}",
                encoded
            );
            assert!(
                !encoded.contains('='),
                "Encoded string should not contain '=': {}",
                encoded
            );
        }
    }

    #[test]
    fn test_multipart_boundary() {
        let msg = EmailMessage {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body_markdown: "Body".to_string(),
        };

        let mime = build_mime_message(&msg, "sender@example.com");

        assert!(
            mime.contains("multipart/alternative"),
            "Missing multipart/alternative"
        );
        assert!(mime.contains("boundary="), "Missing boundary");
    }

    #[test]
    fn test_empty_body() {
        let msg = EmailMessage {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            body_markdown: "".to_string(),
        };

        let mime = build_mime_message(&msg, "sender@example.com");

        // Should still have both parts
        assert!(mime.contains("text/plain"));
        assert!(mime.contains("text/html"));
    }
}
