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
    ("associatedtype", 13, "Swift keyword", "Associated type in protocol."),
    ("class", 13, "Swift keyword", "Defines a reference type class."),
    ("deinit", 13, "Swift keyword", "Deinitializer."),
    ("enum", 13, "Swift keyword", "Defines an enumeration."),
    ("extension", 13, "Swift keyword", "Extends an existing type."),
    ("fileprivate", 13, "Swift keyword", "File-private access."),
    ("func", 13, "Swift keyword", "Defines a function."),
    ("import", 13, "Swift keyword", "Imports a module."),
    ("init", 13, "Swift keyword", "Initializer."),
    ("inout", 13, "Swift keyword", "Pass by reference."),
    ("internal", 13, "Swift keyword", "Module-internal access."),
    ("let", 13, "Swift keyword", "Immutable constant."),
    ("open", 13, "Swift keyword", "Open access modifier."),
    ("operator", 13, "Swift keyword", ""),
    ("private", 13, "Swift keyword", "Declaration-only access."),
    ("precedencegroup", 13, "Swift keyword", ""),
    ("protocol", 13, "Swift keyword", "Defines a protocol (interface)."),
    ("public", 13, "Swift keyword", "Publicly accessible."),
    ("rethrows", 13, "Swift keyword", ""),
    ("static", 13, "Swift keyword", "Type-level member."),
    ("struct", 13, "Swift keyword", "Defines a value type struct."),
    ("subscript", 13, "Swift keyword", "Subscript access."),
    ("typealias", 13, "Swift keyword", "Type alias."),
    ("var", 13, "Swift keyword", "Mutable variable."),
    ("break", 13, "Swift keyword", ""),
    ("case", 13, "Swift keyword", "Switch case."),
    ("continue", 13, "Swift keyword", ""),
    ("default", 13, "Swift keyword", ""),
    ("defer", 13, "Swift keyword", "Deferred execution until scope exit."),
    ("do", 13, "Swift keyword", "Error handling scope."),
    ("else", 13, "Swift keyword", "Else branch."),
    ("fallthrough", 13, "Swift keyword", ""),
    ("for", 13, "Swift keyword", "For-in loop."),
    ("guard", 13, "Swift keyword", "Early exit conditional."),
    ("if", 13, "Swift keyword", "Conditional."),
    ("in", 13, "Swift keyword", "Closure parameter separator / for-in."),
    ("repeat", 13, "Swift keyword", "Repeat-while loop."),
    ("return", 13, "Swift keyword", "Returns value."),
    ("throw", 13, "Swift keyword", "Throws an error."),
    ("switch", 13, "Swift keyword", "Multi-branch switch (exhaustive)."),
    ("where", 13, "Swift keyword", "Type constraint."),
    ("while", 13, "Swift keyword", "While loop."),
    ("as", 13, "Swift keyword", "Type cast."),
    ("Any", 13, "Swift keyword", ""),
    ("catch", 13, "Swift keyword", "Catches error."),
    ("false", 13, "Swift keyword", "Boolean false."),
    ("is", 13, "Swift keyword", "Type check."),
    ("nil", 13, "Swift keyword", "Null value."),
    ("super", 13, "Swift keyword", "Parent type."),
    ("self", 13, "Swift keyword", "Current instance."),
    ("Self", 13, "Swift keyword", "Current type."),
    ("throws", 13, "Swift keyword", "Marks throwing function."),
    ("true", 13, "Swift keyword", "Boolean true."),
    ("try", 13, "Swift keyword", "Calls throwing function."),
    ("#available", 13, "Swift keyword", "Availability check."),
    ("#colorLiteral", 13, "Swift keyword", ""),
    ("#column", 13, "Swift keyword", ""),
    ("#dsohandle", 13, "Swift keyword", ""),
    ("#elseif", 13, "Swift keyword", ""),
    ("#else", 13, "Swift keyword", ""),
    ("#endif", 13, "Swift keyword", ""),
    ("#error", 13, "Swift keyword", ""),
    ("#fileID", 13, "Swift keyword", ""),
    ("#fileLiteral", 13, "Swift keyword", ""),
    ("#filePath", 13, "Swift keyword", ""),
    ("#file", 13, "Swift keyword", "Current file name."),
    ("#function", 13, "Swift keyword", "Current function name."),
    ("#if", 13, "Swift keyword", "Conditional compilation."),
    ("#imageLiteral", 13, "Swift keyword", ""),
    ("#keyPath", 13, "Swift keyword", ""),
    ("#line", 13, "Swift keyword", "Current line number."),
    ("#selector", 13, "Swift keyword", ""),
    ("#sourceLocation", 13, "Swift keyword", ""),
    ("#warning", 13, "Swift keyword", ""),
    ("associativity", 13, "Swift keyword", ""),
    ("convenience", 13, "Swift keyword", "Convenience initializer."),
    ("didSet", 13, "Swift keyword", "Property observer (after)."),
    ("dynamic", 13, "Swift keyword", ""),
    ("final", 13, "Swift keyword", "Prevents subclassing/overriding."),
    ("get", 13, "Swift keyword", "Getter."),
    ("indirect", 13, "Swift keyword", "Indirect enum case."),
    ("infix", 13, "Swift keyword", ""),
    ("lazy", 13, "Swift keyword", "Lazy initialization."),
    ("left", 13, "Swift keyword", ""),
    ("mutating", 13, "Swift keyword", "Mutates value type."),
    ("none", 13, "Swift keyword", ""),
    ("nonmutating", 13, "Swift keyword", "Marks non-mutating version."),
    ("optional", 13, "Swift keyword", "Protocol optional member."),
    ("override", 13, "Swift keyword", "Overrides inherited member."),
    ("postfix", 13, "Swift keyword", ""),
    ("precedence", 13, "Swift keyword", ""),
    ("prefix", 13, "Swift keyword", ""),
    ("Protocol", 13, "Swift keyword", ""),
    ("required", 13, "Swift keyword", "Required initializer."),
    ("right", 13, "Swift keyword", ""),
    ("set", 13, "Swift keyword", "Setter."),
    ("some", 13, "Swift keyword", "Opaque return type."),
    ("Type", 13, "Swift keyword", ""),
    ("unowned", 13, "Swift keyword", "Unowned reference."),
    ("weak", 13, "Swift keyword", "Weak reference."),
    ("willSet", 13, "Swift keyword", "Property observer (before)."),
    ("Actor", 13, "Swift keyword", "Concurrency actor."),
    ("async", 13, "Swift keyword", "Async function."),
    ("await", 13, "Swift keyword", "Awaits async expression."),
    ("isolated", 13, "Swift keyword", "Actor isolation."),
    ("nonisolated", 13, "Swift keyword", "Non-isolated member."),
    ("distributed", 13, "Swift keyword", ""),
    ("consume", 13, "Swift keyword", ""),
    ("copy", 13, "Swift keyword", ""),
    ("borrowing", 13, "Swift keyword", ""),
    ("sending", 13, "Swift keyword", ""),
    ("Int", 6, "stdlib type", "Integer type."),
    ("Int8", 6, "stdlib type", ""),
    ("Int16", 6, "stdlib type", ""),
    ("Int32", 6, "stdlib type", ""),
    ("Int64", 6, "stdlib type", ""),
    ("UInt", 6, "stdlib type", ""),
    ("UInt8", 6, "stdlib type", ""),
    ("UInt16", 6, "stdlib type", ""),
    ("UInt32", 6, "stdlib type", ""),
    ("UInt64", 6, "stdlib type", ""),
    ("Float", 6, "stdlib type", "32-bit float."),
    ("Double", 6, "stdlib type", "64-bit float."),
    ("Bool", 6, "stdlib type", "Boolean."),
    ("Character", 6, "stdlib type", "Single Unicode character."),
    ("String", 6, "stdlib type", "Unicode string."),
    ("Substring", 6, "stdlib type", ""),
    ("Unicode", 6, "stdlib type", ""),
    ("Array", 6, "stdlib type", "Ordered collection."),
    ("ContiguousArray", 6, "stdlib type", ""),
    ("ArraySlice", 6, "stdlib type", ""),
    ("Dictionary", 6, "stdlib type", "Key-value collection."),
    ("Set", 6, "stdlib type", "Unordered unique collection."),
    ("Optional", 6, "stdlib type", "Optional value (Some/None)."),
    ("Result", 6, "stdlib type", "Success or failure value."),
    ("Never", 6, "stdlib type", "Type that never returns."),
    ("AnyObject", 6, "stdlib type", ""),
    ("AnyClass", 6, "stdlib type", ""),
    ("Void", 6, "stdlib type", "Empty tuple / no value."),
    ("Comparable", 6, "stdlib type", "Supports comparison."),
    ("Equatable", 6, "stdlib type", "Supports equality."),
    ("Hashable", 6, "stdlib type", "Supports hashing."),
    ("Codable", 6, "stdlib type", "Supports encode/decode."),
    ("Encodable", 6, "stdlib type", ""),
    ("Decodable", 6, "stdlib type", ""),
    ("Error", 6, "stdlib type", "Error protocol."),
    ("LocalizedError", 6, "stdlib type", ""),
    ("Sequence", 6, "stdlib type", "Iterable sequence."),
    ("Collection", 6, "stdlib type", "Indexed collection."),
    ("BidirectionalCollection", 6, "stdlib type", ""),
    ("RandomAccessCollection", 6, "stdlib type", ""),
    ("MutableCollection", 6, "stdlib type", ""),
    ("RangeReplaceableCollection", 6, "stdlib type", ""),
    ("IteratorProtocol", 6, "stdlib type", ""),
    ("LazySequence", 6, "stdlib type", ""),
    ("LazyCollection", 6, "stdlib type", ""),
    ("Range", 6, "stdlib type", "Half-open range."),
    ("ClosedRange", 6, "stdlib type", "Closed range."),
    ("PartialRangeFrom", 6, "stdlib type", ""),
    ("PartialRangeThrough", 6, "stdlib type", ""),
    ("PartialRangeUpTo", 6, "stdlib type", ""),
    ("Stride", 6, "stdlib type", ""),
    ("Mirror", 6, "stdlib type", ""),
    ("KeyPath", 6, "stdlib type", "Type-safe key path."),
    ("WritableKeyPath", 6, "stdlib type", ""),
    ("ReferenceWritableKeyPath", 6, "stdlib type", ""),
    ("PartialKeyPath", 6, "stdlib type", ""),
    ("AnyKeyPath", 6, "stdlib type", ""),
    ("print", 5, "built-in", "Prints to stdout."),
    ("debugPrint", 5, "built-in", "Debug print."),
    ("dump", 5, "built-in", ""),
    ("readLine", 5, "built-in", "Reads line from stdin."),
    ("min", 5, "built-in", "Minimum of two values."),
    ("max", 5, "built-in", "Maximum of two values."),
    ("abs", 5, "built-in", "Absolute value."),
    ("stride", 5, "built-in", "Stride over range."),
    ("zip", 5, "built-in", "Zips two sequences."),
    ("sequence", 5, "built-in", ""),
    ("repeatElement", 5, "built-in", ""),
    ("CollectionOfOne", 5, "built-in", ""),
    ("EmptyCollection", 5, "built-in", ""),
    ("swap", 5, "built-in", "Swaps two values."),
    ("withUnsafePointer", 5, "built-in", ""),
    ("withUnsafeMutablePointer", 5, "built-in", ""),
    ("withUnsafeBytes", 5, "built-in", ""),
    ("withUnsafeMutableBytes", 5, "built-in", ""),
    ("precondition", 5, "built-in", "Runtime condition check."),
    ("preconditionFailure", 5, "built-in", ""),
    ("fatalError", 5, "built-in", "Unconditional fatal error."),
    ("assert", 5, "built-in", "Debug assertion."),
    ("assertionFailure", 5, "built-in", ""),
    ("type(of:)", 5, "built-in", "Returns dynamic type."),
    ("unsafeBitCast", 5, "built-in", ""),
    ("unsafeDowncast", 5, "built-in", ""),
    ("MemoryLayout", 5, "built-in", "Memory layout info."),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "swift".to_string()
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
        assert_eq!(language_id(), "swift");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("fu", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
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
