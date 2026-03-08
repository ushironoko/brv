use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use brv::{cache, compiler, config, expand, output, placeholder};

#[derive(Parser, Debug)]
#[command(name = "brv")]
#[command(about = "Fast and safe abbreviation expansion for zsh")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 設定コンパイル + 衝突検証
    Compile {
        /// サフィックス衝突もエラーにする
        #[arg(long, default_value = "false")]
        strict: bool,

        /// 設定ファイルパス
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// 展開 (ZLE から呼び出し)
    Expand {
        /// カーソル左側のバッファ
        #[arg(long)]
        lbuffer: String,

        /// カーソル右側のバッファ
        #[arg(long)]
        rbuffer: String,

        /// キャッシュファイルパス
        #[arg(long)]
        cache: Option<PathBuf>,

        /// 設定ファイルパス (鮮度チェック用)
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// プレースホルダージャンプ
    NextPlaceholder {
        /// カーソル左側のバッファ
        #[arg(long)]
        lbuffer: String,

        /// カーソル右側のバッファ
        #[arg(long)]
        rbuffer: String,
    },

    /// 登録済み abbreviation 一覧
    List {
        /// 設定ファイルパス
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// 設定の構文チェックのみ
    Check {
        /// 設定ファイルパス
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// 設定ファイル雛形生成
    Init,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Compile { strict, config: cfg } => cmd_compile(strict, cfg),
        Commands::Expand {
            lbuffer,
            rbuffer,
            cache: cache_path,
            config: cfg,
        } => cmd_expand(lbuffer, rbuffer, cache_path, cfg),
        Commands::NextPlaceholder { lbuffer, rbuffer } => cmd_next_placeholder(lbuffer, rbuffer),
        Commands::List { config: cfg } => cmd_list(cfg),
        Commands::Check { config: cfg } => cmd_check(cfg),
        Commands::Init => cmd_init(),
    }
}

fn resolve_config_path(config: Option<PathBuf>) -> Result<PathBuf> {
    match config {
        Some(p) => Ok(p),
        None => config::default_config_path(),
    }
}

fn resolve_cache_path(cache: Option<PathBuf>) -> Result<PathBuf> {
    match cache {
        Some(p) => Ok(p),
        None => config::default_cache_path(),
    }
}

fn cmd_compile(strict: bool, cfg: Option<PathBuf>) -> Result<()> {
    let config_path = resolve_config_path(cfg)?;
    let cache_path = resolve_cache_path(None)?;

    if !config_path.exists() {
        anyhow::bail!(
            "設定ファイルが見つかりません: {}\n`brv init` で雛形を生成できます",
            config_path.display()
        );
    }

    let result = compiler::compile(&config_path, &cache_path, strict)?;

    // 警告表示
    for warning in &result.warnings {
        eprintln!("  ⚠ {}", warning);
    }

    eprintln!(
        "✓ {} 件の abbreviation をコンパイルしました → {}",
        result.abbr_count,
        cache_path.display()
    );

    Ok(())
}

fn cmd_expand(
    lbuffer: String,
    rbuffer: String,
    cache_path: Option<PathBuf>,
    cfg: Option<PathBuf>,
) -> Result<()> {
    let cache_file = resolve_cache_path(cache_path)?;
    let config_path = resolve_config_path(cfg)?;

    // キャッシュ読み込み
    let compiled = match cache::read(&cache_file) {
        Ok(c) => c,
        Err(_) => {
            println!("{}", output::ExpandOutput::StaleCache);
            return Ok(());
        }
    };

    // 鮮度チェック
    if config_path.exists() {
        if let Ok(fresh) = cache::is_fresh(&compiled, &config_path) {
            if !fresh {
                println!("{}", output::ExpandOutput::StaleCache);
                return Ok(());
            }
        }
    }

    let input = expand::ExpandInput { lbuffer, rbuffer };
    let result = expand::expand(&input, &compiled.matcher);
    println!("{}", result);

    Ok(())
}

fn cmd_next_placeholder(lbuffer: String, rbuffer: String) -> Result<()> {
    let full_buffer = format!("{}{}", lbuffer, rbuffer);
    let cursor = lbuffer.len();

    match placeholder::find_next_placeholder(&full_buffer, cursor) {
        Some((start, end)) => {
            // プレースホルダーを除去してカーソルをそこに移動
            let mut new_buffer = String::with_capacity(full_buffer.len() - (end - start));
            new_buffer.push_str(&full_buffer[..start]);
            new_buffer.push_str(&full_buffer[end..]);

            println!(
                "{}",
                output::PlaceholderOutput::Success {
                    buffer: new_buffer,
                    cursor: start,
                }
            );
        }
        None => {
            println!("{}", output::PlaceholderOutput::NoPlaceholder);
        }
    }

    Ok(())
}

fn cmd_list(cfg: Option<PathBuf>) -> Result<()> {
    let config_path = resolve_config_path(cfg)?;

    if !config_path.exists() {
        anyhow::bail!(
            "設定ファイルが見つかりません: {}\n`brv init` で雛形を生成できます",
            config_path.display()
        );
    }

    let config = config::load(&config_path)?;

    if config.abbr.is_empty() {
        println!("(abbreviation が登録されていません)");
        return Ok(());
    }

    // ヘッダー
    println!("{:<15} {:<6} {:<40}", "KEYWORD", "TYPE", "EXPANSION");
    println!("{}", "-".repeat(65));

    for abbr in &config.abbr {
        let abbr_type = if abbr.context.is_some() {
            "ctx"
        } else if abbr.global {
            "global"
        } else {
            "reg"
        };

        let expansion = if abbr.expansion.len() > 38 {
            format!("{}...", &abbr.expansion[..35])
        } else {
            abbr.expansion.clone()
        };

        let mut flags = Vec::new();
        if abbr.evaluate {
            flags.push("eval");
        }
        if abbr.allow_conflict {
            flags.push("allow");
        }
        let flag_str = if flags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", flags.join(","))
        };

        println!("{:<15} {:<6} {}{}", abbr.keyword, abbr_type, expansion, flag_str);
    }

    println!("\n合計: {} 件", config.abbr.len());
    Ok(())
}

fn cmd_check(cfg: Option<PathBuf>) -> Result<()> {
    let config_path = resolve_config_path(cfg)?;

    if !config_path.exists() {
        anyhow::bail!(
            "設定ファイルが見つかりません: {}\n`brv init` で雛形を生成できます",
            config_path.display()
        );
    }

    let count = compiler::check(&config_path)?;
    eprintln!("✓ 設定ファイルは有効です ({} 件の abbreviation)", count);
    Ok(())
}

fn cmd_init() -> Result<()> {
    let config_path = config::default_config_path()?;

    if config_path.exists() {
        anyhow::bail!(
            "設定ファイルは既に存在します: {}",
            config_path.display()
        );
    }

    // 親ディレクトリを作成
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let template = r#"# brv - abbreviation configuration
# See: https://github.com/ushironoko/brv

[settings]
strict = false  # true: サフィックス衝突もエラーにする

# Regular abbreviation (コマンド位置のみで展開)
[[abbr]]
keyword = "g"
expansion = "git"

[[abbr]]
keyword = "gc"
expansion = "git commit -m '{{message}}'"

[[abbr]]
keyword = "gp"
expansion = "git push"

# Global abbreviation (位置を問わず展開)
# [[abbr]]
# keyword = "NE"
# expansion = "2>/dev/null"
# global = true

# Context abbreviation (コンテキスト条件付き)
# [[abbr]]
# keyword = "main"
# expansion = "main --branch"
# context.lbuffer = "^git (checkout|switch)"

# Command evaluation (コマンド実行結果で展開)
# [[abbr]]
# keyword = "TODAY"
# expansion = "date +%Y-%m-%d"
# evaluate = true
# global = true
"#;

    std::fs::write(&config_path, template)?;
    eprintln!("✓ 設定ファイルを生成しました: {}", config_path.display());
    Ok(())
}
