![ci workflow](https://github.com/kazuki0824/b25-kit-rs/actions/workflows/rust.yml/badge.svg)

B25 kit
====
Rustで書かれたARIB-STD-B25およびテレビチューナーリーダー
従来のrecpt1, b25を代替する
- クロスプラットフォーム（BonDriver, キャラクタデバイス型の両方を読み取り可能）
- Rustによる実装でシングルボード向け低メモリ消費、連続録画時のエラー防止を目指す
- チャンネル名ハードコード・二重バッファなど、従来のソフトウェアの設計の問題を自分なりに修正
- ECM/EMMロギング・デバッグ機能

Tools for reading ARIB-STD-B25, and dealing with some kinds of tuner devices. Works fine on both Windows and Linux.  
B25-rs and b25-sys are more convenient Rust wrapper for libarib25. Recisdb-rs can read both Unix character device-based and BonDriver-based TV sources. 

## Description
- recisdb-rs: [px4_drv](https://github.com/nns779/px4_drv)
- b25-rs: send the stream from https://www.kernelconfig.io/config_dvb_pt3
- b25-sys: a wrapper for libarib25 written in Rust


## Usage
### Linux
- TODO: recisdb-rustのオプションをここに書く(chardev)
- Video4Linux dvbデバイスはdvbv5-zap --> b25-rsへのパイプで対応
### Windows
- TODO: recisdb-rustのオプションをここに書く(BonDriver)

## Installation
### Linux
### Windows

## Licence
[GPL v3]

## Author
[kazuki0824](https://github.com/kazuki0824)
