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
    ("break", 13, "JS keyword", ""),
    ("case", 13, "JS keyword", ""),
    ("catch", 13, "JS keyword", "Catches exceptions."),
    ("class", 13, "JS keyword", "Declares a class."),
    ("const", 13, "JS keyword", "Declares a block-scoped constant."),
    ("continue", 13, "JS keyword", ""),
    ("debugger", 13, "JS keyword", ""),
    ("default", 13, "JS keyword", ""),
    ("delete", 13, "JS keyword", ""),
    ("do", 13, "JS keyword", ""),
    ("else", 13, "JS keyword", "Else branch."),
    ("export", 13, "JS keyword", "Exports module members."),
    ("extends", 13, "JS keyword", ""),
    ("false", 13, "JS keyword", "Boolean false."),
    ("finally", 13, "JS keyword", ""),
    ("for", 13, "JS keyword", "Loop."),
    ("function", 13, "JS keyword", "Declares a function."),
    ("if", 13, "JS keyword", "Conditional statement."),
    ("import", 13, "JS keyword", "Imports module exports."),
    ("in", 13, "JS keyword", ""),
    ("instanceof", 13, "JS keyword", "Tests prototype chain."),
    ("let", 13, "JS keyword", "Declares a block-scoped variable."),
    ("new", 13, "JS keyword", "Creates an instance."),
    ("null", 13, "JS keyword", "Null literal."),
    ("of", 13, "JS keyword", ""),
    ("return", 13, "JS keyword", "Returns a value."),
    ("static", 13, "JS keyword", ""),
    ("super", 13, "JS keyword", "Calls parent class."),
    ("switch", 13, "JS keyword", ""),
    ("this", 13, "JS keyword", "Refers to the current context."),
    ("throw", 13, "JS keyword", "Throws an exception."),
    ("true", 13, "JS keyword", "Boolean true."),
    ("try", 13, "JS keyword", "Exception handling block."),
    ("typeof", 13, "JS keyword", "Returns the type as a string."),
    ("undefined", 13, "JS keyword", "Undefined value."),
    ("var", 13, "JS keyword", "Declares a function-scoped variable."),
    ("void", 13, "JS keyword", ""),
    ("while", 13, "JS keyword", "Loop while condition is true."),
    ("with", 13, "JS keyword", ""),
    ("yield", 13, "JS keyword", ""),
    ("async", 13, "JS keyword", "Marks an async function."),
    ("await", 13, "JS keyword", "Awaits a Promise."),
    ("console", 5, "built-in", "Provides debugging console."),
    ("window", 5, "built-in", "Browser global object."),
    ("document", 5, "built-in", "DOM document object."),
    ("process", 5, "built-in", ""),
    ("require", 5, "built-in", ""),
    ("module", 5, "built-in", ""),
    ("exports", 5, "built-in", ""),
    ("Promise", 5, "built-in", "Represents eventual async value."),
    ("Array", 5, "built-in", "Array constructor."),
    ("Object", 5, "built-in", "Base object type."),
    ("String", 5, "built-in", ""),
    ("Number", 5, "built-in", ""),
    ("Boolean", 5, "built-in", ""),
    ("Date", 5, "built-in", ""),
    ("Math", 5, "built-in", "Mathematical constants and functions."),
    ("JSON", 5, "built-in", "JSON parse/stringify."),
    ("RegExp", 5, "built-in", ""),
    ("Error", 5, "built-in", ""),
    ("Map", 5, "built-in", ""),
    ("Set", 5, "built-in", ""),
    ("WeakMap", 5, "built-in", ""),
    ("WeakSet", 5, "built-in", ""),
    ("Symbol", 5, "built-in", ""),
    ("Proxy", 5, "built-in", ""),
    ("Reflect", 5, "built-in", ""),
    ("parseInt", 5, "built-in", ""),
    ("parseFloat", 5, "built-in", ""),
    ("isNaN", 5, "built-in", ""),
    ("isFinite", 5, "built-in", ""),
    ("encodeURIComponent", 5, "built-in", ""),
    ("decodeURIComponent", 5, "built-in", ""),
    ("setTimeout", 5, "built-in", "Executes after a delay."),
    ("setInterval", 5, "built-in", "Executes at intervals."),
    ("clearTimeout", 5, "built-in", ""),
    ("clearInterval", 5, "built-in", ""),
    ("fetch", 5, "built-in", "Makes HTTP requests."),
    ("localStorage", 5, "built-in", "Browser local storage."),
    ("sessionStorage", 5, "built-in", ""),
    ("console.log", 1, "method", "Logs to console."),
    ("console.error", 1, "method", ""),
    ("console.warn", 1, "method", ""),
    ("console.info", 1, "method", ""),
    ("Array.from", 1, "method", "Creates array from iterable."),
    ("Array.isArray", 1, "method", ""),
    ("Object.keys", 1, "method", "Returns array of object keys."),
    ("Object.values", 1, "method", "Returns array of object values."),
    ("Object.entries", 1, "method", ""),
    ("Object.assign", 1, "method", "Copies properties to target."),
    ("Object.freeze", 1, "method", ""),
    ("JSON.stringify", 1, "method", "Converts to JSON string."),
    ("JSON.parse", 1, "method", "Parses JSON string."),
    ("Promise.all", 1, "method", "Awaits all promises."),
    ("Promise.resolve", 1, "method", ""),
    ("Promise.reject", 1, "method", ""),
    ("Math.floor", 1, "method", "Rounds down."),
    ("Math.ceil", 1, "method", "Rounds up."),
    ("Math.round", 1, "method", "Rounds to nearest integer."),
    ("Math.random", 1, "method", "Returns random 0-1 float."),
    ("Math.abs", 1, "method", ""),
    ("Math.max", 1, "method", ""),
    ("Math.min", 1, "method", ""),
    ("String.fromCharCode", 1, "method", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "javascript".to_string()
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
    let diags = count_braces(code);
    serde_json::to_string(&diags).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use serde_json;

    #[test]
    fn test_language_id() {
        assert_eq!(language_id(), "javascript");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("co", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
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
        let result = get_hover("const x = 1", 0, 1);
        assert_ne!(result, "null");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["contents"].as_str().unwrap().contains("Declares a block-scoped constant."));
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
        let result = get_diagnostics("{ {");
        let diags: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_diagnostics_unexpected_close() {
        let result = get_diagnostics("}");
        let diags: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(diags.len(), 1);
        assert!(diags[0]["message"].as_str().unwrap().contains("Unexpected"));
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
