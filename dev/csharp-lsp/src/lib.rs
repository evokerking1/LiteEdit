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
    ("abstract", 13, "C# keyword", "Abstract class or member."),
    ("as", 13, "C# keyword", ""),
    ("base", 13, "C# keyword", "Parent class reference."),
    ("bool", 13, "C# keyword", ""),
    ("break", 13, "C# keyword", ""),
    ("byte", 13, "C# keyword", ""),
    ("case", 13, "C# keyword", "Switch case."),
    ("catch", 13, "C# keyword", "Catches exception."),
    ("char", 13, "C# keyword", ""),
    ("checked", 13, "C# keyword", ""),
    ("class", 13, "C# keyword", "Declares a class."),
    ("const", 13, "C# keyword", "Compile-time constant."),
    ("continue", 13, "C# keyword", ""),
    ("decimal", 13, "C# keyword", ""),
    ("default", 13, "C# keyword", ""),
    ("delegate", 13, "C# keyword", "Function pointer type."),
    ("do", 13, "C# keyword", ""),
    ("double", 13, "C# keyword", ""),
    ("else", 13, "C# keyword", "Else branch."),
    ("enum", 13, "C# keyword", "Declares an enumeration."),
    ("event", 13, "C# keyword", "Event declaration."),
    ("explicit", 13, "C# keyword", ""),
    ("extern", 13, "C# keyword", ""),
    ("false", 13, "C# keyword", "Boolean false."),
    ("finally", 13, "C# keyword", "Always executes."),
    ("fixed", 13, "C# keyword", ""),
    ("float", 13, "C# keyword", ""),
    ("for", 13, "C# keyword", "Loop."),
    ("foreach", 13, "C# keyword", "Iterates collection."),
    ("goto", 13, "C# keyword", ""),
    ("if", 13, "C# keyword", "Conditional."),
    ("implicit", 13, "C# keyword", ""),
    ("in", 13, "C# keyword", ""),
    ("int", 13, "C# keyword", ""),
    ("interface", 13, "C# keyword", "Declares an interface."),
    ("internal", 13, "C# keyword", "Assembly-internal access."),
    ("is", 13, "C# keyword", ""),
    ("lock", 13, "C# keyword", "Thread synchronization."),
    ("long", 13, "C# keyword", ""),
    ("namespace", 13, "C# keyword", "Declares a namespace."),
    ("new", 13, "C# keyword", "Allocates instance or hides member."),
    ("null", 13, "C# keyword", "Null literal."),
    ("object", 13, "C# keyword", ""),
    ("operator", 13, "C# keyword", ""),
    ("out", 13, "C# keyword", ""),
    ("override", 13, "C# keyword", "Overrides a virtual member."),
    ("params", 13, "C# keyword", ""),
    ("private", 13, "C# keyword", "Private access modifier."),
    ("protected", 13, "C# keyword", "Protected access modifier."),
    ("public", 13, "C# keyword", "Public access modifier."),
    ("readonly", 13, "C# keyword", "Runtime constant field."),
    ("ref", 13, "C# keyword", ""),
    ("return", 13, "C# keyword", "Returns from method."),
    ("sbyte", 13, "C# keyword", ""),
    ("sealed", 13, "C# keyword", "Prevents class inheritance."),
    ("short", 13, "C# keyword", ""),
    ("sizeof", 13, "C# keyword", "Gets size in bytes."),
    ("stackalloc", 13, "C# keyword", ""),
    ("static", 13, "C# keyword", "Class-level member."),
    ("string", 13, "C# keyword", ""),
    ("struct", 13, "C# keyword", "Declares a value type struct."),
    ("switch", 13, "C# keyword", "Multi-branch."),
    ("this", 13, "C# keyword", "Current instance reference."),
    ("throw", 13, "C# keyword", "Throws exception."),
    ("true", 13, "C# keyword", "Boolean true."),
    ("try", 13, "C# keyword", "Exception handling."),
    ("typeof", 13, "C# keyword", "Gets System.Type."),
    ("uint", 13, "C# keyword", ""),
    ("ulong", 13, "C# keyword", ""),
    ("unchecked", 13, "C# keyword", ""),
    ("unsafe", 13, "C# keyword", ""),
    ("ushort", 13, "C# keyword", ""),
    ("using", 13, "C# keyword", "Imports a namespace or resource."),
    ("var", 13, "C# keyword", "Implicitly typed local variable."),
    ("virtual", 13, "C# keyword", "Overridable member."),
    ("void", 13, "C# keyword", ""),
    ("volatile", 13, "C# keyword", ""),
    ("while", 13, "C# keyword", "Loop."),
    ("async", 13, "C# keyword", "Async method modifier."),
    ("await", 13, "C# keyword", "Awaits a Task."),
    ("dynamic", 13, "C# keyword", ""),
    ("record", 13, "C# keyword", "Immutable reference type (C#9+)."),
    ("init", 13, "C# keyword", "Init-only setter (C#9+)."),
    ("nint", 13, "C# keyword", ""),
    ("nuint", 13, "C# keyword", ""),
    ("with", 13, "C# keyword", "Non-destructive mutation (C#9+)."),
    ("global", 13, "C# keyword", ""),
    ("file", 13, "C# keyword", ""),
    ("required", 13, "C# keyword", ""),
    ("scoped", 13, "C# keyword", ""),
    ("String", 6, "System", "Immutable sequence of characters."),
    ("Int32", 6, "System", ""),
    ("Int64", 6, "System", ""),
    ("Double", 6, "System", ""),
    ("Decimal", 6, "System", ""),
    ("Boolean", 6, "System", ""),
    ("Char", 6, "System", ""),
    ("Byte", 6, "System", ""),
    ("DateTime", 6, "System", "Date and time representation."),
    ("DateOnly", 6, "System", ""),
    ("TimeOnly", 6, "System", ""),
    ("TimeSpan", 6, "System", ""),
    ("Guid", 6, "System", "Globally unique identifier."),
    ("Uri", 6, "System", ""),
    ("Version", 6, "System", ""),
    ("List", 6, "System", "Generic dynamic list."),
    ("Dictionary", 6, "System", "Key-value collection."),
    ("HashSet", 6, "System", ""),
    ("Queue", 6, "System", ""),
    ("Stack", 6, "System", ""),
    ("LinkedList", 6, "System", ""),
    ("Array", 6, "System", ""),
    ("IEnumerable", 6, "System", "Iteration interface."),
    ("IList", 6, "System", ""),
    ("IDictionary", 6, "System", ""),
    ("ICollection", 6, "System", ""),
    ("IQueryable", 6, "System", ""),
    ("Task", 6, "System", "Represents async operation."),
    ("Thread", 6, "System", ""),
    ("Mutex", 6, "System", ""),
    ("Monitor", 6, "System", ""),
    ("Semaphore", 6, "System", ""),
    ("CancellationToken", 6, "System", "Cancellation signal."),
    ("Exception", 6, "System", "Base exception class."),
    ("ArgumentException", 6, "System", ""),
    ("InvalidOperationException", 6, "System", ""),
    ("NullReferenceException", 6, "System", ""),
    ("NotImplementedException", 6, "System", ""),
    ("Console", 6, "System", "Console I/O."),
    ("Math", 6, "System", "Mathematical functions."),
    ("Random", 6, "System", ""),
    ("StringBuilder", 6, "System", "Mutable string builder."),
    ("Regex", 6, "System", "Regular expression engine."),
    ("Environment", 6, "System", ""),
    ("Path", 6, "System", "Path operations."),
    ("File", 6, "System", "File I/O operations."),
    ("Directory", 6, "System", ""),
    ("Stream", 6, "System", ""),
    ("StreamReader", 6, "System", ""),
    ("StreamWriter", 6, "System", ""),
    ("HttpClient", 6, "System", "HTTP client."),
    ("JsonSerializer", 6, "System", "JSON serialization (System.Text.Json)."),
    ("Enumerable", 6, "System", "LINQ extension methods."),
    ("Activator", 6, "System", ""),
    ("Convert", 6, "System", ""),
    ("Tuple", 6, "System", ""),
    ("ValueTuple", 6, "System", ""),
    ("Console.WriteLine", 1, "method", "Writes a line to stdout."),
    ("Console.ReadLine", 1, "method", "Reads a line from stdin."),
    ("String.Format", 1, "method", "Formats a string."),
    ("String.IsNullOrEmpty", 1, "method", "Checks for null or empty."),
    ("String.IsNullOrWhiteSpace", 1, "method", ""),
    ("String.Join", 1, "method", ""),
    ("String.Concat", 1, "method", ""),
    ("Convert.ToInt32", 1, "method", "Converts to 32-bit integer."),
    ("Convert.ToString", 1, "method", ""),
    ("Math.Abs", 1, "method", "Absolute value."),
    ("Math.Floor", 1, "method", ""),
    ("Math.Ceiling", 1, "method", ""),
    ("Math.Round", 1, "method", ""),
    ("Math.Max", 1, "method", ""),
    ("Math.Min", 1, "method", ""),
    ("Math.Sqrt", 1, "method", ""),
    ("Path.Combine", 1, "method", ""),
    ("Path.GetFileName", 1, "method", ""),
    ("Path.GetExtension", 1, "method", ""),
    ("File.ReadAllText", 1, "method", "Reads file as string."),
    ("File.WriteAllText", 1, "method", "Writes string to file."),
    ("Enumerable.Range", 1, "method", "Generates integer sequence."),
    ("Enumerable.Repeat", 1, "method", ""),
    ("Task.Run", 1, "method", "Runs delegate on thread pool."),
    ("Task.Delay", 1, "method", ""),
    ("Task.WhenAll", 1, "method", "Awaits all tasks."),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "csharp".to_string()
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
        assert_eq!(language_id(), "csharp");
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
