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
    ("break", 13, "Go keyword", "Exits a loop or switch."),
    ("case", 13, "Go keyword", "Branch of switch/select."),
    ("chan", 13, "Go keyword", "Channel type for goroutine communication."),
    ("const", 13, "Go keyword", "Declares a constant."),
    ("continue", 13, "Go keyword", "Next iteration."),
    ("default", 13, "Go keyword", "Default case."),
    ("defer", 13, "Go keyword", "Defers execution until surrounding function returns."),
    ("else", 13, "Go keyword", "Else branch."),
    ("fallthrough", 13, "Go keyword", "Falls through to next case."),
    ("for", 13, "Go keyword", "Loop (also while-style)."),
    ("func", 13, "Go keyword", "Defines a function."),
    ("go", 13, "Go keyword", "Starts a goroutine."),
    ("goto", 13, "Go keyword", "Unconditional jump."),
    ("if", 13, "Go keyword", "Conditional statement."),
    ("import", 13, "Go keyword", "Imports packages."),
    ("interface", 13, "Go keyword", "Defines an interface."),
    ("map", 13, "Go keyword", "Hash map type."),
    ("package", 13, "Go keyword", "Declares the current package."),
    ("range", 13, "Go keyword", "Iterates over a slice, map, channel, or string."),
    ("return", 13, "Go keyword", "Returns from a function."),
    ("select", 13, "Go keyword", "Selects from channel operations."),
    ("struct", 13, "Go keyword", "Defines a composite type."),
    ("switch", 13, "Go keyword", "Multi-way branch."),
    ("type", 13, "Go keyword", "Declares a type alias or definition."),
    ("var", 13, "Go keyword", "Declares a variable."),
    ("append", 5, "built-in", "Appends elements to a slice."),
    ("cap", 5, "built-in", "Returns the capacity of a slice/channel."),
    ("close", 5, "built-in", "Closes a channel."),
    ("complex", 5, "built-in", ""),
    ("copy", 5, "built-in", "Copies slice elements."),
    ("delete", 5, "built-in", "Deletes a key from a map."),
    ("imag", 5, "built-in", ""),
    ("len", 5, "built-in", "Returns length of string/slice/map/channel."),
    ("make", 5, "built-in", "Allocates and initializes a slice/map/channel."),
    ("new", 5, "built-in", "Allocates a zeroed value and returns pointer."),
    ("panic", 5, "built-in", "Stops execution and unwinds stack."),
    ("print", 5, "built-in", "Prints to stderr (builtin)."),
    ("println", 5, "built-in", "Prints with newline to stderr (builtin)."),
    ("real", 5, "built-in", ""),
    ("recover", 5, "built-in", "Regains control after a panic."),
    ("fmt", 8, "std package", "Formatted I/O."),
    ("os", 8, "std package", "OS functions."),
    ("io", 8, "std package", "Basic I/O interfaces."),
    ("bufio", 8, "std package", ""),
    ("strings", 8, "std package", "String manipulation."),
    ("strconv", 8, "std package", "String conversions."),
    ("bytes", 8, "std package", ""),
    ("errors", 8, "std package", "Error creation."),
    ("log", 8, "std package", "Logging."),
    ("math", 8, "std package", "Math functions."),
    ("sort", 8, "std package", "Sorting."),
    ("sync", 8, "std package", "Synchronization primitives."),
    ("time", 8, "std package", "Time functions."),
    ("net", 8, "std package", ""),
    ("net/http", 8, "std package", "HTTP client and server."),
    ("encoding/json", 8, "std package", "JSON encoding/decoding."),
    ("encoding/xml", 8, "std package", ""),
    ("path", 8, "std package", ""),
    ("path/filepath", 8, "std package", ""),
    ("regexp", 8, "std package", "Regular expressions."),
    ("reflect", 8, "std package", ""),
    ("context", 8, "std package", "Deadlines and cancellations."),
    ("atomic", 8, "std package", ""),
    ("unicode", 8, "std package", ""),
    ("runtime", 8, "std package", ""),
    ("testing", 8, "std package", "Testing framework."),
    ("error", 6, "type", "Built-in error interface."),
    ("string", 6, "type", "UTF-8 string."),
    ("int", 6, "type", "Platform-sized integer."),
    ("int8", 6, "type", ""),
    ("int16", 6, "type", ""),
    ("int32", 6, "type", ""),
    ("int64", 6, "type", "64-bit integer."),
    ("uint", 6, "type", ""),
    ("uint8", 6, "type", ""),
    ("uint16", 6, "type", ""),
    ("uint32", 6, "type", ""),
    ("uint64", 6, "type", ""),
    ("uintptr", 6, "type", ""),
    ("float32", 6, "type", ""),
    ("float64", 6, "type", "64-bit float."),
    ("complex64", 6, "type", ""),
    ("complex128", 6, "type", ""),
    ("bool", 6, "type", "Boolean."),
    ("byte", 6, "type", "Alias for uint8."),
    ("rune", 6, "type", "Alias for int32, Unicode code point."),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "go".to_string()
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
        assert_eq!(language_id(), "go");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("fu", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
        for item in &items {
            let label = item["label"].as_str().unwrap().to_lowercase();
            assert!(label.starts_with("fu"), "label '{}' does not start with 'fu'", label);
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
        let result = get_hover("func myFunc() {}", 0, 1);
        assert_ne!(result, "null");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["contents"].as_str().unwrap().contains("Defines a function."));
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
        assert!(diags.len() > 0);
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
