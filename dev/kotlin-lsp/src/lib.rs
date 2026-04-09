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
    ("as", 13, "Kotlin keyword", "Type cast."),
    ("as?", 13, "Kotlin keyword", ""),
    ("break", 13, "Kotlin keyword", ""),
    ("class", 13, "Kotlin keyword", "Declares a class."),
    ("continue", 13, "Kotlin keyword", ""),
    ("do", 13, "Kotlin keyword", "Do-while loop."),
    ("else", 13, "Kotlin keyword", "Else branch."),
    ("false", 13, "Kotlin keyword", "Boolean false."),
    ("for", 13, "Kotlin keyword", "Loop over iterable."),
    ("fun", 13, "Kotlin keyword", "Defines a function."),
    ("if", 13, "Kotlin keyword", "Conditional (also expression)."),
    ("in", 13, "Kotlin keyword", "Membership/iteration."),
    ("!in", 13, "Kotlin keyword", ""),
    ("interface", 13, "Kotlin keyword", "Declares an interface."),
    ("is", 13, "Kotlin keyword", "Type check."),
    ("!is", 13, "Kotlin keyword", ""),
    ("null", 13, "Kotlin keyword", "Null literal."),
    ("object", 13, "Kotlin keyword", "Singleton object."),
    ("package", 13, "Kotlin keyword", "Package declaration."),
    ("return", 13, "Kotlin keyword", "Returns from function."),
    ("super", 13, "Kotlin keyword", "Parent class."),
    ("this", 13, "Kotlin keyword", "Current instance."),
    ("throw", 13, "Kotlin keyword", "Throws exception."),
    ("true", 13, "Kotlin keyword", "Boolean true."),
    ("try", 13, "Kotlin keyword", "Exception handling."),
    ("typealias", 13, "Kotlin keyword", "Type alias."),
    ("val", 13, "Kotlin keyword", "Read-only variable."),
    ("var", 13, "Kotlin keyword", "Mutable variable."),
    ("when", 13, "Kotlin keyword", "Replacement for switch, also expression."),
    ("while", 13, "Kotlin keyword", "While loop."),
    ("by", 13, "Kotlin keyword", "Delegation."),
    ("catch", 13, "Kotlin keyword", "Catches exception."),
    ("constructor", 13, "Kotlin keyword", "Secondary constructor."),
    ("delegate", 13, "Kotlin keyword", "Delegation keyword."),
    ("dynamic", 13, "Kotlin keyword", "Dynamic type (JS target)."),
    ("field", 13, "Kotlin keyword", "Backing field reference."),
    ("file", 13, "Kotlin keyword", ""),
    ("finally", 13, "Kotlin keyword", "Always executes."),
    ("get", 13, "Kotlin keyword", "Property getter."),
    ("import", 13, "Kotlin keyword", "Imports a class or function."),
    ("init", 13, "Kotlin keyword", "Initializer block."),
    ("param", 13, "Kotlin keyword", ""),
    ("property", 13, "Kotlin keyword", ""),
    ("receiver", 13, "Kotlin keyword", ""),
    ("set", 13, "Kotlin keyword", "Property setter."),
    ("setparam", 13, "Kotlin keyword", ""),
    ("value", 13, "Kotlin keyword", ""),
    ("where", 13, "Kotlin keyword", ""),
    ("actual", 13, "Kotlin keyword", "Multiplatform actual."),
    ("abstract", 13, "Kotlin keyword", "Abstract class or member."),
    ("annotation", 13, "Kotlin keyword", ""),
    ("companion", 13, "Kotlin keyword", "Companion object inside class."),
    ("crossinline", 13, "Kotlin keyword", "No non-local returns."),
    ("data", 13, "Kotlin keyword", "Data class (auto equals/hashCode/toString/copy)."),
    ("enum", 13, "Kotlin keyword", "Enumeration."),
    ("expect", 13, "Kotlin keyword", "Multiplatform expect."),
    ("external", 13, "Kotlin keyword", "External implementation."),
    ("final", 13, "Kotlin keyword", ""),
    ("infix", 13, "Kotlin keyword", "Infix notation function."),
    ("inline", 13, "Kotlin keyword", "Inlines function body."),
    ("inner", 13, "Kotlin keyword", ""),
    ("internal", 13, "Kotlin keyword", "Module visibility."),
    ("lateinit", 13, "Kotlin keyword", "Late-initialized var."),
    ("noinline", 13, "Kotlin keyword", "Don't inline lambda."),
    ("open", 13, "Kotlin keyword", "Allows class/member to be overridden."),
    ("operator", 13, "Kotlin keyword", "Operator overloading."),
    ("out", 13, "Kotlin keyword", ""),
    ("override", 13, "Kotlin keyword", "Overrides a member."),
    ("private", 13, "Kotlin keyword", "Class-only visibility."),
    ("protected", 13, "Kotlin keyword", "Subclass visibility."),
    ("public", 13, "Kotlin keyword", "Full visibility (default)."),
    ("reified", 13, "Kotlin keyword", "Reified type parameter."),
    ("sealed", 13, "Kotlin keyword", "Sealed class hierarchy."),
    ("suspend", 13, "Kotlin keyword", "Suspending function."),
    ("tailrec", 13, "Kotlin keyword", "Tail recursive optimization."),
    ("vararg", 13, "Kotlin keyword", "Variable number of args."),
    ("String", 6, "stdlib", "UTF-16 string."),
    ("Int", 6, "stdlib", "32-bit integer."),
    ("Long", 6, "stdlib", "64-bit integer."),
    ("Double", 6, "stdlib", "64-bit float."),
    ("Float", 6, "stdlib", ""),
    ("Boolean", 6, "stdlib", "Boolean type."),
    ("Char", 6, "stdlib", ""),
    ("Byte", 6, "stdlib", ""),
    ("Short", 6, "stdlib", ""),
    ("Unit", 6, "stdlib", "No return value."),
    ("Nothing", 6, "stdlib", "Never returns."),
    ("Any", 6, "stdlib", "Root of class hierarchy."),
    ("Array", 6, "stdlib", ""),
    ("IntArray", 6, "stdlib", ""),
    ("LongArray", 6, "stdlib", ""),
    ("DoubleArray", 6, "stdlib", ""),
    ("FloatArray", 6, "stdlib", ""),
    ("BooleanArray", 6, "stdlib", ""),
    ("CharArray", 6, "stdlib", ""),
    ("ByteArray", 6, "stdlib", ""),
    ("ShortArray", 6, "stdlib", ""),
    ("List", 6, "stdlib", "Read-only list."),
    ("MutableList", 6, "stdlib", "Mutable list."),
    ("Set", 6, "stdlib", "Read-only set."),
    ("MutableSet", 6, "stdlib", "Mutable set."),
    ("Map", 6, "stdlib", "Read-only map."),
    ("MutableMap", 6, "stdlib", "Mutable map."),
    ("Collection", 6, "stdlib", ""),
    ("MutableCollection", 6, "stdlib", ""),
    ("Iterable", 6, "stdlib", ""),
    ("MutableIterable", 6, "stdlib", ""),
    ("Sequence", 6, "stdlib", "Lazy collection."),
    ("Pair", 6, "stdlib", "Two-element tuple."),
    ("Triple", 6, "stdlib", "Three-element tuple."),
    ("Result", 6, "stdlib", "Success or failure."),
    ("Lazy", 6, "stdlib", "Lazy initialization."),
    ("Regex", 6, "stdlib", "Regular expression."),
    ("StringBuilder", 6, "stdlib", "Mutable string builder."),
    ("Exception", 6, "stdlib", ""),
    ("Throwable", 6, "stdlib", ""),
    ("Error", 6, "stdlib", ""),
    ("IllegalArgumentException", 6, "stdlib", ""),
    ("IllegalStateException", 6, "stdlib", ""),
    ("NullPointerException", 6, "stdlib", ""),
    ("IndexOutOfBoundsException", 6, "stdlib", ""),
    ("UnsupportedOperationException", 6, "stdlib", ""),
    ("NumberFormatException", 6, "stdlib", ""),
    ("ArithmeticException", 6, "stdlib", ""),
    ("println", 5, "built-in", "Prints with newline."),
    ("print", 5, "built-in", "Prints without newline."),
    ("readLine", 5, "built-in", "Reads a line from stdin."),
    ("arrayOf", 5, "built-in", "Creates array."),
    ("listOf", 5, "built-in", "Creates read-only list."),
    ("mutableListOf", 5, "built-in", "Creates mutable list."),
    ("setOf", 5, "built-in", "Creates read-only set."),
    ("mutableSetOf", 5, "built-in", ""),
    ("mapOf", 5, "built-in", "Creates read-only map."),
    ("mutableMapOf", 5, "built-in", ""),
    ("emptyList", 5, "built-in", ""),
    ("emptyMap", 5, "built-in", ""),
    ("emptySet", 5, "built-in", ""),
    ("sequenceOf", 5, "built-in", ""),
    ("generateSequence", 5, "built-in", ""),
    ("run", 5, "built-in", "Executes block and returns result."),
    ("let", 5, "built-in", "Calls block with receiver, returns result."),
    ("also", 5, "built-in", "Like let, returns original."),
    ("apply", 5, "built-in", "Calls block with this, returns this."),
    ("with", 5, "built-in", "Calls block on receiver."),
    ("repeat", 5, "built-in", "Repeats block n times."),
    ("check", 5, "built-in", "Throws IllegalStateException if false."),
    ("require", 5, "built-in", "Throws IllegalArgumentException if false."),
    ("error", 5, "built-in", ""),
    ("TODO", 5, "built-in", "Throws NotImplementedError."),
    ("assert", 5, "built-in", ""),
    ("checkNotNull", 5, "built-in", ""),
    ("requireNotNull", 5, "built-in", ""),
    ("lazy", 5, "built-in", "Creates lazy delegate."),
    ("by", 5, "built-in", ""),
    ("takeIf", 5, "built-in", "Returns receiver if predicate true, else null."),
    ("takeUnless", 5, "built-in", "Returns receiver if predicate false, else null."),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "kotlin".to_string()
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
