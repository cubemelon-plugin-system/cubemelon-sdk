# Getting Started
CubeMelon Plugin System v0.11.3

![img](../img/lang.en.png)[ ENGLISH](getting_started.en.md)

## 目次

1. [環境構築](#1-環境構築)
2. [プロジェクトの作成](#2-プロジェクトの作成)
3. [TOML ファイルへの追記](#3-TOML-ファイルへの追記)
4. [最小限のプラグイン実装](#4-最小限のプラグイン実装)
5. [実際に動かしてみる](#5-実際に動かしてみる)

---

## 1. 環境構築

### 1.1 Windows SDK のインストール (Windows の場合)
以下のサイトから SDK を探してダウンロード
https://developer.microsoft.com/windows/downloads/windows-sdk/


### 1.2 Rust のインストール
以下のサイトからインストーラをダウンロード
https://www.rust-lang.org/

### 1.3 Git のインストール
以下のサイトからインストーラをダウンロード
https://git-scm.com/

### 1.4 CubeMelon SDK のダウンロード

任意のフォルダ内で、以下の git コマンドを実行してください。

```bash
# Clone this repository
$ git clone https://github.com/cubemelon-plugin-system/cubemelon-sdk
```

---

## 2. プロジェクトの作成

任意のフォルダ内で、以下のコマンドを実行してください。

```bash
$ cargo new my_plugin --lib
$ cd my_plugin && touch src/lib.rs
```

---

## 3. TOML ファイルへの追記

`"../cubemelon-sdk/sdk"` の部分は、プロジェクトフォルダの位置関係によって変わります。

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
cubemelon_sdk = { path = "../cubemelon-sdk/sdk" }
```

---

## 4. 最小限のプラグイン実装

`UUID` は、プロジェクトに固有の値を使ってください。
オンラインジェネレータなどで簡単に作成できます。

```rust
// src/lib.rs
use cubemelon_sdk::prelude::*;

#[plugin]
pub struct MyPlugin {
}

#[plugin_impl]
impl MyPlugin {
    pub fn new() -> Self { Self {} }
    pub fn get_uuid() -> CubeMelonUUID { uuid!("12345678-1234-5678-9abc-123456789abc") }
    pub fn get_version() -> CubeMelonVersion { version!(1, 0, 0) }
    pub fn get_supported_types() -> u64 { CubeMelonPluginType::Basic as u64 }
}

#[plugin_interface(basic)]
impl MyPlugin {}
```

---

## 5. 実際に動かしてみる

### 1. プラグインのビルド

まずはプラグインをビルドします。

```bash
# my_plugin/
$ cargo build --release
```

`my_plugin/target/release` 内にファイルが作られます。

拡張子は
- Window なら **.dll**
- Linux なら **.so**
- macOS なら **.dylib**

になります。

### 2. SDK のビルド

cubemelon-sdk フォルダに移動し、SDK 全体をビルドします。

```bash
# cubemelon-sdk/
$ cargo build --release
```

### 3. テスト環境の作成

先程作ったプラグインを、SDK 内の plugins フォルダの中にコピーします。

```bash
# cubemelon-sdk/
$ cd target/release && mkdir plugins
$ cp "my_plugin.dll" plugins
```

`"my_plugin.dll"` の箇所は、実際に作成されたプラグインファイルのフルパスを入れてください。
図で示すと以下のようになります。

```
├── ...
my_plugin/
├── Cargo.toml
├── src/
│　　　└──lib.rs
├── target/
│　　　└── release/
│　　　　　　　└── my_plugin.dll　←　これを
cubemelon-sdk/
├── Cargo.toml
├── ...
├── target/
│　　　└── release/
│　　　　　　　├── plugins/
│　　　　　　　│　　　└── my_plugin.dll　←　ここにコピー
│　　　　　　　├── cubemelon.exe
│　　　　　　　└ ...
└── ...
```

### 4. テスト

アプリケーションを起動し、動作のテストを行います。

```bash
# cubemelon-sdk/target/release/
$ ./cubemelon
```

`./cubemelon` は、SDK に含まれている動作確認用のコマンドラインアプリケーションです。
Windows の場合は `cubemelon.exe` が、それ以外の OS では `cubemelon` がアプリの実体です。

---

## 6. より複雑なプラグイン

複数の機能を組み合わせることで、より複雑なプラグインを作ることができます。
詳しくは、[仕様書](specification/specification.ja.md)をご覧ください。