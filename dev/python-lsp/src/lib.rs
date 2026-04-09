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
    ("False", 13, "Python keyword", "Boolean false literal."),
    ("None", 13, "Python keyword", "Null/None literal."),
    ("True", 13, "Python keyword", "Boolean true literal."),
    ("and", 13, "Python keyword", "Logical AND."),
    ("as", 13, "Python keyword", ""),
    ("assert", 13, "Python keyword", "Asserts a condition is true."),
    ("async", 13, "Python keyword", "Marks a coroutine function."),
    ("await", 13, "Python keyword", "Awaits an async expression."),
    ("break", 13, "Python keyword", ""),
    ("class", 13, "Python keyword", "Defines a class."),
    ("continue", 13, "Python keyword", ""),
    ("def", 13, "Python keyword", "Defines a function."),
    ("del", 13, "Python keyword", ""),
    ("elif", 13, "Python keyword", "Else-if branch."),
    ("else", 13, "Python keyword", "Else branch."),
    ("except", 13, "Python keyword", "Catches exceptions."),
    ("finally", 13, "Python keyword", "Executes regardless of exceptions."),
    ("for", 13, "Python keyword", "Iterates over a sequence."),
    ("from", 13, "Python keyword", "Imports specific attributes from a module."),
    ("global", 13, "Python keyword", ""),
    ("if", 13, "Python keyword", "Conditional statement."),
    ("import", 13, "Python keyword", "Imports a module."),
    ("in", 13, "Python keyword", "Membership or iteration operator."),
    ("is", 13, "Python keyword", "Identity comparison."),
    ("lambda", 13, "Python keyword", "Anonymous inline function."),
    ("nonlocal", 13, "Python keyword", ""),
    ("not", 13, "Python keyword", "Logical negation."),
    ("or", 13, "Python keyword", "Logical OR."),
    ("pass", 13, "Python keyword", "No-op placeholder statement."),
    ("raise", 13, "Python keyword", "Raises an exception."),
    ("return", 13, "Python keyword", "Returns a value from a function."),
    ("try", 13, "Python keyword", "Begins an exception handling block."),
    ("while", 13, "Python keyword", "Loops while condition is true."),
    ("with", 13, "Python keyword", "Context manager block."),
    ("yield", 13, "Python keyword", "Yields a value from a generator."),
    ("print", 5, "built-in", "Prints to stdout."),
    ("len", 5, "built-in", "Returns the length of an object."),
    ("range", 5, "built-in", "Returns an immutable sequence of numbers."),
    ("type", 5, "built-in", "Returns the type of an object."),
    ("isinstance", 5, "built-in", "Checks if an object is an instance of a class."),
    ("issubclass", 5, "built-in", ""),
    ("hasattr", 5, "built-in", ""),
    ("getattr", 5, "built-in", ""),
    ("setattr", 5, "built-in", ""),
    ("delattr", 5, "built-in", ""),
    ("int", 5, "built-in", "Creates an integer."),
    ("float", 5, "built-in", "Creates a float."),
    ("str", 5, "built-in", "Creates a string."),
    ("bool", 5, "built-in", ""),
    ("list", 5, "built-in", "Creates a list."),
    ("tuple", 5, "built-in", "Creates a tuple."),
    ("dict", 5, "built-in", "Creates a dictionary."),
    ("set", 5, "built-in", "Creates a set."),
    ("frozenset", 5, "built-in", ""),
    ("bytes", 5, "built-in", ""),
    ("bytearray", 5, "built-in", ""),
    ("enumerate", 5, "built-in", "Returns an enumerate object."),
    ("zip", 5, "built-in", "Aggregates elements from iterables."),
    ("map", 5, "built-in", "Applies function to each element."),
    ("filter", 5, "built-in", "Filters elements by a function."),
    ("sorted", 5, "built-in", "Returns a sorted list."),
    ("reversed", 5, "built-in", ""),
    ("sum", 5, "built-in", "Sums elements."),
    ("min", 5, "built-in", "Returns the minimum."),
    ("max", 5, "built-in", "Returns the maximum."),
    ("abs", 5, "built-in", ""),
    ("round", 5, "built-in", ""),
    ("pow", 5, "built-in", ""),
    ("divmod", 5, "built-in", ""),
    ("hash", 5, "built-in", ""),
    ("id", 5, "built-in", ""),
    ("repr", 5, "built-in", ""),
    ("format", 5, "built-in", ""),
    ("input", 5, "built-in", ""),
    ("open", 5, "built-in", "Opens a file."),
    ("iter", 5, "built-in", ""),
    ("next", 5, "built-in", ""),
    ("callable", 5, "built-in", ""),
    ("staticmethod", 5, "built-in", "Defines a static method."),
    ("classmethod", 5, "built-in", "Defines a class method."),
    ("property", 5, "built-in", "Defines a managed attribute."),
    ("super", 5, "built-in", "Calls the parent class."),
    ("object", 5, "built-in", ""),
    ("os", 8, "module", "OS interface."),
    ("sys", 8, "module", "System-specific parameters."),
    ("re", 8, "module", "Regular expressions."),
    ("json", 8, "module", "JSON encoding/decoding."),
    ("math", 8, "module", "Math functions."),
    ("random", 8, "module", "Random number generation."),
    ("datetime", 8, "module", "Date and time operations."),
    ("pathlib", 8, "module", ""),
    ("collections", 8, "module", "Specialized container datatypes."),
    ("itertools", 8, "module", "Iterator building blocks."),
    ("functools", 8, "module", "Higher-order functions."),
    ("typing", 8, "module", "Type hints support."),
    ("io", 8, "module", ""),
    ("time", 8, "module", ""),
    ("threading", 8, "module", "Thread-based parallelism."),
    ("subprocess", 8, "module", "Subprocess management."),
    ("logging", 8, "module", "Logging facility."),
    ("unittest", 8, "module", "Unit testing framework."),
    ("pytest", 8, "module", ""),
    ("numpy", 8, "module", "Numerical Python."),
    ("pandas", 8, "module", "Data analysis library."),
    ("requests", 8, "module", "HTTP library."),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "python".to_string()
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
        assert_eq!(language_id(), "python");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("de", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
        for item in &items {
            let label = item["label"].as_str().unwrap().to_lowercase();
            assert!(label.starts_with("de"), "label '{}' does not start with 'de'", label);
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
        let result = get_hover("def my_func():", 0, 1);
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
