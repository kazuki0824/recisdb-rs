![ci workflow](https://github.com/kazuki0824/b25-kit-rs/actions/workflows/rust.yml/badge.svg)

ISDB kit
====
Rustで書かれたARIB-STD-B25およびテレビチューナーリーダー  
従来のrecpt1, b25を代替する  
Tools for reading ARIB-STD-B25, and dealing with some kinds of tuner devices. Works fine on both Windows and Linux.  
recisdb-rs and b25-sys are more convenient Rust wrapper for libarib25. Recisdb-rs can read both Unix character device-based and BonDriver-based TV sources. 
- クロスプラットフォーム（BonDriver, キャラクタデバイス型の両方を読み取り可能）
- Rustによる実装でシングルボード向け低メモリ消費、連続録画時のエラー防止を目指す
- チャンネル名ハードコード・二重バッファなど、従来のソフトウェアの設計の問題を自分なりに修正
- ECM/EMMロギング・デバッグ機能

## Description
- recisdb-rs: reads a bitstream from both character devices and BonDriver
- b25-sys: a wrapper for libarib25 written in Rust


## Usage
### 共通 使用方法
- tune 
- decode
- checksignal

チャンネルは以下のように物理チャンネルで指定する
T24(地上波24ch)
C2(CATV2ch)
BS2_0
CS11

BonDriverについては以下のように

### Linux
```bash
recisdb tune --device /dev/px4video0 -c T18 - | ffplay
recisdb decode -i $HOME/hoge.m2ts ./descrambled.m2ts
```
- Video4Linux...dvbデバイスはdvbv5-zapの出力を標準入力から受ける形で対応
```bash
dvbv5-zap  -a 1 -c ./isdbt.conf -r -P 24 | ./CLionProjects/recisdb-rs/b25-toolkit-rs/target/debug/recisdb decode - | ffplay
```
### Windows
- チャンネル名をChannel-ChannelSpaceの形（例：12-1）で指定
- デバイス名としてBonDriverへのパスを渡す
```
recisdb tune --device .\BonDriver_mirakc.dll -c 0-8 -t 20 -
recisdb decode -i %USERPROFILE%\Desktop\hoge.m2ts .\descrambled.m2ts
```

## Licence
[GPL v3](https://github.com/kazuki0824/b25-kit-rs/blob/master/LICENSE)

## Author
[maleicacid](https://github.com/kazuki0824)

## Special thanks
このアプリケーションは[px4_drv](https://github.com/nns779/px4_drv)を参考にして実装されています。  
また[libarib25](https://github.com/stz2012/libarib25)のラッパー実装を含んでいます。  
This application has been implemented with reference to [px4_drv](https://github.com/nns779/px4_drv).  
It also contains a wrapper implementation of [libarib25](https://github.com/stz2012/libarib25).

