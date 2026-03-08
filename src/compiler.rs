use crate::{cache, config, conflict, matcher};
use anyhow::Result;
use std::path::Path;

/// コンパイル結果
#[derive(Debug)]
pub struct CompileResult {
    pub warnings: Vec<conflict::Conflict>,
    pub abbr_count: usize,
}

/// brv compile のメインフロー
pub fn compile(config_path: &Path, output_path: &Path, strict: bool) -> Result<CompileResult> {
    // 1. TOML パース
    let cfg = config::load(config_path)?;

    // settings.strict と CLI の --strict を OR で結合
    let effective_strict = strict || cfg.settings.strict;

    // 2. PATH スキャン
    let path_commands = conflict::scan_path();

    // 3. 衝突検出
    let report = conflict::detect_conflicts(&cfg.abbr, &path_commands, effective_strict);

    // 4. エラーがあればコンパイル失敗
    if report.has_errors() {
        // エラーメッセージを構築
        let mut message = String::from("衝突が検出されました:\n");
        for err in &report.errors {
            message.push_str(&format!("  ✗ {}\n", err));
        }
        for warn in &report.warnings {
            message.push_str(&format!("  ⚠ {}\n", warn));
        }
        message.push_str("\nヒント: allow_conflict = true で個別に衝突を許可できます");
        anyhow::bail!(message);
    }

    // 5. 警告を収集
    let warnings = report.warnings.clone();

    // 6. Matcher 構築 + バイナリキャッシュ書き出し
    let matcher = matcher::build(&cfg.abbr);
    cache::write(output_path, &matcher, config_path)?;

    Ok(CompileResult {
        warnings,
        abbr_count: cfg.abbr.len(),
    })
}

/// 設定の構文チェックのみ (コンパイルなし)
pub fn check(config_path: &Path) -> Result<usize> {
    let cfg = config::load(config_path)?;
    Ok(cfg.abbr.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_config(dir: &TempDir, content: &str) -> std::path::PathBuf {
        let config_path = dir.path().join("brv.toml");
        std::fs::write(&config_path, content).unwrap();
        config_path
    }

    #[test]
    fn test_compile_success() {
        let dir = TempDir::new().unwrap();
        let config_path = write_config(
            &dir,
            r#"
[[abbr]]
keyword = "g"
expansion = "git"

[[abbr]]
keyword = "gc"
expansion = "git commit"
"#,
        );
        let cache_path = dir.path().join("brv.cache");

        let result = compile(&config_path, &cache_path, false).unwrap();
        assert_eq!(result.abbr_count, 2);
        assert!(cache_path.exists());
    }

    #[test]
    fn test_compile_builtin_conflict_error() {
        let dir = TempDir::new().unwrap();
        let config_path = write_config(
            &dir,
            r#"
[[abbr]]
keyword = "cd"
expansion = "custom_cd"
"#,
        );
        let cache_path = dir.path().join("brv.cache");

        let result = compile(&config_path, &cache_path, false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cd"));
        assert!(err_msg.contains("ビルトイン"));
    }

    #[test]
    fn test_compile_allow_conflict() {
        let dir = TempDir::new().unwrap();
        let config_path = write_config(
            &dir,
            r#"
[[abbr]]
keyword = "cd"
expansion = "custom_cd"
allow_conflict = true
"#,
        );
        let cache_path = dir.path().join("brv.cache");

        let result = compile(&config_path, &cache_path, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_valid_config() {
        let dir = TempDir::new().unwrap();
        let config_path = write_config(
            &dir,
            r#"
[[abbr]]
keyword = "g"
expansion = "git"
"#,
        );
        let count = check(&config_path).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_check_invalid_config() {
        let dir = TempDir::new().unwrap();
        let config_path = write_config(
            &dir,
            r#"
[[abbr]]
keyword = ""
expansion = "git"
"#,
        );
        let result = check(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_settings_strict() {
        let dir = TempDir::new().unwrap();
        // strict モードでもサフィックス衝突がないキーワードなら成功する
        let config_path = write_config(
            &dir,
            r#"
[settings]
strict = true

[[abbr]]
keyword = "xyzzy"
expansion = "some command"
"#,
        );
        let cache_path = dir.path().join("brv.cache");

        let result = compile(&config_path, &cache_path, false);
        assert!(result.is_ok());
    }
}
