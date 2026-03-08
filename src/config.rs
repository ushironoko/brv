use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub abbr: Vec<Abbreviation>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub strict: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Abbreviation {
    pub keyword: String,
    pub expansion: String,
    #[serde(default)]
    pub global: bool,
    #[serde(default)]
    pub evaluate: bool,
    #[serde(default)]
    pub allow_conflict: bool,
    #[serde(default)]
    pub context: Option<AbbreviationContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbbreviationContext {
    #[serde(default)]
    pub lbuffer: Option<String>,
    #[serde(default)]
    pub rbuffer: Option<String>,
}

pub fn load(path: &Path) -> Result<Config> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("設定ファイルを読み込めません: {}", path.display()))?;
    parse(&content)
}

pub fn parse(content: &str) -> Result<Config> {
    let config: Config = toml::from_str(content).context("TOML パースエラー")?;
    validate(&config)?;
    Ok(config)
}

fn validate(config: &Config) -> Result<()> {
    for abbr in &config.abbr {
        if abbr.keyword.is_empty() {
            anyhow::bail!("keyword が空の abbreviation があります");
        }
        if abbr.expansion.is_empty() {
            anyhow::bail!(
                "keyword \"{}\" の expansion が空です",
                abbr.keyword
            );
        }
        if abbr.keyword.contains(' ') {
            anyhow::bail!(
                "keyword \"{}\" にスペースは使用できません",
                abbr.keyword
            );
        }
        // コンテキスト正規表現のバリデーション
        if let Some(ctx) = &abbr.context {
            if let Some(ref pat) = ctx.lbuffer {
                regex::Regex::new(pat).with_context(|| {
                    format!(
                        "keyword \"{}\" の context.lbuffer 正規表現が不正です: {}",
                        abbr.keyword, pat
                    )
                })?;
            }
            if let Some(ref pat) = ctx.rbuffer {
                regex::Regex::new(pat).with_context(|| {
                    format!(
                        "keyword \"{}\" の context.rbuffer 正規表現が不正です: {}",
                        abbr.keyword, pat
                    )
                })?;
            }
        }
    }
    Ok(())
}

/// デフォルトの設定ファイルパスを取得
pub fn default_config_path() -> Result<std::path::PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("brv").context("XDG ディレクトリの取得に失敗")?;
    Ok(xdg.get_config_home().join("brv.toml"))
}

/// デフォルトのキャッシュファイルパスを取得
pub fn default_cache_path() -> Result<std::path::PathBuf> {
    let xdg = xdg::BaseDirectories::with_prefix("brv").context("XDG ディレクトリの取得に失敗")?;
    Ok(xdg.get_cache_home().join("brv.cache"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[[abbr]]
keyword = "g"
expansion = "git"
"#;
        let config = parse(toml).unwrap();
        assert_eq!(config.abbr.len(), 1);
        assert_eq!(config.abbr[0].keyword, "g");
        assert_eq!(config.abbr[0].expansion, "git");
        assert!(!config.abbr[0].global);
        assert!(!config.abbr[0].evaluate);
        assert!(!config.abbr[0].allow_conflict);
        assert!(config.abbr[0].context.is_none());
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[settings]
strict = true

[[abbr]]
keyword = "g"
expansion = "git"

[[abbr]]
keyword = "gc"
expansion = "git commit -m '{{message}}'"

[[abbr]]
keyword = "NE"
expansion = "2>/dev/null"
global = true

[[abbr]]
keyword = "main"
expansion = "main --branch"
context.lbuffer = "^git (checkout|switch)"

[[abbr]]
keyword = "gs"
expansion = "git status --short"
allow_conflict = true

[[abbr]]
keyword = "TODAY"
expansion = "date +%Y-%m-%d"
evaluate = true
global = true
"#;
        let config = parse(toml).unwrap();
        assert!(config.settings.strict);
        assert_eq!(config.abbr.len(), 6);

        // global
        assert!(config.abbr[2].global);
        // context
        assert!(config.abbr[3].context.is_some());
        let ctx = config.abbr[3].context.as_ref().unwrap();
        assert_eq!(ctx.lbuffer.as_deref(), Some("^git (checkout|switch)"));
        // allow_conflict
        assert!(config.abbr[4].allow_conflict);
        // evaluate
        assert!(config.abbr[5].evaluate);
        assert!(config.abbr[5].global);
    }

    #[test]
    fn test_parse_empty_config() {
        let toml = "";
        let config = parse(toml).unwrap();
        assert!(config.abbr.is_empty());
        assert!(!config.settings.strict);
    }

    #[test]
    fn test_parse_settings_only() {
        let toml = r#"
[settings]
strict = true
"#;
        let config = parse(toml).unwrap();
        assert!(config.settings.strict);
        assert!(config.abbr.is_empty());
    }

    #[test]
    fn test_validate_empty_keyword() {
        let toml = r#"
[[abbr]]
keyword = ""
expansion = "git"
"#;
        let result = parse(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("keyword が空"));
    }

    #[test]
    fn test_validate_empty_expansion() {
        let toml = r#"
[[abbr]]
keyword = "g"
expansion = ""
"#;
        let result = parse(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expansion が空"));
    }

    #[test]
    fn test_validate_keyword_with_space() {
        let toml = r#"
[[abbr]]
keyword = "g c"
expansion = "git commit"
"#;
        let result = parse(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("スペースは使用できません"));
    }

    #[test]
    fn test_validate_invalid_regex() {
        let toml = r#"
[[abbr]]
keyword = "main"
expansion = "main --branch"
context.lbuffer = "[invalid"
"#;
        let result = parse(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("正規表現が不正"));
    }

    #[test]
    fn test_default_settings() {
        let config = Config {
            settings: Settings::default(),
            abbr: vec![],
        };
        assert!(!config.settings.strict);
    }
}
