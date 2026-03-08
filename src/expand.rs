use crate::context;
use crate::matcher::{self, CompiledAbbr, Matcher};
use crate::output::ExpandOutput;
use crate::placeholder;

/// 展開入力
pub struct ExpandInput {
    pub lbuffer: String,
    pub rbuffer: String,
}

/// lbuffer からキーワードを抽出
/// lbuffer の末尾のトークン(スペースで区切られた最後の単語)をキーワードとして返す
fn extract_keyword(lbuffer: &str) -> Option<(&str, &str)> {
    let trimmed = lbuffer.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    // 末尾のトークンを取得
    if let Some(space_pos) = trimmed.rfind(' ') {
        let keyword = &trimmed[space_pos + 1..];
        let prefix = &trimmed[..space_pos + 1];
        if keyword.is_empty() {
            None
        } else {
            Some((prefix, keyword))
        }
    } else {
        // スペースなし = lbuffer 全体がキーワード
        Some(("", trimmed))
    }
}

/// コマンド位置かどうかを判定
/// lbuffer にスペースが含まれていない場合はコマンド位置
fn is_command_position(prefix: &str) -> bool {
    prefix.trim().is_empty()
}

/// 展開を実行
pub fn expand(input: &ExpandInput, matcher_data: &Matcher) -> ExpandOutput {
    let Some((prefix, keyword)) = extract_keyword(&input.lbuffer) else {
        return ExpandOutput::NoMatch;
    };

    // 1. コンテキスト付き abbreviation を最優先で検索
    // lbuffer からキーワードを除いた部分をコンテキストとして使う
    if let Some(abbr) =
        context::find_contextual_match(&matcher_data.contextual, keyword, prefix, &input.rbuffer)
    {
        return build_output(prefix, abbr, &input.rbuffer);
    }

    // 2. コマンド位置の場合、regular abbreviation を検索
    if is_command_position(prefix) {
        if let Some(abbr) = matcher::lookup_regular(matcher_data, keyword) {
            return build_output(prefix, abbr, &input.rbuffer);
        }
    }

    // 3. グローバル abbreviation を検索 (位置を問わず)
    if let Some(abbr) = matcher::lookup_global(matcher_data, keyword) {
        return build_output(prefix, abbr, &input.rbuffer);
    }

    ExpandOutput::NoMatch
}

fn build_output(prefix: &str, abbr: &CompiledAbbr, rbuffer: &str) -> ExpandOutput {
    if abbr.evaluate {
        return ExpandOutput::Evaluate {
            command: abbr.expansion.clone(),
            prefix: prefix.to_string(),
            rbuffer: rbuffer.to_string(),
        };
    }

    let expansion = &abbr.expansion;

    // プレースホルダー処理
    let new_lbuffer = format!("{}{}", prefix, expansion);
    let full_buffer = format!("{}{}", new_lbuffer, rbuffer);

    let placeholder_result =
        placeholder::apply_first_placeholder(&full_buffer, new_lbuffer.len());

    ExpandOutput::Success {
        buffer: placeholder_result.text,
        cursor: placeholder_result.cursor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Abbreviation, AbbreviationContext};

    fn build_test_matcher() -> Matcher {
        let abbrs = vec![
            Abbreviation {
                keyword: "g".to_string(),
                expansion: "git".to_string(),
                global: false,
                evaluate: false,
                allow_conflict: false,
                context: None,
            },
            Abbreviation {
                keyword: "gc".to_string(),
                expansion: "git commit -m '{{message}}'".to_string(),
                global: false,
                evaluate: false,
                allow_conflict: false,
                context: None,
            },
            Abbreviation {
                keyword: "gp".to_string(),
                expansion: "git push".to_string(),
                global: false,
                evaluate: false,
                allow_conflict: false,
                context: None,
            },
            Abbreviation {
                keyword: "NE".to_string(),
                expansion: "2>/dev/null".to_string(),
                global: true,
                evaluate: false,
                allow_conflict: false,
                context: None,
            },
            Abbreviation {
                keyword: "main".to_string(),
                expansion: "main --branch".to_string(),
                global: false,
                evaluate: false,
                allow_conflict: false,
                context: Some(AbbreviationContext {
                    lbuffer: Some("^git (checkout|switch) ".to_string()),
                    rbuffer: None,
                }),
            },
            Abbreviation {
                keyword: "TODAY".to_string(),
                expansion: "date +%Y-%m-%d".to_string(),
                global: true,
                evaluate: true,
                allow_conflict: false,
                context: None,
            },
        ];
        matcher::build(&abbrs)
    }

    #[test]
    fn test_extract_keyword_simple() {
        let (prefix, keyword) = extract_keyword("g").unwrap();
        assert_eq!(prefix, "");
        assert_eq!(keyword, "g");
    }

    #[test]
    fn test_extract_keyword_with_trailing_space() {
        // 末尾がスペースのみの場合、trim_end でスペースが消えて最後のトークンが返る
        let (prefix, keyword) = extract_keyword("git commit ").unwrap();
        assert_eq!(prefix, "git ");
        assert_eq!(keyword, "commit");
    }

    #[test]
    fn test_extract_keyword_with_args() {
        let (prefix, keyword) = extract_keyword("echo NE").unwrap();
        assert_eq!(prefix, "echo ");
        assert_eq!(keyword, "NE");
    }

    #[test]
    fn test_extract_keyword_empty() {
        assert!(extract_keyword("").is_none());
        assert!(extract_keyword("   ").is_none());
    }

    #[test]
    fn test_is_command_position() {
        assert!(is_command_position(""));
        assert!(is_command_position("  "));
        assert!(!is_command_position("echo "));
        assert!(!is_command_position("git commit "));
    }

    #[test]
    fn test_expand_regular_command_position() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "g".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::Success { buffer, cursor } => {
                assert_eq!(buffer, "git");
                assert_eq!(cursor, 3);
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_regular_not_command_position() {
        let matcher = build_test_matcher();
        // "g" は regular なので、コマンド位置でないとマッチしない
        let input = ExpandInput {
            lbuffer: "echo g".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::NoMatch => {}
            other => panic!("Expected NoMatch, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_global() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "echo hello NE".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::Success { buffer, cursor } => {
                assert_eq!(buffer, "echo hello 2>/dev/null");
                assert_eq!(cursor, 22);
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_with_placeholder() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "gc".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::Success { buffer, cursor } => {
                assert_eq!(buffer, "git commit -m ''");
                assert_eq!(cursor, 15);
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_contextual() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "git checkout main".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::Success { buffer, cursor } => {
                assert_eq!(buffer, "git checkout main --branch");
                assert_eq!(cursor, 26);
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_contextual_no_match() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "git commit main".to_string(),
            rbuffer: "".to_string(),
        };
        // "main" はコンテキスト付きだが、lbuffer が "git commit " なのでマッチしない
        // regular にも "main" はないのでマッチしない
        match expand(&input, &matcher) {
            ExpandOutput::NoMatch => {}
            other => panic!("Expected NoMatch, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_evaluate() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "echo TODAY".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::Evaluate {
                command,
                prefix,
                rbuffer,
            } => {
                assert_eq!(command, "date +%Y-%m-%d");
                assert_eq!(prefix, "echo ");
                assert_eq!(rbuffer, "");
            }
            other => panic!("Expected Evaluate, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_no_match() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "unknown_command".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::NoMatch => {}
            other => panic!("Expected NoMatch, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_empty_input() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "".to_string(),
            rbuffer: "".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::NoMatch => {}
            other => panic!("Expected NoMatch, got {:?}", other),
        }
    }

    #[test]
    fn test_expand_with_rbuffer() {
        let matcher = build_test_matcher();
        let input = ExpandInput {
            lbuffer: "g".to_string(),
            rbuffer: " --help".to_string(),
        };
        match expand(&input, &matcher) {
            ExpandOutput::Success { buffer, cursor } => {
                assert_eq!(buffer, "git --help");
                assert_eq!(cursor, 3);
            }
            other => panic!("Expected Success, got {:?}", other),
        }
    }
}
