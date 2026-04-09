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
    ("__ENCODING__", 13, "Ruby keyword", ""),
    ("__LINE__", 13, "Ruby keyword", ""),
    ("__FILE__", 13, "Ruby keyword", ""),
    ("BEGIN", 13, "Ruby keyword", "Code run before program."),
    ("END", 13, "Ruby keyword", "Code run after program."),
    ("alias", 13, "Ruby keyword", "Creates method alias."),
    ("and", 13, "Ruby keyword", "Logical AND (low precedence)."),
    ("begin", 13, "Ruby keyword", "Exception handling block."),
    ("break", 13, "Ruby keyword", "Exits loop/block."),
    ("case", 13, "Ruby keyword", ""),
    ("class", 13, "Ruby keyword", "Defines a class."),
    ("def", 13, "Ruby keyword", "Defines a method."),
    ("defined?", 13, "Ruby keyword", "Checks if expression is defined."),
    ("do", 13, "Ruby keyword", "Begins a block."),
    ("else", 13, "Ruby keyword", "Else branch."),
    ("elsif", 13, "Ruby keyword", "Else-if branch."),
    ("end", 13, "Ruby keyword", "Closes a block/class/def."),
    ("ensure", 13, "Ruby keyword", "Always executes."),
    ("false", 13, "Ruby keyword", "Boolean false."),
    ("for", 13, "Ruby keyword", "For loop."),
    ("if", 13, "Ruby keyword", "Conditional."),
    ("in", 13, "Ruby keyword", "For-in iterator."),
    ("module", 13, "Ruby keyword", "Defines a module."),
    ("next", 13, "Ruby keyword", "Goes to next iteration."),
    ("nil", 13, "Ruby keyword", "Null value."),
    ("not", 13, "Ruby keyword", "Logical NOT."),
    ("or", 13, "Ruby keyword", "Logical OR (low precedence)."),
    ("raise", 13, "Ruby keyword", "Raises an exception."),
    ("redo", 13, "Ruby keyword", "Redo current iteration."),
    ("rescue", 13, "Ruby keyword", "Catches exceptions."),
    ("retry", 13, "Ruby keyword", "Retries rescue block."),
    ("return", 13, "Ruby keyword", "Returns from method."),
    ("self", 13, "Ruby keyword", "Current object reference."),
    ("super", 13, "Ruby keyword", "Calls parent method."),
    ("then", 13, "Ruby keyword", "Then for one-liner conditionals."),
    ("true", 13, "Ruby keyword", "Boolean true."),
    ("undef", 13, "Ruby keyword", "Removes method definition."),
    ("unless", 13, "Ruby keyword", "Negative conditional."),
    ("until", 13, "Ruby keyword", "Loop until condition."),
    ("when", 13, "Ruby keyword", "Case branch."),
    ("while", 13, "Ruby keyword", "Loop."),
    ("yield", 13, "Ruby keyword", "Yields to block."),
    ("Array", 6, "built-in", "Ordered collection."),
    ("Hash", 6, "built-in", "Key-value store."),
    ("String", 6, "built-in", "Character sequence."),
    ("Integer", 6, "built-in", "Integer number."),
    ("Float", 6, "built-in", "Floating point number."),
    ("Symbol", 6, "built-in", "Immutable identifier."),
    ("Range", 6, "built-in", "Contiguous sequence."),
    ("Regexp", 6, "built-in", "Regular expression."),
    ("Proc", 6, "built-in", "Encapsulated block."),
    ("Method", 6, "built-in", ""),
    ("IO", 6, "built-in", "Input/output stream."),
    ("File", 6, "built-in", "File operations."),
    ("Dir", 6, "built-in", "Directory operations."),
    ("Env", 6, "built-in", ""),
    ("Kernel", 6, "built-in", "Core object methods."),
    ("Object", 6, "built-in", ""),
    ("Module", 6, "built-in", ""),
    ("Class", 6, "built-in", ""),
    ("Numeric", 6, "built-in", ""),
    ("Comparable", 6, "built-in", "Comparison mixin."),
    ("Enumerable", 6, "built-in", "Collection traversal mixin."),
    ("Enumerator", 6, "built-in", ""),
    ("puts", 1, "method", "Prints with newline."),
    ("print", 1, "method", "Prints without newline."),
    ("p", 1, "method", "Prints inspect."),
    ("pp", 1, "method", ""),
    ("gets", 1, "method", ""),
    ("require", 1, "method", "Loads a library."),
    ("require_relative", 1, "method", ""),
    ("attr_reader", 1, "method", ""),
    ("attr_writer", 1, "method", ""),
    ("attr_accessor", 1, "method", "Creates read/write accessor."),
    ("include", 1, "method", "Mixes in module."),
    ("extend", 1, "method", ""),
    ("prepend", 1, "method", ""),
    ("map", 1, "method", "Transforms collection."),
    ("select", 1, "method", "Filters collection."),
    ("reject", 1, "method", "Inverse filter."),
    ("each", 1, "method", "Iterates collection."),
    ("flat_map", 1, "method", ""),
    ("reduce", 1, "method", "Folds collection."),
    ("inject", 1, "method", ""),
    ("all?", 1, "method", ""),
    ("any?", 1, "method", ""),
    ("none?", 1, "method", ""),
    ("one?", 1, "method", ""),
    ("count", 1, "method", ""),
    ("first", 1, "method", ""),
    ("last", 1, "method", ""),
    ("sort", 1, "method", "Sorts collection."),
    ("sort_by", 1, "method", ""),
    ("min", 1, "method", ""),
    ("max", 1, "method", ""),
    ("min_by", 1, "method", ""),
    ("max_by", 1, "method", ""),
    ("group_by", 1, "method", ""),
    ("each_with_object", 1, "method", ""),
    ("zip", 1, "method", ""),
    ("flatten", 1, "method", "Flattens nested array."),
    ("compact", 1, "method", "Removes nil values."),
    ("uniq", 1, "method", "Removes duplicates."),
    ("length", 1, "method", ""),
    ("size", 1, "method", ""),
    ("empty?", 1, "method", "Checks if empty."),
    ("nil?", 1, "method", "Checks if nil."),
    ("to_s", 1, "method", "Converts to string."),
    ("to_i", 1, "method", ""),
    ("to_f", 1, "method", ""),
    ("to_a", 1, "method", "Converts to array."),
    ("to_h", 1, "method", ""),
    ("to_sym", 1, "method", ""),
    ("freeze", 1, "method", "Makes object immutable."),
    ("frozen?", 1, "method", ""),
    ("dup", 1, "method", ""),
    ("clone", 1, "method", ""),
    ("respond_to?", 1, "method", "Checks method availability."),
    ("send", 1, "method", ""),
    ("method", 1, "method", ""),
    ("inspect", 1, "method", ""),
    ("object_id", 1, "method", ""),
    ("class", 1, "method", ""),
    ("is_a?", 1, "method", ""),
    ("kind_of?", 1, "method", ""),
    ("instance_of?", 1, "method", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "ruby".to_string()
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
        assert_eq!(language_id(), "ruby");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("de", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
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
        let result = get_hover("def my_method", 0, 1);
        assert_ne!(result, "null");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["contents"].as_str().unwrap().contains("Defines a method."));
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
