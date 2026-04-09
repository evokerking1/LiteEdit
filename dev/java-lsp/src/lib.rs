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
    ("abstract", 13, "Java keyword", ""),
    ("assert", 13, "Java keyword", ""),
    ("boolean", 13, "Java keyword", ""),
    ("break", 13, "Java keyword", "Exits the current loop or switch."),
    ("byte", 13, "Java keyword", ""),
    ("case", 13, "Java keyword", ""),
    ("catch", 13, "Java keyword", ""),
    ("char", 13, "Java keyword", ""),
    ("class", 13, "Java keyword", "Declares a new class."),
    ("const", 13, "Java keyword", ""),
    ("continue", 13, "Java keyword", ""),
    ("default", 13, "Java keyword", ""),
    ("do", 13, "Java keyword", ""),
    ("double", 13, "Java keyword", ""),
    ("else", 13, "Java keyword", ""),
    ("enum", 13, "Java keyword", ""),
    ("extends", 13, "Java keyword", "Indicates a class inherits from another."),
    ("final", 13, "Java keyword", "Prevents modification of variables, methods, or classes."),
    ("finally", 13, "Java keyword", ""),
    ("float", 13, "Java keyword", ""),
    ("for", 13, "Java keyword", "Looping construct."),
    ("goto", 13, "Java keyword", ""),
    ("if", 13, "Java keyword", "Conditional branching."),
    ("implements", 13, "Java keyword", ""),
    ("import", 13, "Java keyword", "Imports a package or class."),
    ("instanceof", 13, "Java keyword", ""),
    ("int", 13, "Java keyword", ""),
    ("interface", 13, "Java keyword", "Declares an interface."),
    ("long", 13, "Java keyword", ""),
    ("native", 13, "Java keyword", ""),
    ("new", 13, "Java keyword", "Creates a new object instance."),
    ("null", 13, "Java keyword", "Null literal."),
    ("package", 13, "Java keyword", "Declares the current package."),
    ("private", 13, "Java keyword", "Access modifier: class-only visibility."),
    ("protected", 13, "Java keyword", "Access modifier: package + subclass visibility."),
    ("public", 13, "Java keyword", "Access modifier: full visibility."),
    ("return", 13, "Java keyword", "Returns a value from a method."),
    ("short", 13, "Java keyword", ""),
    ("static", 13, "Java keyword", "Class-level member."),
    ("strictfp", 13, "Java keyword", ""),
    ("super", 13, "Java keyword", ""),
    ("switch", 13, "Java keyword", ""),
    ("synchronized", 13, "Java keyword", ""),
    ("this", 13, "Java keyword", "Refers to the current instance."),
    ("throw", 13, "Java keyword", "Throws an exception."),
    ("throws", 13, "Java keyword", "Declares exceptions a method may throw."),
    ("transient", 13, "Java keyword", ""),
    ("try", 13, "Java keyword", "Begins a try-catch block."),
    ("var", 13, "Java keyword", ""),
    ("void", 13, "Java keyword", "Indicates no return value."),
    ("volatile", 13, "Java keyword", ""),
    ("while", 13, "Java keyword", "Looping construct."),
    ("String", 6, "java.lang / java.util", "Represents a sequence of characters."),
    ("Integer", 6, "java.lang / java.util", "Wrapper class for int."),
    ("Long", 6, "java.lang / java.util", ""),
    ("Double", 6, "java.lang / java.util", ""),
    ("Float", 6, "java.lang / java.util", ""),
    ("Boolean", 6, "java.lang / java.util", ""),
    ("Character", 6, "java.lang / java.util", ""),
    ("Byte", 6, "java.lang / java.util", ""),
    ("Short", 6, "java.lang / java.util", ""),
    ("Object", 6, "java.lang / java.util", ""),
    ("Class", 6, "java.lang / java.util", ""),
    ("System", 6, "java.lang / java.util", "Provides access to system resources."),
    ("Math", 6, "java.lang / java.util", "Provides basic numeric operations."),
    ("Thread", 6, "java.lang / java.util", "A thread of execution."),
    ("StringBuilder", 6, "java.lang / java.util", ""),
    ("StringBuffer", 6, "java.lang / java.util", ""),
    ("ArrayList", 6, "java.lang / java.util", "Resizable-array implementation of List."),
    ("LinkedList", 6, "java.lang / java.util", ""),
    ("HashMap", 6, "java.lang / java.util", "Hash table based implementation of Map."),
    ("HashSet", 6, "java.lang / java.util", ""),
    ("TreeMap", 6, "java.lang / java.util", ""),
    ("TreeSet", 6, "java.lang / java.util", ""),
    ("Arrays", 6, "java.lang / java.util", ""),
    ("Collections", 6, "java.lang / java.util", ""),
    ("Optional", 6, "java.lang / java.util", "Container object that may or may not contain a non-null value."),
    ("Stream", 6, "java.lang / java.util", "A sequence of elements supporting sequential and parallel aggregate operations."),
    ("List", 6, "java.lang / java.util", ""),
    ("Map", 6, "java.lang / java.util", ""),
    ("Set", 6, "java.lang / java.util", ""),
    ("Queue", 6, "java.lang / java.util", ""),
    ("Deque", 6, "java.lang / java.util", ""),
    ("Iterator", 6, "java.lang / java.util", ""),
    ("Iterable", 6, "java.lang / java.util", ""),
    ("Comparable", 6, "java.lang / java.util", ""),
    ("Cloneable", 6, "java.lang / java.util", ""),
    ("Serializable", 6, "java.lang / java.util", ""),
    ("Runnable", 6, "java.lang / java.util", ""),
    ("Callable", 6, "java.lang / java.util", ""),
    ("Exception", 6, "java.lang / java.util", "Base class for checked exceptions."),
    ("RuntimeException", 6, "java.lang / java.util", "Base class for unchecked exceptions."),
    ("IOException", 6, "java.lang / java.util", ""),
    ("NullPointerException", 6, "java.lang / java.util", ""),
    ("IllegalArgumentException", 6, "java.lang / java.util", ""),
    ("IndexOutOfBoundsException", 6, "java.lang / java.util", ""),
    ("NumberFormatException", 6, "java.lang / java.util", ""),
    ("System.out.println", 1, "method", "Prints to standard output with newline."),
    ("System.out.print", 1, "method", ""),
    ("System.err.println", 1, "method", ""),
    ("toString", 1, "method", "Returns string representation."),
    ("equals", 1, "method", "Compares two objects for equality."),
    ("hashCode", 1, "method", "Returns the hash code of the object."),
    ("compareTo", 1, "method", ""),
    ("length", 1, "method", "Returns the length of the string."),
    ("charAt", 1, "method", ""),
    ("substring", 1, "method", ""),
    ("indexOf", 1, "method", ""),
    ("contains", 1, "method", ""),
    ("startsWith", 1, "method", ""),
    ("endsWith", 1, "method", ""),
    ("replace", 1, "method", ""),
    ("split", 1, "method", ""),
    ("trim", 1, "method", ""),
    ("isEmpty", 1, "method", ""),
    ("toUpperCase", 1, "method", ""),
    ("toLowerCase", 1, "method", ""),
    ("get", 1, "method", ""),
    ("put", 1, "method", ""),
    ("remove", 1, "method", ""),
    ("add", 1, "method", ""),
    ("size", 1, "method", ""),
    ("isEmpty", 1, "method", ""),
    ("iterator", 1, "method", ""),
    ("stream", 1, "method", "Returns a sequential Stream."),
    ("forEach", 1, "method", "Iterates over each element."),
    ("map", 1, "method", ""),
    ("filter", 1, "method", ""),
    ("collect", 1, "method", ""),
    ("reduce", 1, "method", ""),
    ("sort", 1, "method", "Sorts the collection."),
    ("Arrays.sort", 1, "method", ""),
    ("Collections.sort", 1, "method", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "java".to_string()
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

    #[test]
    fn test_language_id() {
        assert_eq!(language_id(), "java");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("cl", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
        for item in &items {
            let label = item["label"].as_str().unwrap().to_lowercase();
            assert!(label.starts_with("cl"), "label '{}' does not start with 'cl'", label);
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
        let result = get_hover("class Foo {}", 0, 2);
        assert_ne!(result, "null");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["contents"].as_str().unwrap().contains("Declares a new class."));
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
