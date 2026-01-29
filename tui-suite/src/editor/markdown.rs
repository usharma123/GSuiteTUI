use pulldown_cmark::{html, Parser};

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
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
