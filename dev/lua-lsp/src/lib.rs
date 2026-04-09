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
    ("and", 13, "Lua keyword", "Logical AND."),
    ("break", 13, "Lua keyword", "Exits loop."),
    ("do", 13, "Lua keyword", "Opens a block."),
    ("else", 13, "Lua keyword", "Else branch."),
    ("elseif", 13, "Lua keyword", "Else-if branch."),
    ("end", 13, "Lua keyword", "Closes a block."),
    ("false", 13, "Lua keyword", "Boolean false."),
    ("for", 13, "Lua keyword", "Numeric or generic for loop."),
    ("function", 13, "Lua keyword", "Defines a function."),
    ("goto", 13, "Lua keyword", "Unconditional jump (Lua 5.2+)."),
    ("if", 13, "Lua keyword", "Conditional."),
    ("in", 13, "Lua keyword", "Generic for iteration."),
    ("local", 13, "Lua keyword", "Declares local variable."),
    ("nil", 13, "Lua keyword", "Null value."),
    ("not", 13, "Lua keyword", "Logical NOT."),
    ("or", 13, "Lua keyword", "Logical OR."),
    ("repeat", 13, "Lua keyword", "Repeat-until loop."),
    ("return", 13, "Lua keyword", "Returns from function."),
    ("then", 13, "Lua keyword", "Then for if conditions."),
    ("true", 13, "Lua keyword", "Boolean true."),
    ("until", 13, "Lua keyword", "Until condition for repeat loop."),
    ("while", 13, "Lua keyword", "While loop."),
    ("io", 6, "std library", "Input/output operations."),
    ("os", 6, "std library", "OS functions."),
    ("math", 6, "std library", "Mathematical functions."),
    ("string", 6, "std library", "String manipulation."),
    ("table", 6, "std library", "Table manipulation."),
    ("coroutine", 6, "std library", "Coroutine control."),
    ("package", 6, "std library", "Package/module system."),
    ("debug", 6, "std library", "Debug interface."),
    ("utf8", 6, "std library", "UTF-8 support."),
    ("bit32", 6, "std library", ""),
    ("print", 5, "built-in", "Prints to stdout."),
    ("ipairs", 5, "built-in", "Iterates array part of table."),
    ("pairs", 5, "built-in", "Iterates all table pairs."),
    ("next", 5, "built-in", ""),
    ("select", 5, "built-in", ""),
    ("type", 5, "built-in", "Returns type as string."),
    ("error", 5, "built-in", "Raises error."),
    ("assert", 5, "built-in", "Errors if condition false."),
    ("pcall", 5, "built-in", "Protected call, catches errors."),
    ("xpcall", 5, "built-in", ""),
    ("tostring", 5, "built-in", "Converts to string."),
    ("tonumber", 5, "built-in", "Converts to number."),
    ("rawget", 5, "built-in", ""),
    ("rawset", 5, "built-in", ""),
    ("rawequal", 5, "built-in", ""),
    ("rawlen", 5, "built-in", ""),
    ("setmetatable", 5, "built-in", "Sets table metatable."),
    ("getmetatable", 5, "built-in", "Gets table metatable."),
    ("require", 5, "built-in", "Loads a module."),
    ("load", 5, "built-in", "Loads chunk from string."),
    ("loadfile", 5, "built-in", ""),
    ("dofile", 5, "built-in", ""),
    ("collectgarbage", 5, "built-in", ""),
    ("unpack", 5, "built-in", ""),
    ("table.unpack", 5, "built-in", ""),
    ("string.format", 5, "built-in", "Formats string (like printf)."),
    ("string.len", 5, "built-in", ""),
    ("string.sub", 5, "built-in", ""),
    ("string.upper", 5, "built-in", ""),
    ("string.lower", 5, "built-in", ""),
    ("string.rep", 5, "built-in", ""),
    ("string.reverse", 5, "built-in", ""),
    ("string.byte", 5, "built-in", ""),
    ("string.char", 5, "built-in", ""),
    ("string.find", 5, "built-in", ""),
    ("string.match", 5, "built-in", ""),
    ("string.gmatch", 5, "built-in", ""),
    ("string.gsub", 5, "built-in", ""),
    ("table.insert", 5, "built-in", "Inserts into table."),
    ("table.remove", 5, "built-in", "Removes from table."),
    ("table.sort", 5, "built-in", "Sorts table in place."),
    ("table.concat", 5, "built-in", ""),
    ("table.move", 5, "built-in", ""),
    ("math.floor", 5, "built-in", "Rounds down."),
    ("math.ceil", 5, "built-in", "Rounds up."),
    ("math.abs", 5, "built-in", "Absolute value."),
    ("math.sqrt", 5, "built-in", "Square root."),
    ("math.random", 5, "built-in", "Random number."),
    ("math.max", 5, "built-in", ""),
    ("math.min", 5, "built-in", ""),
    ("math.sin", 5, "built-in", ""),
    ("math.cos", 5, "built-in", ""),
    ("math.tan", 5, "built-in", ""),
    ("math.log", 5, "built-in", ""),
    ("math.exp", 5, "built-in", ""),
    ("math.huge", 5, "built-in", ""),
    ("math.pi", 5, "built-in", ""),
    ("io.write", 5, "built-in", ""),
    ("io.read", 5, "built-in", ""),
    ("io.open", 5, "built-in", "Opens a file."),
    ("io.close", 5, "built-in", ""),
    ("os.time", 5, "built-in", "Current Unix time."),
    ("os.date", 5, "built-in", "Formatted date/time."),
    ("os.clock", 5, "built-in", ""),
    ("os.exit", 5, "built-in", ""),
    ("__index", 20, "metamethod", "Called on missing key access."),
    ("__newindex", 20, "metamethod", "Called on new key assignment."),
    ("__call", 20, "metamethod", "Called when table is called."),
    ("__tostring", 20, "metamethod", "Custom string representation."),
    ("__len", 20, "metamethod", "Length operator (#)."),
    ("__eq", 20, "metamethod", "Equality comparison."),
    ("__lt", 20, "metamethod", "Less-than comparison."),
    ("__le", 20, "metamethod", "Less-or-equal comparison."),
    ("__add", 20, "metamethod", "Addition operator."),
    ("__sub", 20, "metamethod", "Subtraction operator."),
    ("__mul", 20, "metamethod", "Multiplication operator."),
    ("__div", 20, "metamethod", "Division operator."),
    ("__mod", 20, "metamethod", ""),
    ("__pow", 20, "metamethod", ""),
    ("__unm", 20, "metamethod", ""),
    ("__idiv", 20, "metamethod", ""),
    ("__band", 20, "metamethod", ""),
    ("__bor", 20, "metamethod", ""),
    ("__bxor", 20, "metamethod", ""),
    ("__bnot", 20, "metamethod", ""),
    ("__shl", 20, "metamethod", ""),
    ("__shr", 20, "metamethod", ""),
    ("__concat", 20, "metamethod", "Concatenation operator (..)."),
    ("__gc", 20, "metamethod", "Garbage collector finalizer."),
    ("__close", 20, "metamethod", "To-be-closed variable hook."),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "lua".to_string()
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
        assert_eq!(language_id(), "lua");
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
        let result = get_hover("function my_func() end", 0, 5);
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
