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
    ("alignas", 13, "C++ keyword", ""),
    ("alignof", 13, "C++ keyword", ""),
    ("and", 13, "C++ keyword", ""),
    ("and_eq", 13, "C++ keyword", ""),
    ("asm", 13, "C++ keyword", ""),
    ("auto", 13, "C++ keyword", "Automatic type deduction."),
    ("bitand", 13, "C++ keyword", ""),
    ("bitor", 13, "C++ keyword", ""),
    ("bool", 13, "C++ keyword", ""),
    ("break", 13, "C++ keyword", ""),
    ("case", 13, "C++ keyword", ""),
    ("catch", 13, "C++ keyword", "Catches exceptions."),
    ("char", 13, "C++ keyword", ""),
    ("char8_t", 13, "C++ keyword", ""),
    ("char16_t", 13, "C++ keyword", ""),
    ("char32_t", 13, "C++ keyword", ""),
    ("class", 13, "C++ keyword", "Declares a class."),
    ("compl", 13, "C++ keyword", ""),
    ("concept", 13, "C++ keyword", ""),
    ("const", 13, "C++ keyword", "Immutable variable."),
    ("consteval", 13, "C++ keyword", ""),
    ("constexpr", 13, "C++ keyword", "Compile-time constant."),
    ("constinit", 13, "C++ keyword", ""),
    ("const_cast", 13, "C++ keyword", ""),
    ("continue", 13, "C++ keyword", ""),
    ("co_await", 13, "C++ keyword", ""),
    ("co_return", 13, "C++ keyword", ""),
    ("co_yield", 13, "C++ keyword", ""),
    ("decltype", 13, "C++ keyword", "Deduces expression type."),
    ("default", 13, "C++ keyword", ""),
    ("delete", 13, "C++ keyword", "Frees memory."),
    ("do", 13, "C++ keyword", ""),
    ("double", 13, "C++ keyword", ""),
    ("dynamic_cast", 13, "C++ keyword", ""),
    ("else", 13, "C++ keyword", ""),
    ("enum", 13, "C++ keyword", ""),
    ("explicit", 13, "C++ keyword", "Prevents implicit conversions."),
    ("export", 13, "C++ keyword", ""),
    ("extern", 13, "C++ keyword", ""),
    ("false", 13, "C++ keyword", "Boolean false."),
    ("float", 13, "C++ keyword", ""),
    ("for", 13, "C++ keyword", ""),
    ("friend", 13, "C++ keyword", "Grants access to private members."),
    ("goto", 13, "C++ keyword", ""),
    ("if", 13, "C++ keyword", ""),
    ("inline", 13, "C++ keyword", "Inline function hint."),
    ("int", 13, "C++ keyword", ""),
    ("long", 13, "C++ keyword", ""),
    ("mutable", 13, "C++ keyword", ""),
    ("namespace", 13, "C++ keyword", "Declares a namespace."),
    ("new", 13, "C++ keyword", "Allocates memory."),
    ("noexcept", 13, "C++ keyword", "Declares no exceptions thrown."),
    ("not", 13, "C++ keyword", ""),
    ("not_eq", 13, "C++ keyword", ""),
    ("nullptr", 13, "C++ keyword", "Null pointer literal."),
    ("operator", 13, "C++ keyword", "Operator overloading."),
    ("or", 13, "C++ keyword", ""),
    ("or_eq", 13, "C++ keyword", ""),
    ("private", 13, "C++ keyword", ""),
    ("protected", 13, "C++ keyword", ""),
    ("public", 13, "C++ keyword", ""),
    ("register", 13, "C++ keyword", ""),
    ("reinterpret_cast", 13, "C++ keyword", ""),
    ("requires", 13, "C++ keyword", ""),
    ("return", 13, "C++ keyword", "Returns from function."),
    ("short", 13, "C++ keyword", ""),
    ("signed", 13, "C++ keyword", ""),
    ("sizeof", 13, "C++ keyword", "Returns size in bytes."),
    ("static", 13, "C++ keyword", "Static member or local."),
    ("static_assert", 13, "C++ keyword", ""),
    ("static_cast", 13, "C++ keyword", ""),
    ("struct", 13, "C++ keyword", "Declares a struct."),
    ("switch", 13, "C++ keyword", ""),
    ("template", 13, "C++ keyword", "Generic programming."),
    ("this", 13, "C++ keyword", "Pointer to current object."),
    ("thread_local", 13, "C++ keyword", ""),
    ("throw", 13, "C++ keyword", "Throws an exception."),
    ("true", 13, "C++ keyword", "Boolean true."),
    ("try", 13, "C++ keyword", "Begins exception handling."),
    ("typedef", 13, "C++ keyword", ""),
    ("typeid", 13, "C++ keyword", ""),
    ("typename", 13, "C++ keyword", "Introduces a type name."),
    ("union", 13, "C++ keyword", ""),
    ("unsigned", 13, "C++ keyword", ""),
    ("using", 13, "C++ keyword", "Using declaration or directive."),
    ("virtual", 13, "C++ keyword", "Declares a virtual method."),
    ("void", 13, "C++ keyword", ""),
    ("volatile", 13, "C++ keyword", ""),
    ("wchar_t", 13, "C++ keyword", ""),
    ("while", 13, "C++ keyword", ""),
    ("xor", 13, "C++ keyword", ""),
    ("xor_eq", 13, "C++ keyword", ""),
    ("std::string", 6, "std::", "Standard string class."),
    ("std::vector", 6, "std::", "Dynamic array."),
    ("std::map", 6, "std::", "Sorted key-value store."),
    ("std::unordered_map", 6, "std::", "Hash map."),
    ("std::set", 6, "std::", ""),
    ("std::unordered_set", 6, "std::", ""),
    ("std::pair", 6, "std::", "Two-element struct."),
    ("std::tuple", 6, "std::", "Fixed-size heterogeneous collection."),
    ("std::array", 6, "std::", ""),
    ("std::list", 6, "std::", ""),
    ("std::deque", 6, "std::", ""),
    ("std::queue", 6, "std::", ""),
    ("std::stack", 6, "std::", ""),
    ("std::priority_queue", 6, "std::", ""),
    ("std::optional", 6, "std::", "Optional value (C++17)."),
    ("std::variant", 6, "std::", "Type-safe union (C++17)."),
    ("std::any", 6, "std::", "Type-safe container for any type (C++17)."),
    ("std::unique_ptr", 6, "std::", "Unique ownership smart pointer."),
    ("std::shared_ptr", 6, "std::", "Shared ownership smart pointer."),
    ("std::weak_ptr", 6, "std::", ""),
    ("std::function", 6, "std::", "Polymorphic function wrapper."),
    ("std::thread", 6, "std::", "Thread of execution."),
    ("std::mutex", 6, "std::", "Mutual exclusion."),
    ("std::condition_variable", 6, "std::", ""),
    ("std::atomic", 6, "std::", ""),
    ("std::future", 6, "std::", "Future async result."),
    ("std::promise", 6, "std::", ""),
    ("std::fstream", 6, "std::", ""),
    ("std::ifstream", 6, "std::", ""),
    ("std::ofstream", 6, "std::", ""),
    ("std::stringstream", 6, "std::", ""),
    ("std::exception", 6, "std::", ""),
    ("std::runtime_error", 6, "std::", ""),
    ("std::logic_error", 6, "std::", ""),
    ("NULL", 20, "preprocessor", ""),
    ("EOF", 20, "preprocessor", ""),
    ("INT_MAX", 20, "preprocessor", ""),
    ("INT_MIN", 20, "preprocessor", ""),
    ("UINT_MAX", 20, "preprocessor", ""),
    ("SIZE_MAX", 20, "preprocessor", ""),
    ("DBL_MAX", 20, "preprocessor", ""),
    ("FLT_MAX", 20, "preprocessor", ""),
    ("NDEBUG", 20, "preprocessor", ""),
    ("assert", 20, "preprocessor", ""),
    ("#include", 20, "preprocessor", ""),
    ("#define", 20, "preprocessor", ""),
    ("#ifdef", 20, "preprocessor", ""),
    ("#ifndef", 20, "preprocessor", ""),
    ("#endif", 20, "preprocessor", ""),
    ("#pragma", 20, "preprocessor", ""),
    ("#if", 20, "preprocessor", ""),
    ("#elif", 20, "preprocessor", ""),
    ("#else", 20, "preprocessor", ""),
    ("#error", 20, "preprocessor", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "cpp".to_string()
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
        assert_eq!(language_id(), "cpp");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("cl", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(items.len() > 0);
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
        assert!(parsed["contents"].as_str().unwrap().contains("Declares a class."));
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
