use std::fmt;

/// 展開結果
#[derive(Debug)]
pub enum ExpandOutput {
    /// 展開成功
    Success {
        /// 新しいバッファ全体
        buffer: String,
        /// 新しいカーソル位置
        cursor: usize,
    },
    /// マッチなし
    NoMatch,
    /// コマンド評価が必要
    Evaluate {
        command: String,
        prefix: String,
        rbuffer: String,
    },
    /// キャッシュが古い
    StaleCache,
}

impl fmt::Display for ExpandOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpandOutput::Success { buffer, cursor } => {
                writeln!(f, "success")?;
                writeln!(f, "{}", buffer)?;
                write!(f, "{}", cursor)
            }
            ExpandOutput::NoMatch => {
                write!(f, "no_match")
            }
            ExpandOutput::Evaluate {
                command,
                prefix,
                rbuffer,
            } => {
                writeln!(f, "evaluate")?;
                writeln!(f, "{}", command)?;
                writeln!(f, "{}", prefix)?;
                write!(f, "{}", rbuffer)
            }
            ExpandOutput::StaleCache => {
                write!(f, "stale_cache")
            }
        }
    }
}

/// プレースホルダージャンプ結果
#[derive(Debug)]
pub enum PlaceholderOutput {
    /// ジャンプ成功
    Success {
        buffer: String,
        cursor: usize,
    },
    /// プレースホルダーなし
    NoPlaceholder,
}

impl fmt::Display for PlaceholderOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaceholderOutput::Success { buffer, cursor } => {
                writeln!(f, "success")?;
                writeln!(f, "{}", buffer)?;
                write!(f, "{}", cursor)
            }
            PlaceholderOutput::NoPlaceholder => {
                write!(f, "no_placeholder")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_output_success_display() {
        let output = ExpandOutput::Success {
            buffer: "git commit".to_string(),
            cursor: 10,
        };
        let formatted = output.to_string();
        assert_eq!(formatted, "success\ngit commit\n10");
    }

    #[test]
    fn test_expand_output_no_match_display() {
        let output = ExpandOutput::NoMatch;
        assert_eq!(output.to_string(), "no_match");
    }

    #[test]
    fn test_expand_output_evaluate_display() {
        let output = ExpandOutput::Evaluate {
            command: "date +%Y-%m-%d".to_string(),
            prefix: "echo ".to_string(),
            rbuffer: "".to_string(),
        };
        let formatted = output.to_string();
        assert_eq!(formatted, "evaluate\ndate +%Y-%m-%d\necho \n");
    }

    #[test]
    fn test_expand_output_stale_cache_display() {
        let output = ExpandOutput::StaleCache;
        assert_eq!(output.to_string(), "stale_cache");
    }

    #[test]
    fn test_placeholder_output_success_display() {
        let output = PlaceholderOutput::Success {
            buffer: "git commit -m ''".to_string(),
            cursor: 15,
        };
        let formatted = output.to_string();
        assert_eq!(formatted, "success\ngit commit -m ''\n15");
    }

    #[test]
    fn test_placeholder_output_no_placeholder_display() {
        let output = PlaceholderOutput::NoPlaceholder;
        assert_eq!(output.to_string(), "no_placeholder");
    }
}
