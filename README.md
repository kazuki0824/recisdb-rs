![ci workflow](https://github.com/kazuki0824/b25-kit-rs/actions/workflows/rust.yml/badge.svg)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fkazuki0824%2Frecisdb-rs.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fkazuki0824%2Frecisdb-rs?ref=badge_shield)
[![Release](https://github.com/kazuki0824/recisdb-rs/actions/workflows/release.yml/badge.svg)](https://github.com/kazuki0824/recisdb-rs/actions/workflows/release.yml)

recisdb
=======

Rust で書かれた ARIB STD-B25 およびテレビチューナーリーダー。  
従来の recpt1, b25, arib-b25-stream-test コマンドを代替します。

Tools for reading ARIB STD-B25, and dealing with some kinds of tuner devices. Works fine on both Windows and Linux.  
recisdb-rs and b25-sys are more convenient Rust wrapper for libarib25. Recisdb-rs can read both Unix character device-based and BonDriver-based TV sources. 

## Features

- クロスプラットフォーム（BonDriver, キャラクタデバイス型の両方を読み取り可能）
- Rust による実装でシングルボード向け低メモリ消費、連続録画時のエラー防止を目指す
- チャンネル名ハードコード・二重バッファなど、従来のソフトウェアの設計の問題を自分なりに修正
- ECM/EMM ロギング・デバッグ機能

## Description

- recisdb-rs: reads a bitstream from both character devices and BonDriver
- b25-sys: a wrapper for libarib25 written in Rust

## Usage

### General

- `tune` `-c <channel> [-t <time>] -device <device> [ <output_path> | - ]`
- `decode` `-i [ <input_path> | - ] [ <output_path> | - ]`
- `checksignal` `-c <channel> -device <device>`

チャンネルは以下のように、物理チャンネルで指定します。

- -c T24 (地上波: 24ch)
- -c T60 (地上波: 60ch: 通常地上波は 52ch までだが、CATV のコミュニティチャンネルでは 62ch まで使われていることがある)
- -c C30 (CATV: 30ch)
- -c BS01_0 (BS: BS01/TS0)
- -c BS23_3 (BS: BS23/TS3)
- -c CS02 (CS: ND02)
- -c CS24 (CS: ND24)

### Linux

```bash
recisdb tune --device /dev/px4video0 -c T18 - | ffplay
recisdb decode -i $HOME/hoge.m2ts ./descrambled.m2ts
```

Video4Linux DVB デバイスは、dvbv5-zap の出力を標準入力から受ける形で対応します。

```bash
dvbv5-zap -a 1 -c ./isdbt.conf -r -P 24 | recisdb decode - | ffplay
```

### Windows

BonDriver については以下のように使用します。

- チャンネル名を Channel-ChannelSpace の形（例：12-1）で指定
- デバイス名として BonDriver へのパスを渡す

```
recisdb.exe tune --device .\BonDriver_mirakc.dll -c 0-8 -t 20 -
recisdb.exe decode -i %USERPROFILE%\Desktop\hoge.m2ts .\descrambled.m2ts
```

## Licence

[GPL v3](https://github.com/kazuki0824/b25-kit-rs/blob/master/LICENSE)

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fkazuki0824%2Frecisdb-rs.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fkazuki0824%2Frecisdb-rs?ref=badge_large)

## Author

[maleicacid](https://github.com/kazuki0824)

## Special thanks

このアプリケーションは [px4_drv](https://github.com/nns779/px4_drv) を参考にして実装されています。  
また [libaribb25](https://github.com/tsukumijima/libaribb25) のラッパー実装を含んでいます。

This application has been implemented with reference to [px4_drv](https://github.com/nns779/px4_drv).  
It also contains a wrapper implementation of [libaribb25](https://github.com/tsukumijima/libaribb25).
