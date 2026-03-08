# CLAUDE.md

## Project Overview

brv (brevis = ラテン語で「短い」) は Rust 製の高速・安全な zsh abbreviation 展開ツール。
`brv compile` による事前検証 + バイナリキャッシュで、パフォーマンスと安全性の両方を解決する。

## Build & Development Commands

```bash
cargo build
cargo build --release
cargo test
cargo test <test_name>
cargo bench
```

## Architecture

```
brv.toml (TOML設定)
    │
    ▼
brv compile ─── PATH スキャン + シェルビルトイン照合 ─── 衝突検出 → エラー or 警告
    │
    ▼
brv.cache (バイナリキャッシュ: bincode)
    │
    ▼
brv expand ─── キャッシュ読み込み → HashMap ルックアップ → 結果出力
    │
    ▼
ZLE ウィジェット ─── $BUFFER/$CURSOR 書き換え
```

## Module Structure

- **main.rs**: clap CLI エントリポイント
- **lib.rs**: ライブラリクレート (テスト・ベンチ用)
- **config.rs**: TOML 設定パース
- **compiler.rs**: brv compile: 検証 + キャッシュ生成
- **conflict.rs**: 衝突検出エンジン (PATH スキャン, ビルトイン照合)
- **matcher.rs**: HashMap ベースのマッチングエンジン
- **expand.rs**: 展開ロジック (トークナイズ → ルックアップ → 結果)
- **placeholder.rs**: {{placeholder}} 処理
- **context.rs**: lbuffer/rbuffer 正規表現コンテキスト判定
- **cache.rs**: バイナリキャッシュ読み書き (bincode)
- **output.rs**: ZLE 向け出力フォーマット

## Dependency Version Policy

Cargo.toml での依存クレートは必ず正確なバージョンを指定する。
