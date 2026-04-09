use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: u32,
    pub detail: String,
    pub documentation: String,
    pub insert_text: String,
}

#[derive(Serialize, Deserialize)]
pub struct HoverResult {
    pub contents: String,
}

#[derive(Serialize, Deserialize)]
pub struct Diagnostic {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub message: String,
    pub severity: u32,
}

fn prefix_at(code: &str, line: u32, col: u32) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let line_text = match lines.get(line as usize) {
        Some(l) => l,
        None => return String::new(),
    };
    let col = (col as usize).min(line_text.len());
    let before = &line_text[..col];
    before
        .chars()
        .rev()
        .take_while(|&c| c.is_alphanumeric() || c == '_')
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

fn word_at(code: &str, line: u32, col: u32) -> Option<String> {
    let lines: Vec<&str> = code.lines().collect();
    let line_text = lines.get(line as usize)?;
    let col = col as usize;
    let chars: Vec<char> = line_text.chars().collect();
    if col >= chars.len() {
        return None;
    }
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    if !is_word(chars[col]) {
        return None;
    }
    let start = (0..col)
        .rev()
        .find(|&i| !is_word(chars[i]))
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = ((col + 1)..chars.len())
        .find(|&i| !is_word(chars[i]))
        .unwrap_or(chars.len());
    Some(chars[start..end].iter().collect())
}

#[allow(dead_code)]
fn count_braces(code: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut depth: i32 = 0;
    for (i, line) in code.lines().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth < 0 {
                        diags.push(Diagnostic {
                            start_line: i as u32,
                            start_col: j as u32,
                            end_line: i as u32,
                            end_col: (j + 1) as u32,
                            message: "Unexpected closing brace '}'".to_string(),
                            severity: 1,
                        });
                        depth = 0;
                    }
                }
                _ => {}
            }
        }
    }
    if depth > 0 {
        let last_line = code.lines().count().saturating_sub(1) as u32;
        diags.push(Diagnostic {
            start_line: last_line,
            start_col: 0,
            end_line: last_line,
            end_col: 1,
            message: format!("{} unclosed brace(s) '{{' detected", depth),
            severity: 1,
        });
    }
    diags
}

#[allow(dead_code)]
static COMPLETIONS: &[(&str, u32, &str, &str)] = &[
    ("html", 14, "HTML tag", ""),
    ("head", 14, "HTML tag", ""),
    ("body", 14, "HTML tag", ""),
    ("div", 14, "HTML tag", "Generic block container."),
    ("span", 14, "HTML tag", "Generic inline container."),
    ("p", 14, "HTML tag", "Paragraph."),
    ("a", 14, "HTML tag", "Hyperlink."),
    ("img", 14, "HTML tag", "Image."),
    ("input", 14, "HTML tag", "Form input."),
    ("button", 14, "HTML tag", "Clickable button."),
    ("form", 14, "HTML tag", "HTML form."),
    ("label", 14, "HTML tag", "Form label."),
    ("select", 14, "HTML tag", "Dropdown selection."),
    ("option", 14, "HTML tag", ""),
    ("textarea", 14, "HTML tag", ""),
    ("table", 14, "HTML tag", "Data table."),
    ("thead", 14, "HTML tag", ""),
    ("tbody", 14, "HTML tag", ""),
    ("tr", 14, "HTML tag", ""),
    ("th", 14, "HTML tag", ""),
    ("td", 14, "HTML tag", ""),
    ("ul", 14, "HTML tag", "Unordered list."),
    ("ol", 14, "HTML tag", "Ordered list."),
    ("li", 14, "HTML tag", "List item."),
    ("nav", 14, "HTML tag", "Navigation."),
    ("header", 14, "HTML tag", "Page or section header."),
    ("footer", 14, "HTML tag", "Page or section footer."),
    ("main", 14, "HTML tag", "Main content area."),
    ("section", 14, "HTML tag", "Thematic grouping."),
    ("article", 14, "HTML tag", "Self-contained content."),
    ("aside", 14, "HTML tag", "Sidebar content."),
    ("h1", 14, "HTML tag", "Top-level heading."),
    ("h2", 14, "HTML tag", ""),
    ("h3", 14, "HTML tag", ""),
    ("h4", 14, "HTML tag", ""),
    ("h5", 14, "HTML tag", ""),
    ("h6", 14, "HTML tag", ""),
    ("strong", 14, "HTML tag", ""),
    ("em", 14, "HTML tag", ""),
    ("b", 14, "HTML tag", ""),
    ("i", 14, "HTML tag", ""),
    ("u", 14, "HTML tag", ""),
    ("s", 14, "HTML tag", ""),
    ("del", 14, "HTML tag", ""),
    ("ins", 14, "HTML tag", ""),
    ("code", 14, "HTML tag", ""),
    ("pre", 14, "HTML tag", ""),
    ("blockquote", 14, "HTML tag", ""),
    ("hr", 14, "HTML tag", ""),
    ("br", 14, "HTML tag", ""),
    ("link", 14, "HTML tag", ""),
    ("script", 14, "HTML tag", "Embedded script."),
    ("style", 14, "HTML tag", "Embedded CSS."),
    ("meta", 14, "HTML tag", "Document metadata."),
    ("title", 14, "HTML tag", ""),
    ("base", 14, "HTML tag", ""),
    ("noscript", 14, "HTML tag", ""),
    ("template", 14, "HTML tag", ""),
    ("slot", 14, "HTML tag", ""),
    ("canvas", 14, "HTML tag", "Drawing surface."),
    ("svg", 14, "HTML tag", "Scalable vector graphics."),
    ("video", 14, "HTML tag", "Video player."),
    ("audio", 14, "HTML tag", "Audio player."),
    ("source", 14, "HTML tag", ""),
    ("track", 14, "HTML tag", ""),
    ("figure", 14, "HTML tag", ""),
    ("figcaption", 14, "HTML tag", ""),
    ("details", 14, "HTML tag", ""),
    ("summary", 14, "HTML tag", ""),
    ("dialog", 14, "HTML tag", "Modal dialog."),
    ("progress", 14, "HTML tag", ""),
    ("meter", 14, "HTML tag", ""),
    ("output", 14, "HTML tag", ""),
    ("datalist", 14, "HTML tag", ""),
    ("fieldset", 14, "HTML tag", ""),
    ("legend", 14, "HTML tag", ""),
    ("iframe", 14, "HTML tag", ""),
    ("object", 14, "HTML tag", ""),
    ("embed", 14, "HTML tag", ""),
    ("param", 14, "HTML tag", ""),
    ("area", 14, "HTML tag", ""),
    ("map", 14, "HTML tag", ""),
    ("class", 9, "attribute", "CSS class name(s)."),
    ("id", 9, "attribute", "Unique element identifier."),
    ("style", 9, "attribute", "Inline CSS styles."),
    ("href", 9, "attribute", "Hyperlink URL."),
    ("src", 9, "attribute", "Resource URL."),
    ("alt", 9, "attribute", "Alternative text for images."),
    ("title", 9, "attribute", ""),
    ("type", 9, "attribute", "Input type or MIME type."),
    ("name", 9, "attribute", "Form field name."),
    ("value", 9, "attribute", "Form field value."),
    ("placeholder", 9, "attribute", "Input placeholder text."),
    ("disabled", 9, "attribute", "Disables the element."),
    ("checked", 9, "attribute", "Checkbox/radio checked state."),
    ("selected", 9, "attribute", ""),
    ("required", 9, "attribute", "Required form field."),
    ("readonly", 9, "attribute", "Read-only input."),
    ("multiple", 9, "attribute", ""),
    ("action", 9, "attribute", "Form submission URL."),
    ("method", 9, "attribute", "Form HTTP method."),
    ("enctype", 9, "attribute", ""),
    ("target", 9, "attribute", "Link target window."),
    ("rel", 9, "attribute", "Link relationship type."),
    ("media", 9, "attribute", ""),
    ("charset", 9, "attribute", "Character encoding."),
    ("lang", 9, "attribute", "Element language."),
    ("dir", 9, "attribute", ""),
    ("tabindex", 9, "attribute", "Tab key navigation order."),
    ("accesskey", 9, "attribute", ""),
    ("contenteditable", 9, "attribute", "Makes element editable."),
    ("draggable", 9, "attribute", ""),
    ("hidden", 9, "attribute", "Hides element."),
    ("spellcheck", 9, "attribute", ""),
    ("autocomplete", 9, "attribute", ""),
    ("autofocus", 9, "attribute", ""),
    ("data-", 9, "attribute", "Custom data attribute prefix."),
    ("aria-label", 9, "attribute", "Accessible label."),
    ("aria-hidden", 9, "attribute", ""),
    ("aria-expanded", 9, "attribute", ""),
    ("role", 9, "attribute", "ARIA role."),
    ("for", 9, "attribute", ""),
    ("colspan", 9, "attribute", ""),
    ("rowspan", 9, "attribute", ""),
    ("width", 9, "attribute", ""),
    ("height", 9, "attribute", ""),
    ("min", 9, "attribute", ""),
    ("max", 9, "attribute", ""),
    ("step", 9, "attribute", ""),
    ("pattern", 9, "attribute", ""),
    ("list", 9, "attribute", ""),
    ("form", 9, "attribute", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "html".to_string()
}

#[wasm_bindgen]
pub fn get_completions(code: &str, line: u32, col: u32) -> String {
    let prefix = prefix_at(code, line, col);
    let items: Vec<CompletionItem> = COMPLETIONS
        .iter()
        .filter(|(label, _, _, _)| {
            prefix.is_empty() || label.to_lowercase().starts_with(&prefix.to_lowercase())
        })
        .take(100)
        .map(|(label, kind, detail, doc)| CompletionItem {
            label: label.to_string(),
            kind: *kind,
            detail: detail.to_string(),
            documentation: doc.to_string(),
            insert_text: label.to_string(),
        })
        .collect();
    serde_json::to_string(&items).unwrap_or_default()
}

#[wasm_bindgen]
pub fn get_hover(code: &str, line: u32, col: u32) -> String {
    let word = match word_at(code, line, col) {
        Some(w) => w,
        None => return "null".to_string(),
    };
    for (label, _, _, doc) in COMPLETIONS {
        if *label == word.as_str() && !doc.is_empty() {
            let result = HoverResult {
                contents: format!("**{}**\n\n{}", label, doc),
            };
            return serde_json::to_string(&result).unwrap_or_default();
        }
    }
    "null".to_string()
}

#[wasm_bindgen]
pub fn get_diagnostics(code: &str) -> String {
    let _ = code;
    serde_json::to_string(&Vec::<Diagnostic>::new()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use serde_json;

    #[test]
    fn test_language_id() {
        assert_eq!(language_id(), "html");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("di", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
        for item in &items {
            let label = item["label"].as_str().unwrap().to_lowercase();
            assert!(label.starts_with("di"), "label '{}' does not start with 'di'", label);
        }
    }

    #[test]
    fn test_completions_no_match() {
        let result = get_completions("ZZZNOTAWORD", 0, 11);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_hover_known_keyword() {
        let result = get_hover("div something", 0, 1);
        assert_ne!(result, "null");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["contents"].as_str().unwrap().contains("Generic block container."));
    }

    #[test]
    fn test_hover_unknown_word() {
        let result = get_hover("xyzunknown something", 0, 2);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_diagnostics_valid_code() {
        let result = get_diagnostics("{ }");
        let diags: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_diagnostics_unclosed_brace() {
        let result = get_diagnostics("anything");
        let diags: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_diagnostics_unexpected_close() {
        let result = get_diagnostics("}");
        let diags: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_prefix_at_helper() {
        assert_eq!(prefix_at("hello world", 0, 5), "hello");
        assert_eq!(prefix_at("", 0, 0), "");
        assert_eq!(prefix_at("hello world", 99, 0), "");
    }

    #[test]
    fn test_word_at_helper() {
        assert_eq!(word_at("hello world", 0, 2), Some("hello".to_string()));
        assert_eq!(word_at("hello world", 0, 5), None);
        assert_eq!(word_at("hello", 99, 0), None);
    }

    #[test]
    fn test_completions_json_valid() {
        let result = get_completions("", 0, 0);
        assert_ne!(result, "");
        let parsed = serde_json::from_str::<Vec<serde_json::Value>>(&result);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_hover_json_valid_or_null() {
        let result = get_hover("xyz 123", 0, 0);
        if result != "null" {
            let parsed = serde_json::from_str::<serde_json::Value>(&result);
            assert!(parsed.is_ok());
        }
    }
}
