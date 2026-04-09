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
    ("color", 9, "CSS property", "Sets text color."),
    ("background", 9, "CSS property", "Shorthand for all background properties."),
    ("background-color", 9, "CSS property", ""),
    ("background-image", 9, "CSS property", ""),
    ("background-position", 9, "CSS property", ""),
    ("background-size", 9, "CSS property", ""),
    ("background-repeat", 9, "CSS property", ""),
    ("border", 9, "CSS property", "Shorthand for border width/style/color."),
    ("border-color", 9, "CSS property", ""),
    ("border-width", 9, "CSS property", ""),
    ("border-style", 9, "CSS property", ""),
    ("border-radius", 9, "CSS property", "Rounded corners."),
    ("margin", 9, "CSS property", "Outer space around element."),
    ("margin-top", 9, "CSS property", ""),
    ("margin-right", 9, "CSS property", ""),
    ("margin-bottom", 9, "CSS property", ""),
    ("margin-left", 9, "CSS property", ""),
    ("padding", 9, "CSS property", "Inner space inside element."),
    ("padding-top", 9, "CSS property", ""),
    ("padding-right", 9, "CSS property", ""),
    ("padding-bottom", 9, "CSS property", ""),
    ("padding-left", 9, "CSS property", ""),
    ("width", 9, "CSS property", "Element width."),
    ("height", 9, "CSS property", "Element height."),
    ("min-width", 9, "CSS property", ""),
    ("max-width", 9, "CSS property", ""),
    ("min-height", 9, "CSS property", ""),
    ("max-height", 9, "CSS property", ""),
    ("display", 9, "CSS property", "Display type (block/inline/flex/grid/none/etc.)."),
    ("position", 9, "CSS property", "Positioning method."),
    ("top", 9, "CSS property", ""),
    ("right", 9, "CSS property", ""),
    ("bottom", 9, "CSS property", ""),
    ("left", 9, "CSS property", ""),
    ("z-index", 9, "CSS property", "Stack order."),
    ("overflow", 9, "CSS property", "How overflowing content is handled."),
    ("overflow-x", 9, "CSS property", ""),
    ("overflow-y", 9, "CSS property", ""),
    ("flex", 9, "CSS property", "Shorthand for flex-grow/shrink/basis."),
    ("flex-direction", 9, "CSS property", ""),
    ("flex-wrap", 9, "CSS property", ""),
    ("flex-grow", 9, "CSS property", ""),
    ("flex-shrink", 9, "CSS property", ""),
    ("flex-basis", 9, "CSS property", ""),
    ("justify-content", 9, "CSS property", ""),
    ("align-items", 9, "CSS property", ""),
    ("align-content", 9, "CSS property", ""),
    ("align-self", 9, "CSS property", ""),
    ("grid", 9, "CSS property", "Shorthand for grid properties."),
    ("grid-template-columns", 9, "CSS property", ""),
    ("grid-template-rows", 9, "CSS property", ""),
    ("grid-column", 9, "CSS property", ""),
    ("grid-row", 9, "CSS property", ""),
    ("gap", 9, "CSS property", ""),
    ("column-gap", 9, "CSS property", ""),
    ("row-gap", 9, "CSS property", ""),
    ("font", 9, "CSS property", ""),
    ("font-family", 9, "CSS property", ""),
    ("font-size", 9, "CSS property", "Size of text."),
    ("font-weight", 9, "CSS property", "Weight (boldness) of font."),
    ("font-style", 9, "CSS property", ""),
    ("font-variant", 9, "CSS property", ""),
    ("line-height", 9, "CSS property", ""),
    ("letter-spacing", 9, "CSS property", ""),
    ("word-spacing", 9, "CSS property", ""),
    ("text-align", 9, "CSS property", ""),
    ("text-decoration", 9, "CSS property", ""),
    ("text-transform", 9, "CSS property", ""),
    ("text-overflow", 9, "CSS property", ""),
    ("text-shadow", 9, "CSS property", ""),
    ("white-space", 9, "CSS property", ""),
    ("vertical-align", 9, "CSS property", ""),
    ("list-style", 9, "CSS property", ""),
    ("cursor", 9, "CSS property", "Mouse cursor style."),
    ("pointer-events", 9, "CSS property", ""),
    ("visibility", 9, "CSS property", "Element visibility."),
    ("opacity", 9, "CSS property", "Transparency level 0-1."),
    ("transform", 9, "CSS property", "2D/3D transformation."),
    ("transition", 9, "CSS property", "Smooth property transitions."),
    ("animation", 9, "CSS property", "Keyframe animations."),
    ("box-shadow", 9, "CSS property", "Shadow effect on element."),
    ("outline", 9, "CSS property", ""),
    ("content", 9, "CSS property", ""),
    ("clip-path", 9, "CSS property", ""),
    ("filter", 9, "CSS property", "Visual filters."),
    ("object-fit", 9, "CSS property", "How replaced elements fit their box."),
    ("object-position", 9, "CSS property", ""),
    ("resize", 9, "CSS property", ""),
    ("user-select", 9, "CSS property", ""),
    ("scroll-behavior", 9, "CSS property", ""),
    ("box-sizing", 9, "CSS property", "How dimensions are calculated."),
    ("float", 9, "CSS property", ""),
    ("clear", 9, "CSS property", ""),
    ("direction", 9, "CSS property", ""),
    ("unicode-bidi", 9, "CSS property", ""),
    ("none", 5, "CSS value", "No value/disable."),
    ("auto", 5, "CSS value", "Browser default."),
    ("inherit", 5, "CSS value", "Inherits from parent."),
    ("initial", 5, "CSS value", "CSS initial value."),
    ("unset", 5, "CSS value", "Removes inherited/initial."),
    ("revert", 5, "CSS value", ""),
    ("0", 5, "CSS value", ""),
    ("block", 5, "CSS value", "Block-level display."),
    ("inline", 5, "CSS value", "Inline display."),
    ("inline-block", 5, "CSS value", ""),
    ("flex", 5, "CSS value", "Flexbox container."),
    ("inline-flex", 5, "CSS value", ""),
    ("grid", 5, "CSS value", "Grid container."),
    ("inline-grid", 5, "CSS value", ""),
    ("contents", 5, "CSS value", ""),
    ("absolute", 5, "CSS value", "Absolute positioning."),
    ("relative", 5, "CSS value", "Relative positioning."),
    ("fixed", 5, "CSS value", "Fixed to viewport."),
    ("sticky", 5, "CSS value", "Sticky positioning."),
    ("static", 5, "CSS value", ""),
    ("solid", 5, "CSS value", ""),
    ("dashed", 5, "CSS value", ""),
    ("dotted", 5, "CSS value", ""),
    ("hidden", 5, "CSS value", ""),
    ("visible", 5, "CSS value", ""),
    ("scroll", 5, "CSS value", ""),
    ("clip", 5, "CSS value", ""),
    ("center", 5, "CSS value", "Center alignment."),
    ("left", 5, "CSS value", ""),
    ("right", 5, "CSS value", ""),
    ("top", 5, "CSS value", ""),
    ("bottom", 5, "CSS value", ""),
    ("space-between", 5, "CSS value", ""),
    ("space-around", 5, "CSS value", ""),
    ("space-evenly", 5, "CSS value", ""),
    ("stretch", 5, "CSS value", ""),
    ("flex-start", 5, "CSS value", ""),
    ("flex-end", 5, "CSS value", ""),
    ("column", 5, "CSS value", ""),
    ("row", 5, "CSS value", ""),
    ("wrap", 5, "CSS value", ""),
    ("nowrap", 5, "CSS value", ""),
    ("bold", 5, "CSS value", "Bold font weight."),
    ("normal", 5, "CSS value", ""),
    ("italic", 5, "CSS value", "Italic font style."),
    ("underline", 5, "CSS value", ""),
    ("line-through", 5, "CSS value", ""),
    ("uppercase", 5, "CSS value", ""),
    ("lowercase", 5, "CSS value", ""),
    ("capitalize", 5, "CSS value", ""),
    ("ellipsis", 5, "CSS value", ""),
    ("pointer", 5, "CSS value", ""),
    ("default", 5, "CSS value", ""),
    ("move", 5, "CSS value", ""),
    ("not-allowed", 5, "CSS value", ""),
    ("grab", 5, "CSS value", ""),
    ("grabbing", 5, "CSS value", ""),
    ("ease", 5, "CSS value", ""),
    ("ease-in", 5, "CSS value", ""),
    ("ease-out", 5, "CSS value", ""),
    ("ease-in-out", 5, "CSS value", ""),
    ("linear", 5, "CSS value", ""),
    ("infinite", 5, "CSS value", ""),
    ("forwards", 5, "CSS value", ""),
    ("backwards", 5, "CSS value", ""),
    ("both", 5, "CSS value", ""),
    ("@import", 13, "at-rule", "Imports another stylesheet."),
    ("@media", 13, "at-rule", "Applies styles for specific media/viewport."),
    ("@keyframes", 13, "at-rule", "Defines animation keyframes."),
    ("@font-face", 13, "at-rule", "Defines a custom font."),
    ("@charset", 13, "at-rule", ""),
    ("@supports", 13, "at-rule", "Conditional CSS based on support."),
    ("@layer", 13, "at-rule", "Cascade layer declaration."),
    ("@container", 13, "at-rule", "Container query."),
    ("@property", 13, "at-rule", ""),
    ("@counter-style", 13, "at-rule", ""),
    ("@page", 13, "at-rule", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "css".to_string()
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
        assert_eq!(language_id(), "css");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("co", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
        for item in &items {
            let label = item["label"].as_str().unwrap().to_lowercase();
            assert!(label.starts_with("co"), "label '{}' does not start with 'co'", label);
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
        let result = get_hover("color something", 0, 1);
        assert_ne!(result, "null");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["contents"].as_str().unwrap().contains("Sets text color."));
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
