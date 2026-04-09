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
    ("__halt_compiler", 13, "PHP keyword", ""),
    ("abstract", 13, "PHP keyword", "Abstract class or method."),
    ("and", 13, "PHP keyword", ""),
    ("array", 13, "PHP keyword", "Creates an array."),
    ("as", 13, "PHP keyword", ""),
    ("break", 13, "PHP keyword", ""),
    ("callable", 13, "PHP keyword", ""),
    ("case", 13, "PHP keyword", ""),
    ("catch", 13, "PHP keyword", "Catches exception."),
    ("class", 13, "PHP keyword", "Defines a class."),
    ("clone", 13, "PHP keyword", ""),
    ("const", 13, "PHP keyword", ""),
    ("continue", 13, "PHP keyword", ""),
    ("declare", 13, "PHP keyword", ""),
    ("default", 13, "PHP keyword", ""),
    ("die", 13, "PHP keyword", "Outputs and exits."),
    ("do", 13, "PHP keyword", ""),
    ("echo", 13, "PHP keyword", "Outputs one or more strings."),
    ("else", 13, "PHP keyword", "Else branch."),
    ("elseif", 13, "PHP keyword", "Else-if branch."),
    ("empty", 13, "PHP keyword", "Checks if variable is empty."),
    ("enddeclare", 13, "PHP keyword", ""),
    ("endfor", 13, "PHP keyword", ""),
    ("endforeach", 13, "PHP keyword", ""),
    ("endif", 13, "PHP keyword", ""),
    ("endswitch", 13, "PHP keyword", ""),
    ("endwhile", 13, "PHP keyword", ""),
    ("enum", 13, "PHP keyword", "PHP 8.1 enum."),
    ("eval", 13, "PHP keyword", ""),
    ("exit", 13, "PHP keyword", "Exits script."),
    ("extends", 13, "PHP keyword", "Class inheritance."),
    ("final", 13, "PHP keyword", "Prevents class extension or method overriding."),
    ("finally", 13, "PHP keyword", ""),
    ("fn", 13, "PHP keyword", "Short arrow function (PHP 7.4)."),
    ("for", 13, "PHP keyword", "Loop."),
    ("foreach", 13, "PHP keyword", "Iterates array/iterable."),
    ("function", 13, "PHP keyword", "Defines a function."),
    ("global", 13, "PHP keyword", ""),
    ("goto", 13, "PHP keyword", ""),
    ("if", 13, "PHP keyword", "Conditional."),
    ("implements", 13, "PHP keyword", "Interface implementation."),
    ("include", 13, "PHP keyword", "Includes a file."),
    ("include_once", 13, "PHP keyword", ""),
    ("instanceof", 13, "PHP keyword", ""),
    ("insteadof", 13, "PHP keyword", ""),
    ("interface", 13, "PHP keyword", "Defines an interface."),
    ("isset", 13, "PHP keyword", "Checks if variable is set."),
    ("list", 13, "PHP keyword", "Assigns variables from array."),
    ("match", 13, "PHP keyword", "Match expression (PHP 8)."),
    ("namespace", 13, "PHP keyword", "Declares a namespace."),
    ("new", 13, "PHP keyword", "Creates instance."),
    ("null", 13, "PHP keyword", "Null value."),
    ("or", 13, "PHP keyword", ""),
    ("print", 13, "PHP keyword", "Outputs a string."),
    ("private", 13, "PHP keyword", "Private access."),
    ("protected", 13, "PHP keyword", "Protected access."),
    ("public", 13, "PHP keyword", "Public access."),
    ("readonly", 13, "PHP keyword", ""),
    ("require", 13, "PHP keyword", "Requires a file."),
    ("require_once", 13, "PHP keyword", ""),
    ("return", 13, "PHP keyword", "Returns value."),
    ("static", 13, "PHP keyword", "Static member."),
    ("switch", 13, "PHP keyword", ""),
    ("throw", 13, "PHP keyword", "Throws exception."),
    ("trait", 13, "PHP keyword", "Defines a trait."),
    ("true", 13, "PHP keyword", "Boolean true."),
    ("false", 13, "PHP keyword", "Boolean false."),
    ("try", 13, "PHP keyword", "Exception handling."),
    ("unset", 13, "PHP keyword", "Destroys variables."),
    ("use", 13, "PHP keyword", "Imports a namespace, class, or trait."),
    ("var", 13, "PHP keyword", ""),
    ("while", 13, "PHP keyword", "Loop."),
    ("xor", 13, "PHP keyword", ""),
    ("yield", 13, "PHP keyword", "Generator yield."),
    ("array_map", 5, "built-in", "Applies callback to array elements."),
    ("array_filter", 5, "built-in", "Filters array elements by callback."),
    ("array_reduce", 5, "built-in", ""),
    ("array_keys", 5, "built-in", ""),
    ("array_values", 5, "built-in", ""),
    ("array_merge", 5, "built-in", ""),
    ("array_push", 5, "built-in", ""),
    ("array_pop", 5, "built-in", ""),
    ("array_shift", 5, "built-in", ""),
    ("array_unshift", 5, "built-in", ""),
    ("array_slice", 5, "built-in", ""),
    ("array_splice", 5, "built-in", ""),
    ("array_search", 5, "built-in", ""),
    ("array_unique", 5, "built-in", ""),
    ("array_flip", 5, "built-in", ""),
    ("array_reverse", 5, "built-in", ""),
    ("array_sort", 5, "built-in", ""),
    ("usort", 5, "built-in", ""),
    ("count", 5, "built-in", "Returns element count."),
    ("in_array", 5, "built-in", "Checks if value exists in array."),
    ("implode", 5, "built-in", "Joins array elements with delimiter."),
    ("explode", 5, "built-in", "Splits string by delimiter."),
    ("str_replace", 5, "built-in", "Replaces occurrences in string."),
    ("str_split", 5, "built-in", ""),
    ("str_pad", 5, "built-in", ""),
    ("str_repeat", 5, "built-in", ""),
    ("str_contains", 5, "built-in", ""),
    ("str_starts_with", 5, "built-in", ""),
    ("str_ends_with", 5, "built-in", ""),
    ("strlen", 5, "built-in", "Returns string length."),
    ("strtolower", 5, "built-in", ""),
    ("strtoupper", 5, "built-in", ""),
    ("ucfirst", 5, "built-in", ""),
    ("lcfirst", 5, "built-in", ""),
    ("trim", 5, "built-in", ""),
    ("ltrim", 5, "built-in", ""),
    ("rtrim", 5, "built-in", ""),
    ("substr", 5, "built-in", ""),
    ("strpos", 5, "built-in", ""),
    ("strrpos", 5, "built-in", ""),
    ("sprintf", 5, "built-in", "Formats string."),
    ("printf", 5, "built-in", ""),
    ("number_format", 5, "built-in", ""),
    ("intval", 5, "built-in", ""),
    ("floatval", 5, "built-in", ""),
    ("strval", 5, "built-in", ""),
    ("is_array", 5, "built-in", ""),
    ("is_string", 5, "built-in", ""),
    ("is_int", 5, "built-in", ""),
    ("is_float", 5, "built-in", ""),
    ("is_bool", 5, "built-in", ""),
    ("is_null", 5, "built-in", ""),
    ("is_object", 5, "built-in", ""),
    ("is_numeric", 5, "built-in", ""),
    ("var_dump", 5, "built-in", "Dumps variable info."),
    ("print_r", 5, "built-in", ""),
    ("json_encode", 5, "built-in", "Encodes to JSON."),
    ("json_decode", 5, "built-in", "Decodes JSON string."),
    ("file_get_contents", 5, "built-in", "Reads file into string."),
    ("file_put_contents", 5, "built-in", ""),
    ("date", 5, "built-in", "Formats a date."),
    ("time", 5, "built-in", ""),
    ("mktime", 5, "built-in", ""),
    ("strtotime", 5, "built-in", ""),
    ("preg_match", 5, "built-in", "Performs regex match."),
    ("preg_replace", 5, "built-in", ""),
    ("preg_split", 5, "built-in", ""),
    ("header", 5, "built-in", ""),
    ("htmlspecialchars", 5, "built-in", "Converts special chars to HTML entities."),
    ("htmlspecialchars_decode", 5, "built-in", ""),
    ("strip_tags", 5, "built-in", ""),
    ("md5", 5, "built-in", ""),
    ("sha1", 5, "built-in", ""),
    ("hash", 5, "built-in", ""),
    ("base64_encode", 5, "built-in", ""),
    ("base64_decode", 5, "built-in", ""),
    ("urlencode", 5, "built-in", ""),
    ("urldecode", 5, "built-in", ""),
    ("rand", 5, "built-in", ""),
    ("mt_rand", 5, "built-in", ""),
    ("round", 5, "built-in", ""),
    ("ceil", 5, "built-in", ""),
    ("floor", 5, "built-in", ""),
    ("abs", 5, "built-in", ""),
    ("max", 5, "built-in", ""),
    ("min", 5, "built-in", ""),
    ("sqrt", 5, "built-in", ""),
    ("pow", 5, "built-in", ""),
    ("class_exists", 5, "built-in", ""),
    ("method_exists", 5, "built-in", ""),
    ("property_exists", 5, "built-in", ""),
    ("get_class", 5, "built-in", ""),
    ("is_a", 5, "built-in", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "php".to_string()
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
