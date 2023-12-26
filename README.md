recisdb
=======

![ci workflow](https://github.com/kazuki0824/recisdb-rs/actions/workflows/rust.yml/badge.svg)
[![Release](https://github.com/kazuki0824/recisdb-rs/actions/workflows/release.yml/badge.svg)](https://github.com/kazuki0824/recisdb-rs/actions/workflows/release.yml)

Rust で書かれたテレビチューナーリーダー / ARIB STD-B25 デコーダーです。  
従来の recpt1 / dvbv5-zap / b25 / arib-b25-stream-test コマンドの代替として利用できます。

Tools for reading ARIB STD-B25, and dealing with some kinds of tuner devices. Works fine on both Windows and Linux.  
recisdb-rs and b25-sys are more convenient Rust wrapper for libaribb25. recisdb can read both Unix character device-based and BonDriver-based TV sources. 

---

## Features

- クロスプラットフォーム (BonDriver / キャラクタデバイス (chardev) / DVBv5 デバイスすべてに対応)
- Rust による実装でシングルボード向け低メモリ消費、連続録画時のエラー防止を目指す
- チャンネル名ハードコード・二重バッファなど、従来のソフトウェアの設計の問題を自分なりに修正
- ECM/EMM ロギング・デバッグ機能

## Description

- recisdb-rs: reads a bitstream from both character devices and BonDriver
- b25-sys: a wrapper for libaribb25 written in Rust

## Installation

[Releases](https://github.com/kazuki0824/recisdb-rs/releases) に Ubuntu 20.04 以降向けの Debian パッケージ (.deb) と、Windows (x64) 向けの実行ファイル (.exe) を用意しています。  

Linux では、下記のコマンドで recisdb をインストールできます。  
以下は v1.1.0 をインストールする例です。依存パッケージは自動的にインストールされます。

```bash
# x86_64 環境
wget https://github.com/kazuki0824/recisdb-rs/releases/download/1.1.0/recisdb_1.1.0_amd64.deb
sudo apt install ./recisdb_1.1.0_amd64.deb
rm ./recisdb_1.1.0_amd64.deb

# arm64 環境
wget https://github.com/kazuki0824/recisdb-rs/releases/download/1.1.0/recisdb_1.1.0_arm64.deb
sudo apt install ./recisdb_1.1.0_arm64.deb
rm ./recisdb_1.1.0_arm64.deb
```
Windows では `recisdb.exe` をダウンロードし、適当なフォルダに配置してください。

---

## Usage

### General

recisdb には、3 つのサブコマンドがあります。

`recisdb checksignal` : チャンネルを選局し、信号レベル (dB) を確認します。
```bash
recisdb checksignal [OPTIONS] --device <CANONICAL_PATH> --channel <CHANNEL>
```  

`recisdb tune` : チャンネルを選局し、指定された出力先に受信した TS データを書き出します。
```bash
recisdb tune [OPTIONS] --device <CANONICAL_PATH> --channel <CHANNEL> <OUTPUT>
```
> [!NOTE]  
> ** v1.1.0 から `--no-simd` オプションが追加されました。 **  
> decode時にSIGILLが発生する場合、AVX2命令がご使用のCPUに実装されていないことが考えられます。
> その際はこのオプションを使用することで問題を回避できます。

> [!NOTE]  
> ** v1.2.0 から `-e` オプションが追加されました。 **  
> B-CASカードの抜き取りなどの理由でデコーダーがエラーを返した場合、プログラムを終了します。  
> 逆にデフォルトでは、プログラムを終了せずにデコーダーなしで処理を続行します。

`recisdb decode` : 指定された入力ファイルを ARIB STD-B25 に基づきデコードし、指定された出力先に TS データを書き出します。  
```bash
recisdb decode [OPTIONS] --input <file> <OUTPUT>
```

詳しいオプションは `recisdb --help` / `recisdb <SUBCOMMAND> --help` を参照してください。

### Channel

`-c` または `--channel` でのチャンネル指定フォーマットは下記の通りです。  
**recpt1 のチャンネル指定フォーマットとは微妙に異なるため注意してください。**

- `-c T27` (地上波: 27ch)
  - recpt1 では単に 27 と指定するが、recisdb では T27 と指定する
- `-c T60` (地上波: 60ch)
  - 通常地上波の物理チャンネルは 52ch までだが、CATV のコミュニティチャンネルでは 62ch まで使われていることがある
- `-c C30` (CATV: 30ch)
  - ISDB-T の CATV チャンネルでは C を付ける
- `-c BS1_0` (BS: BS01/TS0)
  - BS01_0 と BS1_0 の両方のフォーマットに対応
- `-c BS03 --tsid 16433` (BS: BS03/TS1 (TSID = 0x4031))
  - BS ではスロット (相対 TS 番号) の代わりに TSID を指定して選局することも可能
  - この例では明示的に BS03 (トランスポンダ) の中から TSID = 0x4031 を持つ TS (NHK BSプレミアム) を選局している
- `-c BS23_3` (BS: BS23/TS3)
- `-c CS02` (CS: ND02)
  - CS2 と CS02 の両方のフォーマットに対応
- `-c CS24` (CS: ND24)

> [!IMPORTANT]  
> ISDB-T / ISDB-S 以外の放送方式 (ISDB-C / DVB-S2 など) には対応していません。

> [!NOTE]  
> **接続/認識されているチューナーデバイスの確認やチャンネルスキャンには、recisdb をチューナーコマンドとして利用する [ISDBScanner](https://github.com/tsukumijima/ISDBScanner) が便利です。**  
> 受信可能な日本のテレビチャンネル (ISDB-T/ISDB-S) を全自動でスキャンし、スキャン結果を EDCB (EDCB-Wine)・Mirakurun・mirakc の各設定ファイルや JSON 形式で出力できます。

### Linux

Linux では、キャラクタデバイス (chardev) または DVBv5 デバイスを指定して選局します。

> [!NOTE]  
> **DVB デバイスのサポートは v1.2.0 から追加されたものです。**  
> v1.1.0 以前のバージョンでは、DVB デバイスを操作することはできません。

> [!WARNING]  
> **DVB 版ドライバ利用時のみ、BS の選局にはスロット番号 (相対 TS 番号) ではなく、recisdb 本体にハードコードされた各スロットと TSID の対照表が利用されます。**  
> DVB 版ドライバでは DVBv5 API の仕様上、BS のみ選局時に TSID を明示的に指定する必要があります。しかし DVB 版ドライバではチューナーが持つ TMCC 情報へのアクセス手段がないため、選局に必要な TSID を相対 TS 番号から算出することができません。   
> このため、**もし今後 BS の帯域再編が行われた場合、帯域再編後も DVB 版ドライバで BS を受信するには、recisdb 自体の更新が必要となります。**  
>   
> PT1 / PT2 / PT3 など chardev 版ドライバが別途存在する機種をお使いの方は、**BS の選局に相対 TS 番号が利用でき、帯域再編後もハードコードされた TSID に依存せず選局可能な、chardev 版ドライバへの切り替えをおすすめします。**

#### Examples

```bash
# キャラクタデバイスをオープンし、地上波 18ch を選局し、信号レベル (dB) を確認
recisdb checksignal --device /dev/px4video2 --channel T18

# DVB デバイス (/dev/dvb/adapter1/frontend0) をオープンし、地上波 27ch を選局し、スクランブル解除せずに /tmp/recorded.m2ts に保存
recisdb tune --device /dev/dvb/adapter1/frontend0 -c T27 --no-decode /tmp/recorded.m2ts

# キャラクタデバイスをオープンし、CATV (ISDB-T) 30ch を選局し、30 秒間の録画データを recorded.m2ts に保存
recisdb tune --device /dev/px4video3 --channel C30 --time 30 recorded.m2ts

# キャラクタデバイスをオープンし、BS03 の相対 TS 番号 1 を選局し、パイプ渡しで ffplay で再生
recisdb tune --device /dev/px4video0 --channel BS3_1 - | ffplay -i -

# DVB デバイス (/dev/dvb/adapter0/frontend0) をオープンし、BS03 の相対 TS 番号 1 を選局し (LNB 給電あり: 15V)、20 秒間の録画データを recorded.m2ts に保存
recisdb tune --device /dev/dvb/adapter0/frontend0 -c BS03_1 --lnb 15v --time 20 recorded.m2ts

# 略記法で DVB デバイス (/dev/dvb/adapter2/frontend0) をオープンし、BS03 (TSID = 0x4031) を選局し、スクランブル解除せずに sudo tee で /root/recorded.m2ts に保存
recisdb tune --device "2|0" -c BS3 --tsid 0x4031 --no-decode - | sudo tee /root/recorded.m2ts > /dev/null

# キャラクタデバイスをオープンし、CS04 を選局し、スクランブル解除せずに /tmp/recorded.m2ts に保存
recisdb tune --device /dev/px4video5 -c CS04 --no-decode /tmp/recorded.m2ts

# スクランブル解除されていない $HOME/scrambled.m2ts をNULL パケットの除去を行わずにスクランブル解除し、descrambled.m2ts に保存
recisdb decode -i $HOME/scrambled.m2ts --no-strip ./descrambled.m2ts
```

### Windows

Windows では、BonDriver のファイルパスと、BonDriver 上のチャンネル空間番号 / チャンネル通し番号を指定して選局します。  
**BonDriver インターフェイスの技術的制約により、Linux 版とはチャンネル指定方法が異なるため注意してください。**

- チャンネル: `<ChannelSpace>-<Channel>` のフォーマット (例: `1-12`) で指定
- デバイス: BonDriver (.dll) へのファイルパスを指定

> [!WARNING]  
> **Windows (BonDriver) 対応は実験的なものであり、動作検証はあまり行われていません。**  
> **このため、一部の BonDriver やチューナーでは正常に動作しない可能性があります。**  
> また、現時点では Windows 版には `recisdb checksignal` は実装されていません。

#### Examples

```powershell
# BonDriver_mirakc.dll をオープンし、チャンネル空間番号:0 / チャンネル通し番号:8 を選局し、録画データを recorded.m2ts に保存
recisdb.exe tune --device .\BonDriver_mirakc.dll -c 0-8 -t 20 recorded.m2ts

# スクランブル解除されていない %USERPROFILE%\Desktop\scrambled.m2ts をスクランブル解除し、descrambled.m2ts に保存
recisdb.exe decode -i %USERPROFILE%\Desktop\scrambled.m2ts .\descrambled.m2ts
```

---

## Build

recisdb をビルドするには Rust が必要です。  
Rust がインストールされていない場合は、[Rustup](https://www.rust-lang.org/ja/tools/install) をインストールしてください。

### Windows (MSVC)


### Windows (MSYS MinGW)


### Debian系
> [!NOTE]  
> 以下のコマンドは Ubuntu 22.04 でのインストール方法です。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

上記のコマンドで Rustup をインストールできます。  
Rustup をインストールするだけで、Rust とビルドに必要なツールチェインが同時にインストールされます。

```bash
git clone https://github.com/kazuki0824/recisdb-rs.git
cd recisdb-rs
sudo apt install -y build-essential clang cmake libdvbv5-dev libpcsclite-dev libudev-dev pkg-config
cargo build -F dvb --release
sudo cp -a target/release/recisdb /usr/local/bin
```

Rust をインストールしたら、上記のコマンドで recisdb をビルドできます。  
ビルドした recisdb は、`target/release/recisdb` に生成されます。  
`cargo install`などで、パスの通った場所へ実行ファイルを自動的に配置することができます。

> [!IMPORTANT]  
> `cargo build` を実行する際 `-F dvb` を指定すると、libdvbv5 経由での DVB デバイスの操作がサポートされます。  
> `-F dvb` を指定してビルドした場合、動作には別途 `libdvbv5-0` パッケージが必要です。

---

## Licence

[GPL v3](https://github.com/kazuki0824/recisdb-rs/blob/master/LICENSE)

## Author

[maleicacid](https://github.com/kazuki0824)

## Special thanks

このアプリケーションは [px4_drv](https://github.com/nns779/px4_drv) を参考にして実装されています。  
また [libaribb25](https://github.com/tsukumijima/libaribb25) のラッパー実装を含んでいます。

This application has been implemented with reference to [px4_drv](https://github.com/nns779/px4_drv).  
It also contains a wrapper implementation of [libaribb25](https://github.com/tsukumijima/libaribb25).
