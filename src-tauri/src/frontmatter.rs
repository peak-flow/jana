use regex::Regex;
use uuid::Uuid;

pub struct FrontmatterResult {
    pub jana_id: String,
    pub content: String,
    pub had_frontmatter: bool,
}

/// Parse raw file text. Extract jana_id from YAML frontmatter.
/// Returns the jana_id and the content with frontmatter stripped.
/// If no frontmatter or no jana_id, generates a new UUID.
pub fn parse_frontmatter(raw: &str) -> FrontmatterResult {
    if !raw.starts_with("---") {
        return FrontmatterResult {
            jana_id: Uuid::new_v4().to_string(),
            content: raw.to_string(),
            had_frontmatter: false,
        };
    }

    // Find closing ---
    let rest = &raw[3..];
    let closing = rest.find("\n---");
    if closing.is_none() {
        return FrontmatterResult {
            jana_id: Uuid::new_v4().to_string(),
            content: raw.to_string(),
            had_frontmatter: false,
        };
    }

    let closing_pos = closing.unwrap();
    let frontmatter_block = &rest[..closing_pos];
    // Content starts after the closing --- and its newline
    let after_closing = &rest[closing_pos + 4..]; // skip \n---
    let content = if after_closing.starts_with('\n') {
        after_closing[1..].to_string()
    } else {
        after_closing.to_string()
    };

    // Extract jana_id from frontmatter block
    let re = Regex::new(r"jana_id:\s*([0-9a-fA-F\-]{36})").unwrap();
    if let Some(caps) = re.captures(frontmatter_block) {
        FrontmatterResult {
            jana_id: caps[1].to_string(),
            content,
            had_frontmatter: true,
        }
    } else {
        FrontmatterResult {
            jana_id: Uuid::new_v4().to_string(),
            content,
            had_frontmatter: false,
        }
    }
}

/// Compose frontmatter + content for writing to disk.
pub fn compose_with_frontmatter(jana_id: &str, content: &str) -> String {
    format!("---\njana_id: {}\n---\n{}", jana_id, content)
}

/// Read a file, ensure it has a jana_id. If not, generate one and write it back.
/// Returns (jana_id, display_content).
pub fn ensure_jana_id(file_path: &str) -> Result<(String, String), String> {
    let raw = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let result = parse_frontmatter(&raw);

    if !result.had_frontmatter {
        let full = compose_with_frontmatter(&result.jana_id, &result.content);
        std::fs::write(file_path, &full)
            .map_err(|e| format!("Failed to write frontmatter: {}", e))?;
    }

    Ok((result.jana_id, result.content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_frontmatter() {
        let raw = "---\njana_id: 550e8400-e29b-41d4-a716-446655440000\n---\nHello world";
        let result = parse_frontmatter(raw);
        assert!(result.had_frontmatter);
        assert_eq!(result.jana_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(result.content, "Hello world");
    }

    #[test]
    fn test_parse_without_frontmatter() {
        let raw = "Just some text";
        let result = parse_frontmatter(raw);
        assert!(!result.had_frontmatter);
        assert_eq!(result.content, "Just some text");
        assert_eq!(result.jana_id.len(), 36); // valid UUID
    }

    #[test]
    fn test_compose() {
        let output = compose_with_frontmatter("abc-123", "Hello");
        assert_eq!(output, "---\njana_id: abc-123\n---\nHello");
    }

    #[test]
    fn test_roundtrip() {
        let content = "# My Note\n\nSome content here.";
        let jana_id = "550e8400-e29b-41d4-a716-446655440000";
        let composed = compose_with_frontmatter(jana_id, content);
        let parsed = parse_frontmatter(&composed);
        assert!(parsed.had_frontmatter);
        assert_eq!(parsed.jana_id, jana_id);
        assert_eq!(parsed.content, content);
    }
}
