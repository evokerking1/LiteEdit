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
    ("as", 13, "Rust keyword", "Type casting."),
    ("async", 13, "Rust keyword", "Async function."),
    ("await", 13, "Rust keyword", "Awaits a future."),
    ("break", 13, "Rust keyword", "Exits a loop."),
    ("const", 13, "Rust keyword", "Defines a compile-time constant."),
    ("continue", 13, "Rust keyword", "Continues to next iteration."),
    ("crate", 13, "Rust keyword", "Refers to the current crate root."),
    ("dyn", 13, "Rust keyword", "Dynamic dispatch trait object."),
    ("else", 13, "Rust keyword", "Else branch."),
    ("enum", 13, "Rust keyword", "Defines an enum."),
    ("extern", 13, "Rust keyword", "External linkage."),
    ("false", 13, "Rust keyword", "Boolean false."),
    ("fn", 13, "Rust keyword", "Defines a function."),
    ("for", 13, "Rust keyword", "Iterates over a collection."),
    ("if", 13, "Rust keyword", "Conditional."),
    ("impl", 13, "Rust keyword", "Implements methods on a type."),
    ("in", 13, "Rust keyword", "Range/iterator operator."),
    ("let", 13, "Rust keyword", "Binds a variable."),
    ("loop", 13, "Rust keyword", "Infinite loop."),
    ("match", 13, "Rust keyword", "Pattern matching."),
    ("mod", 13, "Rust keyword", "Declares a module."),
    ("move", 13, "Rust keyword", "Moves captured variables into closure."),
    ("mut", 13, "Rust keyword", "Makes a binding mutable."),
    ("pub", 13, "Rust keyword", "Public visibility."),
    ("ref", 13, "Rust keyword", "Borrows by reference."),
    ("return", 13, "Rust keyword", "Returns from a function."),
    ("self", 13, "Rust keyword", "Refers to the current instance."),
    ("Self", 13, "Rust keyword", "Refers to the implementing type."),
    ("static", 13, "Rust keyword", "Defines a static item."),
    ("struct", 13, "Rust keyword", "Defines a struct."),
    ("super", 13, "Rust keyword", "Parent module."),
    ("trait", 13, "Rust keyword", "Defines a trait."),
    ("true", 13, "Rust keyword", "Boolean true."),
    ("type", 13, "Rust keyword", "Type alias."),
    ("union", 13, "Rust keyword", ""),
    ("unsafe", 13, "Rust keyword", "Unsafe code block."),
    ("use", 13, "Rust keyword", "Imports items into scope."),
    ("where", 13, "Rust keyword", "Where clause for generic bounds."),
    ("while", 13, "Rust keyword", "Loop while condition is true."),
    ("i8", 7, "primitive type", ""),
    ("i16", 7, "primitive type", ""),
    ("i32", 7, "primitive type", "32-bit signed integer."),
    ("i64", 7, "primitive type", "64-bit signed integer."),
    ("i128", 7, "primitive type", ""),
    ("isize", 7, "primitive type", ""),
    ("u8", 7, "primitive type", ""),
    ("u16", 7, "primitive type", ""),
    ("u32", 7, "primitive type", "32-bit unsigned integer."),
    ("u64", 7, "primitive type", "64-bit unsigned integer."),
    ("u128", 7, "primitive type", ""),
    ("usize", 7, "primitive type", "Pointer-sized unsigned integer."),
    ("f32", 7, "primitive type", "32-bit floating point."),
    ("f64", 7, "primitive type", "64-bit floating point."),
    ("bool", 7, "primitive type", "Boolean type."),
    ("char", 7, "primitive type", "Unicode scalar value."),
    ("str", 7, "primitive type", "String slice."),
    ("String", 7, "primitive type", "Owned heap-allocated string."),
    ("&str", 7, "primitive type", "Borrowed string slice."),
    ("Vec", 6, "std type", "Growable heap-allocated array."),
    ("HashMap", 6, "std type", "Hash map."),
    ("HashSet", 6, "std type", "Hash set."),
    ("BTreeMap", 6, "std type", ""),
    ("BTreeSet", 6, "std type", ""),
    ("Option", 6, "std type", "Optional value: Some(T) or None."),
    ("Result", 6, "std type", "Result type: Ok(T) or Err(E)."),
    ("Box", 6, "std type", "Heap-allocated pointer."),
    ("Rc", 6, "std type", "Reference-counted pointer."),
    ("Arc", 6, "std type", "Atomically reference-counted pointer."),
    ("Mutex", 6, "std type", "Mutual exclusion lock."),
    ("RwLock", 6, "std type", "Read-write lock."),
    ("Cell", 6, "std type", ""),
    ("RefCell", 6, "std type", ""),
    ("Cow", 6, "std type", ""),
    ("Path", 6, "std type", "Filesystem path slice."),
    ("PathBuf", 6, "std type", "Owned filesystem path."),
    ("File", 6, "std type", "Filesystem file."),
    ("BufReader", 6, "std type", ""),
    ("BufWriter", 6, "std type", ""),
    ("Stdin", 6, "std type", ""),
    ("Stdout", 6, "std type", ""),
    ("Stderr", 6, "std type", ""),
    ("Duration", 6, "std type", "Span of time."),
    ("Instant", 6, "std type", ""),
    ("Thread", 6, "std type", ""),
    ("JoinHandle", 6, "std type", ""),
    ("println!", 14, "macro", "Prints with newline to stdout."),
    ("print!", 14, "macro", "Prints without newline to stdout."),
    ("eprintln!", 14, "macro", "Prints with newline to stderr."),
    ("eprint!", 14, "macro", ""),
    ("format!", 14, "macro", "Creates a formatted String."),
    ("vec!", 14, "macro", "Creates a Vec."),
    ("assert!", 14, "macro", "Asserts condition is true, panics otherwise."),
    ("assert_eq!", 14, "macro", "Asserts two values are equal."),
    ("assert_ne!", 14, "macro", "Asserts two values are not equal."),
    ("panic!", 14, "macro", "Terminates the current thread with an error."),
    ("todo!", 14, "macro", "Marks unfinished code."),
    ("unimplemented!", 14, "macro", ""),
    ("unreachable!", 14, "macro", "Marks code as unreachable."),
    ("dbg!", 14, "macro", "Prints debug info and returns value."),
    ("include!", 14, "macro", ""),
    ("include_str!", 14, "macro", ""),
    ("include_bytes!", 14, "macro", ""),
    ("concat!", 14, "macro", ""),
    ("env!", 14, "macro", ""),
    ("cfg!", 14, "macro", ""),
    ("write!", 14, "macro", ""),
    ("writeln!", 14, "macro", ""),
    ("Clone", 7, "trait", "Allows explicit cloning."),
    ("Copy", 7, "trait", "Allows implicit bit-copy."),
    ("Debug", 7, "trait", "Debug formatting via {:?}."),
    ("Display", 7, "trait", "User-facing formatting via {}."),
    ("Default", 7, "trait", "Provides a default value."),
    ("PartialEq", 7, "trait", "Equality comparison."),
    ("Eq", 7, "trait", "Total equality (requires PartialEq)."),
    ("PartialOrd", 7, "trait", ""),
    ("Ord", 7, "trait", ""),
    ("Hash", 7, "trait", ""),
    ("Iterator", 7, "trait", "Lazy sequence of values."),
    ("IntoIterator", 7, "trait", ""),
    ("From", 7, "trait", "Conversion from another type."),
    ("Into", 7, "trait", "Conversion into another type."),
    ("AsRef", 7, "trait", ""),
    ("AsMut", 7, "trait", ""),
    ("Deref", 7, "trait", ""),
    ("DerefMut", 7, "trait", ""),
    ("Drop", 7, "trait", "Custom destructor."),
    ("Send", 7, "trait", "Safe to transfer across threads."),
    ("Sync", 7, "trait", "Safe to share across threads."),
    ("Sized", 7, "trait", ""),
    ("Fn", 7, "trait", ""),
    ("FnMut", 7, "trait", ""),
    ("FnOnce", 7, "trait", ""),
    ("Read", 7, "trait", ""),
    ("Write", 7, "trait", ""),
    ("Seek", 7, "trait", ""),
    ("BufRead", 7, "trait", ""),
    ("ToString", 7, "trait", "Conversion to String."),
    ("FromStr", 7, "trait", "Parsing from string."),
    ("Error", 7, "trait", "Trait for error types."),
    ("Any", 7, "trait", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "rust".to_string()
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
        assert_eq!(language_id(), "rust");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("fn", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
        for item in &items {
            let label = item["label"].as_str().unwrap().to_lowercase();
            assert!(label.starts_with("fn"), "label '{}' does not start with 'fn'", label);
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
        let result = get_hover("fn my_func() {}", 0, 1);
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
