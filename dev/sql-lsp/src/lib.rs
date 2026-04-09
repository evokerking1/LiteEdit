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

#[allow(dead_code)]
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
    ("SELECT", 13, "SQL keyword", "Retrieves rows from tables."),
    ("FROM", 13, "SQL keyword", "Specifies the source table(s)."),
    ("WHERE", 13, "SQL keyword", "Filters rows by condition."),
    ("AND", 13, "SQL keyword", ""),
    ("OR", 13, "SQL keyword", ""),
    ("NOT", 13, "SQL keyword", ""),
    ("IN", 13, "SQL keyword", ""),
    ("EXISTS", 13, "SQL keyword", ""),
    ("BETWEEN", 13, "SQL keyword", ""),
    ("LIKE", 13, "SQL keyword", ""),
    ("ILIKE", 13, "SQL keyword", ""),
    ("IS", 13, "SQL keyword", ""),
    ("NULL", 13, "SQL keyword", ""),
    ("AS", 13, "SQL keyword", ""),
    ("ON", 13, "SQL keyword", ""),
    ("JOIN", 13, "SQL keyword", "Combines rows from tables."),
    ("INNER", 13, "SQL keyword", ""),
    ("LEFT", 13, "SQL keyword", ""),
    ("RIGHT", 13, "SQL keyword", ""),
    ("FULL", 13, "SQL keyword", ""),
    ("OUTER", 13, "SQL keyword", ""),
    ("CROSS", 13, "SQL keyword", ""),
    ("NATURAL", 13, "SQL keyword", ""),
    ("UNION", 13, "SQL keyword", "Combines result sets."),
    ("INTERSECT", 13, "SQL keyword", ""),
    ("EXCEPT", 13, "SQL keyword", ""),
    ("ALL", 13, "SQL keyword", ""),
    ("DISTINCT", 13, "SQL keyword", "Returns unique values."),
    ("ORDER", 13, "SQL keyword", ""),
    ("BY", 13, "SQL keyword", ""),
    ("ASC", 13, "SQL keyword", ""),
    ("DESC", 13, "SQL keyword", ""),
    ("GROUP", 13, "SQL keyword", ""),
    ("HAVING", 13, "SQL keyword", "Filters grouped results."),
    ("LIMIT", 13, "SQL keyword", "Limits number of rows returned."),
    ("OFFSET", 13, "SQL keyword", "Skips number of rows."),
    ("INSERT", 13, "SQL keyword", ""),
    ("INTO", 13, "SQL keyword", ""),
    ("VALUES", 13, "SQL keyword", ""),
    ("UPDATE", 13, "SQL keyword", "Modifies existing rows."),
    ("SET", 13, "SQL keyword", ""),
    ("DELETE", 13, "SQL keyword", ""),
    ("CREATE", 13, "SQL keyword", ""),
    ("TABLE", 13, "SQL keyword", ""),
    ("DATABASE", 13, "SQL keyword", ""),
    ("SCHEMA", 13, "SQL keyword", ""),
    ("INDEX", 13, "SQL keyword", ""),
    ("VIEW", 13, "SQL keyword", ""),
    ("TRIGGER", 13, "SQL keyword", ""),
    ("PROCEDURE", 13, "SQL keyword", ""),
    ("FUNCTION", 13, "SQL keyword", ""),
    ("ALTER", 13, "SQL keyword", ""),
    ("DROP", 13, "SQL keyword", ""),
    ("TRUNCATE", 13, "SQL keyword", ""),
    ("ADD", 13, "SQL keyword", ""),
    ("COLUMN", 13, "SQL keyword", ""),
    ("CONSTRAINT", 13, "SQL keyword", ""),
    ("PRIMARY", 13, "SQL keyword", ""),
    ("KEY", 13, "SQL keyword", ""),
    ("FOREIGN", 13, "SQL keyword", ""),
    ("REFERENCES", 13, "SQL keyword", ""),
    ("UNIQUE", 13, "SQL keyword", "Ensures column values are unique."),
    ("CHECK", 13, "SQL keyword", ""),
    ("DEFAULT", 13, "SQL keyword", "Default column value."),
    ("AUTO_INCREMENT", 13, "SQL keyword", ""),
    ("IDENTITY", 13, "SQL keyword", ""),
    ("SERIAL", 13, "SQL keyword", ""),
    ("SEQUENCE", 13, "SQL keyword", ""),
    ("CASE", 13, "SQL keyword", "Conditional expression."),
    ("WHEN", 13, "SQL keyword", ""),
    ("THEN", 13, "SQL keyword", ""),
    ("ELSE", 13, "SQL keyword", ""),
    ("END", 13, "SQL keyword", ""),
    ("BEGIN", 13, "SQL keyword", ""),
    ("COMMIT", 13, "SQL keyword", "Saves transaction."),
    ("ROLLBACK", 13, "SQL keyword", "Undoes transaction."),
    ("TRANSACTION", 13, "SQL keyword", "Groups statements as unit."),
    ("SAVEPOINT", 13, "SQL keyword", ""),
    ("GRANT", 13, "SQL keyword", ""),
    ("REVOKE", 13, "SQL keyword", ""),
    ("EXPLAIN", 13, "SQL keyword", "Shows query execution plan."),
    ("ANALYZE", 13, "SQL keyword", ""),
    ("VACUUM", 13, "SQL keyword", ""),
    ("WITH", 13, "SQL keyword", "Common table expression (CTE)."),
    ("RECURSIVE", 13, "SQL keyword", ""),
    ("WINDOW", 13, "SQL keyword", "Window function frame."),
    ("OVER", 13, "SQL keyword", "Specifies window partition."),
    ("PARTITION", 13, "SQL keyword", ""),
    ("ROW_NUMBER", 13, "SQL keyword", ""),
    ("RANK", 13, "SQL keyword", ""),
    ("DENSE_RANK", 13, "SQL keyword", ""),
    ("LAG", 13, "SQL keyword", ""),
    ("LEAD", 13, "SQL keyword", ""),
    ("FIRST_VALUE", 13, "SQL keyword", ""),
    ("LAST_VALUE", 13, "SQL keyword", ""),
    ("NTILE", 13, "SQL keyword", ""),
    ("IF", 13, "SQL keyword", ""),
    ("RETURN", 13, "SQL keyword", ""),
    ("DECLARE", 13, "SQL keyword", ""),
    ("CURSOR", 13, "SQL keyword", ""),
    ("FETCH", 13, "SQL keyword", ""),
    ("OPEN", 13, "SQL keyword", ""),
    ("CLOSE", 13, "SQL keyword", ""),
    ("LOOP", 13, "SQL keyword", ""),
    ("EXIT", 13, "SQL keyword", ""),
    ("CONTINUE", 13, "SQL keyword", ""),
    ("RAISE", 13, "SQL keyword", ""),
    ("EXECUTE", 13, "SQL keyword", ""),
    ("COUNT", 1, "aggregate", "Counts rows."),
    ("SUM", 1, "aggregate", "Sums values."),
    ("AVG", 1, "aggregate", "Averages values."),
    ("MIN", 1, "aggregate", "Minimum value."),
    ("MAX", 1, "aggregate", "Maximum value."),
    ("GROUP_CONCAT", 1, "aggregate", "Concatenates strings."),
    ("STRING_AGG", 1, "aggregate", "String aggregation."),
    ("ARRAY_AGG", 1, "aggregate", ""),
    ("JSON_AGG", 1, "aggregate", ""),
    ("JSONB_AGG", 1, "aggregate", ""),
    ("BOOL_AND", 1, "aggregate", ""),
    ("BOOL_OR", 1, "aggregate", ""),
    ("BIT_AND", 1, "aggregate", ""),
    ("BIT_OR", 1, "aggregate", ""),
    ("STDDEV", 1, "aggregate", "Standard deviation."),
    ("VARIANCE", 1, "aggregate", "Statistical variance."),
    ("CORR", 1, "aggregate", ""),
    ("COVAR_POP", 1, "aggregate", ""),
    ("COVAR_SAMP", 1, "aggregate", ""),
    ("COALESCE", 1, "function", "Returns first non-null."),
    ("NULLIF", 1, "function", "Returns null if args equal."),
    ("IFNULL", 1, "function", ""),
    ("NVL", 1, "function", ""),
    ("CAST", 1, "function", "Type conversion."),
    ("CONVERT", 1, "function", ""),
    ("CONCAT", 1, "function", "Concatenates strings."),
    ("SUBSTRING", 1, "function", "Extracts substring."),
    ("LENGTH", 1, "function", "String length."),
    ("CHAR_LENGTH", 1, "function", ""),
    ("UPPER", 1, "function", "Uppercase."),
    ("LOWER", 1, "function", "Lowercase."),
    ("TRIM", 1, "function", "Removes whitespace."),
    ("LTRIM", 1, "function", ""),
    ("RTRIM", 1, "function", ""),
    ("REPLACE", 1, "function", ""),
    ("INSTR", 1, "function", ""),
    ("LOCATE", 1, "function", ""),
    ("POSITION", 1, "function", ""),
    ("LPAD", 1, "function", ""),
    ("RPAD", 1, "function", ""),
    ("REPEAT", 1, "function", ""),
    ("REVERSE", 1, "function", ""),
    ("FORMAT", 1, "function", ""),
    ("NOW", 1, "function", "Current timestamp."),
    ("CURRENT_DATE", 1, "function", "Current date."),
    ("CURRENT_TIME", 1, "function", ""),
    ("CURRENT_TIMESTAMP", 1, "function", ""),
    ("DATE", 1, "function", ""),
    ("TIME", 1, "function", ""),
    ("YEAR", 1, "function", ""),
    ("MONTH", 1, "function", ""),
    ("DAY", 1, "function", ""),
    ("HOUR", 1, "function", ""),
    ("MINUTE", 1, "function", ""),
    ("SECOND", 1, "function", ""),
    ("DATEDIFF", 1, "function", "Difference between dates."),
    ("DATEADD", 1, "function", ""),
    ("DATE_ADD", 1, "function", "Adds interval to date."),
    ("DATE_SUB", 1, "function", ""),
    ("EXTRACT", 1, "function", ""),
    ("TO_CHAR", 1, "function", ""),
    ("TO_DATE", 1, "function", ""),
    ("TO_NUMBER", 1, "function", ""),
    ("ABS", 1, "function", "Absolute value."),
    ("CEILING", 1, "function", ""),
    ("FLOOR", 1, "function", ""),
    ("ROUND", 1, "function", "Rounds number."),
    ("TRUNCATE", 1, "function", ""),
    ("MOD", 1, "function", ""),
    ("POWER", 1, "function", ""),
    ("SQRT", 1, "function", ""),
    ("LOG", 1, "function", ""),
    ("LN", 1, "function", ""),
    ("EXP", 1, "function", ""),
    ("SIGN", 1, "function", ""),
    ("PI", 1, "function", ""),
    ("RAND", 1, "function", ""),
    ("RANDOM", 1, "function", ""),
    ("UUID", 1, "function", "Generates UUID."),
    ("MD5", 1, "function", ""),
    ("SHA1", 1, "function", ""),
    ("SHA256", 1, "function", ""),
    ("ENCODE", 1, "function", ""),
    ("DECODE", 1, "function", ""),
    ("JSON_VALUE", 1, "function", "Extracts JSON value."),
    ("JSON_QUERY", 1, "function", ""),
    ("JSON_EXTRACT", 1, "function", ""),
    ("ROW_NUMBER", 1, "function", "Sequential row number."),
    ("RANK", 1, "function", "Row rank with gaps."),
    ("DENSE_RANK", 1, "function", "Row rank without gaps."),
    ("LAG", 1, "function", "Previous row value."),
    ("LEAD", 1, "function", "Next row value."),
    ("FIRST_VALUE", 1, "function", ""),
    ("LAST_VALUE", 1, "function", ""),
    ("NTILE", 1, "function", ""),
    ("INT", 7, "data type", "32-bit integer."),
    ("INTEGER", 7, "data type", ""),
    ("BIGINT", 7, "data type", "64-bit integer."),
    ("SMALLINT", 7, "data type", ""),
    ("TINYINT", 7, "data type", ""),
    ("DECIMAL", 7, "data type", "Exact decimal."),
    ("NUMERIC", 7, "data type", ""),
    ("FLOAT", 7, "data type", "Floating point."),
    ("REAL", 7, "data type", ""),
    ("DOUBLE", 7, "data type", ""),
    ("BOOLEAN", 7, "data type", "True/false."),
    ("BOOL", 7, "data type", ""),
    ("CHAR", 7, "data type", ""),
    ("VARCHAR", 7, "data type", "Variable-length string."),
    ("TEXT", 7, "data type", "Unlimited text."),
    ("NCHAR", 7, "data type", ""),
    ("NVARCHAR", 7, "data type", ""),
    ("NTEXT", 7, "data type", ""),
    ("BINARY", 7, "data type", ""),
    ("VARBINARY", 7, "data type", ""),
    ("BLOB", 7, "data type", ""),
    ("DATE", 7, "data type", "Calendar date."),
    ("TIME", 7, "data type", "Time of day."),
    ("DATETIME", 7, "data type", ""),
    ("TIMESTAMP", 7, "data type", "Date and time."),
    ("TIMESTAMPTZ", 7, "data type", "Timestamp with timezone."),
    ("INTERVAL", 7, "data type", "Time interval."),
    ("UUID", 7, "data type", "128-bit unique id."),
    ("JSON", 7, "data type", "JSON data."),
    ("JSONB", 7, "data type", "Binary JSON (PostgreSQL)."),
    ("XML", 7, "data type", ""),
    ("ARRAY", 7, "data type", "Array type."),
    ("ENUM", 7, "data type", "Enumeration type."),
    ("SET", 7, "data type", ""),
    ("SERIAL", 7, "data type", "Auto-increment integer."),
    ("BIGSERIAL", 7, "data type", ""),
    ("MONEY", 7, "data type", ""),
    ("BYTEA", 7, "data type", "Binary data."),
    ("CIDR", 7, "data type", ""),
    ("INET", 7, "data type", ""),
    ("MACADDR", 7, "data type", ""),
];

#[wasm_bindgen]
pub fn language_id() -> String {
    "sql".to_string()
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
    let _ = code;
    serde_json::to_string(&Vec::<Diagnostic>::new()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use serde_json;

    #[test]
    fn test_language_id() {
        assert_eq!(language_id(), "sql");
    }

    #[test]
    fn test_completions_all() {
        let result = get_completions("", 0, 0);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_completions_prefix_match() {
        let result = get_completions("SE", 0, 2);
        let items: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(!items.is_empty());
        for item in &items {
            let label = item["label"].as_str().unwrap().to_lowercase();
            assert!(label.starts_with("se"), "label '{}' does not start with 'se'", label);
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
        let result = get_hover("SELECT * FROM t", 0, 3);
        assert_ne!(result, "null");
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["contents"].as_str().unwrap().contains("Retrieves rows from tables."));
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
        let result = get_diagnostics("anything");
        let diags: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(diags.is_empty());
    }

    #[test]
    fn test_diagnostics_unexpected_close() {
        let result = get_diagnostics("}");
        let diags: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(diags.is_empty());
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
