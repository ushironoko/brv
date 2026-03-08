/// プレースホルダー処理結果
pub struct PlaceholderResult {
    /// プレースホルダー除去後のテキスト
    pub text: String,
    /// カーソル位置 (最初のプレースホルダーの開始位置)
    pub cursor: usize,
}

/// 最初の {{...}} プレースホルダーを除去し、そこにカーソルを移動
pub fn apply_first_placeholder(text: &str, default_cursor: usize) -> PlaceholderResult {
    if let Some(start) = text.find("{{") {
        if let Some(end) = text[start..].find("}}") {
            let end_pos = start + end + 2;
            let mut result = String::with_capacity(text.len() - (end_pos - start));
            result.push_str(&text[..start]);
            result.push_str(&text[end_pos..]);
            return PlaceholderResult {
                text: result,
                cursor: start,
            };
        }
    }

    PlaceholderResult {
        text: text.to_string(),
        cursor: default_cursor,
    }
}

/// 次のプレースホルダーを検索 (カーソル位置以降)
pub fn find_next_placeholder(text: &str, cursor: usize) -> Option<(usize, usize)> {
    let search_start = cursor.min(text.len());
    if let Some(offset) = text[search_start..].find("{{") {
        let start = search_start + offset;
        if let Some(end_offset) = text[start..].find("}}") {
            return Some((start, start + end_offset + 2));
        }
    }

    // カーソル以降になければ先頭から検索 (ラップ)
    if cursor > 0 {
        if let Some(start) = text.find("{{") {
            if start < cursor {
                if let Some(end_offset) = text[start..].find("}}") {
                    return Some((start, start + end_offset + 2));
                }
            }
        }
    }

    None
}

/// テキスト中のすべてのプレースホルダーを除去
pub fn remove_all_placeholders(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = remaining.find("{{") {
        result.push_str(&remaining[..start]);
        if let Some(end) = remaining[start..].find("}}") {
            remaining = &remaining[start + end + 2..];
        } else {
            // 閉じタグがない場合は {{ 以降をそのまま追加
            result.push_str(&remaining[start..]);
            return result;
        }
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_first_placeholder_basic() {
        let result = apply_first_placeholder("git commit -m '{{message}}'", 100);
        assert_eq!(result.text, "git commit -m ''");
        assert_eq!(result.cursor, 15);
    }

    #[test]
    fn test_apply_first_placeholder_at_start() {
        let result = apply_first_placeholder("{{cmd}} arg1 arg2", 100);
        assert_eq!(result.text, " arg1 arg2");
        assert_eq!(result.cursor, 0);
    }

    #[test]
    fn test_apply_first_placeholder_at_end() {
        let result = apply_first_placeholder("echo {{value}}", 100);
        assert_eq!(result.text, "echo ");
        assert_eq!(result.cursor, 5);
    }

    #[test]
    fn test_apply_first_placeholder_multiple() {
        let result = apply_first_placeholder("{{a}} and {{b}}", 100);
        assert_eq!(result.text, " and {{b}}");
        assert_eq!(result.cursor, 0);
    }

    #[test]
    fn test_apply_first_placeholder_none() {
        let result = apply_first_placeholder("no placeholder here", 10);
        assert_eq!(result.text, "no placeholder here");
        assert_eq!(result.cursor, 10);
    }

    #[test]
    fn test_apply_first_placeholder_unclosed() {
        let result = apply_first_placeholder("text {{unclosed", 5);
        assert_eq!(result.text, "text {{unclosed");
        assert_eq!(result.cursor, 5);
    }

    #[test]
    fn test_find_next_placeholder() {
        let text = "{{a}} and {{b}} and {{c}}";
        let result = find_next_placeholder(text, 0);
        assert_eq!(result, Some((0, 5)));

        let result = find_next_placeholder(text, 5);
        assert_eq!(result, Some((10, 15)));

        let result = find_next_placeholder(text, 15);
        assert_eq!(result, Some((20, 25)));

        // ラップ
        let result = find_next_placeholder(text, 25);
        assert_eq!(result, Some((0, 5)));
    }

    #[test]
    fn test_find_next_placeholder_none() {
        let result = find_next_placeholder("no placeholder", 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_remove_all_placeholders() {
        assert_eq!(
            remove_all_placeholders("git commit -m '{{message}}' --author='{{author}}'"),
            "git commit -m '' --author=''"
        );
        assert_eq!(remove_all_placeholders("no placeholders"), "no placeholders");
        assert_eq!(remove_all_placeholders("{{only}}"), "");
    }

    #[test]
    fn test_remove_all_placeholders_unclosed() {
        // "text {{valid}} text {{unclosed" → "text " 部分は valid を除去、残りはそのまま
        assert_eq!(
            remove_all_placeholders("{{valid}} text {{unclosed"),
            " text {{unclosed"
        );
        // 閉じタグなしの単純なケース
        assert_eq!(
            remove_all_placeholders("only {{unclosed"),
            "only {{unclosed"
        );
    }

    #[test]
    fn test_placeholder_cursor_within_bounds() {
        // プレースホルダーありの場合、カーソルはテキスト長以内
        let texts = vec![
            "{{a}}",
            "pre {{a}} post",
            "{{a}} {{b}}",
            "text",
            "",
        ];
        for text in texts {
            let result = apply_first_placeholder(text, text.len());
            assert!(
                result.cursor <= result.text.len(),
                "cursor {} > text.len() {} for input {:?}",
                result.cursor,
                result.text.len(),
                text
            );
        }
    }
}
