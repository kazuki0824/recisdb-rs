![ci workflow](https://github.com/kazuki0824/recisdb-rs/actions/workflows/rust.yml/badge.svg)
[![Release](https://github.com/kazuki0824/recisdb-rs/actions/workflows/release.yml/badge.svg)](https://github.com/kazuki0824/recisdb-rs/actions/workflows/release.yml)

recisdb
=======

Rust で書かれた ARIB STD-B25 およびテレビチューナーリーダー。  
従来の recpt1, b25, arib-b25-stream-test コマンドを代替します。

Tools for reading ARIB STD-B25, and dealing with some kinds of tuner devices. Works fine on both Windows and Linux.  
recisdb-rs and b25-sys are more convenient Rust wrapper for libaribb25. recisdb can read both Unix character device-based and BonDriver-based TV sources. 

## Features

- クロスプラットフォーム（BonDriver, キャラクタデバイス型の両方を読み取り可能）
- Rust による実装でシングルボード向け低メモリ消費、連続録画時のエラー防止を目指す
- チャンネル名ハードコード・二重バッファなど、従来のソフトウェアの設計の問題を自分なりに修正
- ECM/EMM ロギング・デバッグ機能

## Description

- recisdb-rs: reads a bitstream from both character devices and BonDriver
- b25-sys: a wrapper for libaribb25 written in Rust

## Build

recisdb をビルドするには Rust が必要です。  
Rust がインストールされていない場合は、[Rustup](https://www.rust-lang.org/ja/tools/install) をインストールしてください。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

具体的には、上記のコマンドでインストールできます。

```bash
git clone https://github.com/kazuki0824/recisdb-rs.git
cd recisdb-rs
sudo apt install -y build-essential cmake clang libpcsclite-dev pkg-config libdvbv5-dev
cargo build -F dvb --release
sudo cp -a target/release/recisdb /usr/local/bin
```

Rust がインストールされている場合は、上記のコマンドでビルドできます。

## Usage

### General

- `tune` `-c <channel> [-t <time>] -device <device> [ <output_path> | - ]`
- `decode` `-i [ <input_path> | - ] [ <output_path> | - ]`
- `checksignal` `-c <channel> -device <device>`

チャンネルは以下のような記法で、物理チャンネルで指定します。

- -c 24 (地上波: 24ch)
- -c T24 (地上波: 24ch)
- -c T60 (地上波: 60ch: 通常地上波は 52ch までだが、CATV のコミュニティチャンネルでは 62ch まで使われていることがある)
- -c C30 (CATV: 30ch)
- -c BS1_0 (BS: BS01/TS0)
- -c BS03 --tsid 16433 (BS: BS03/TS1(TSID=0x4031))
- -c BS23_3 (BS: BS23/TS3)
- -c CS02 (CS: ND02)
- -c CS24 (CS: ND24)

### Linux

```bash
recisdb checksignal --device /dev/px4video2 -c T18  # キャラクタデバイスをオープンし、地上波 18ch を選局
recisdb tune --device /dev/px4video0 -c BS3_1 --lnb low - | ffplay -i -  # キャラクタデバイスをオープンし、BS3 の相対TS番号 1 を選局（LNB なし）
recisdb tune --device /dev/dvb/adapter2/frontend0 -c BS03_1 --lnb low - | ffplay -i -  # DVB デバイスをオープンし、BS3 の相対TS番号 1 を選局
recisdb tune --device "2|0" -c BS3 --tsid 0x4031 --lnb low - | ffplay -i -  # 略記法で DVB デバイスをオープンし、BS3 の TSID=0x4031 を選局（LNB なし）
recisdb decode -i $HOME/hoge.m2ts ./descrambled.m2ts  # ローカルに置かれたファイルのスクランブル解除
```

フィーチャーフラグ`dvb`を有効にすると、DVBデバイスでの録画がサポートされます。  
DVB 版ドライバをお使いの場合、BS のみ V4L-DVB ドライバの API 仕様上選局時に TSID を明示的に指定する必要があるため、recisdb には BS の各スロットごとの TSID がハードコードされています。
DVB 版ドライバでは、チューナの TMCC 情報へのアクセス手段がなく、このような制約が発生します。   
そのため、今後 BS の帯域再編が行われた場合、recisdb 本体を更新する必要があります。
可能であれば、BS の選局時に相対 TS 番号を利用でき、ハードコードされた TSID に依存しない chardev 版ドライバへの切り替えをおすすめします。


### Windows

BonDriver については以下のように使用します。

- チャンネル名を Channel-ChannelSpace の形（例：12-1）で指定
- デバイス名として BonDriver へのパスを渡す

```
recisdb.exe tune --device .\BonDriver_mirakc.dll -c 0-8 -t 20 -
recisdb.exe decode -i %USERPROFILE%\Desktop\hoge.m2ts .\descrambled.m2ts
```

## Licence

[GPL v3](https://github.com/kazuki0824/recisdb-rs/blob/master/LICENSE)


## Author

[maleicacid](https://github.com/kazuki0824)

## Special thanks

このアプリケーションは [px4_drv](https://github.com/nns779/px4_drv) を参考にして実装されています。  
また [libaribb25](https://github.com/tsukumijima/libaribb25) のラッパー実装を含んでいます。

This application has been implemented with reference to [px4_drv](https://github.com/nns779/px4_drv).  
It also contains a wrapper implementation of [libaribb25](https://github.com/tsukumijima/libaribb25).
