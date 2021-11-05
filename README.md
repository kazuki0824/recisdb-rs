![ci workflow](https://github.com/kazuki0824/b25-kit-rs/actions/workflows/rust.yml/badge.svg)

# B25 kit
Rustで書かれたARIB-STD-B25およびテレビチューナーリーダー
従来のrecpt1, b25を代替する
- クロスプラットフォーム（BonDriver, キャラクタデバイス型の両方を読み取り可能）
- Rustによる実装でシングルボード向け低メモリ消費、連続録画時のエラー防止を目指す
- チャンネル名ハードコード・二重バッファなど、従来のソフトウェアの設計の問題を自分なりに修正
- ECM/EMMロギング・デバッグ機能

## Usage
### Linux
- TODO: recisdb-rustのオプションをここに書く(chardev)
- Video4Linux dvbデバイスはdvbv5-zap --> b25-rsへのパイプで対応
### Windows
- TODO: recisdb-rustのオプションをここに書く(BonDriver)

## Installation
### Linux
### Windows
