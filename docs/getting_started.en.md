# Getting Started
CubeMelon Plugin System v0.11.3

![img](../img/lang.ja.png)[ 日本語](getting_started.ja.md)
## Table of Contents

1. [Environment Setup](#1-environment-setup)
2. [Project Creation](#2-project-creation)
3. [Adding to TOML File](#3-adding-to-toml-file)
4. [Minimal Plugin Implementation](#4-minimal-plugin-implementation)
5. [Running It](#5-running-it)

---

## 1. Environment Setup

### 1.1 Windows SDK Installation (for Windows)
Find and download the SDK from the following site:
https://developer.microsoft.com/windows/downloads/windows-sdk/


### 1.2 Rust Installation
Download the installer from the following site:
https://www.rust-lang.org/

### 1.3 Git Installation
Download the installer from the following site:
https://git-scm.com/

### 1.4 CubeMelon SDK Download

In any folder, execute the following git command:

```bash
# Clone this repository
$ git clone https://github.com/cubemelon-plugin-system/cubemelon-sdk.git
```

---

## 2. Project Creation

In any folder, execute the following commands:

```bash
$ cargo new my_plugin --lib
$ cd my_plugin && touch src/lib.rs
```

---

## 3. Adding to TOML File

The `"../cubemelon-sdk/sdk"` part will vary depending on the relative position of your project folder.

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
cubemelon_sdk = { path = "../cubemelon-sdk/sdk" }
```

---

## 4. Minimal Plugin Implementation

Use a unique `UUID` value for your project.
You can easily create one using online generators.

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

## 5. Running It

### 1. Building the Plugin

First, build your plugin.

```bash
# my_plugin/
$ cargo build --release
```

A file will be created in `my_plugin/target/release`.

The file extension will be:
- **`.dll`** on Windows
- **`.so`** on Linux
- **`.dylib`** on macOS

### 2. Building the SDK

Move to the cubemelon-sdk folder and build the entire SDK.

```bash
# cubemelon-sdk/
$ cargo build --release
```

### 3. Creating the Test Environment

Copy the plugin you created into the plugins folder within the SDK.

```bash
# cubemelon-sdk/
$ cd target/release && mkdir plugins
$ cp "my_plugin.dll" plugins
```

Replace `"my_plugin.dll"` with the full path of the plugin file you actually created.
The structure looks like this:

```
├── ...
my_plugin/
├── Cargo.toml
├── src/
│　　　└──lib.rs
├── target/
│　　　└── release/
│　　　　　　　└── my_plugin.dll　←　Copy this
cubemelon-sdk/
├── Cargo.toml
├── ...
├── target/
│　　　└── release/
│　　　　　　　├── plugins/
│　　　　　　　│　　　└── my_plugin.dll　←　To here
│　　　　　　　├── cubemelon.exe
│　　　　　　　└ ...
└── ...
```

### 4. Testing

Launch the application and test its operation.

```bash
# cubemelon-sdk/target/release/
$ ./cubemelon
```

`./cubemelon` is a command-line application included in the SDK for testing functionality.
On Windows, the actual app is `cubemelon.exe`, while on other OSes it's `cubemelon`.

---

## 6. More Complex Plugins

You can create more complex plugins by combining multiple features.
For details, please see the [Specification](specification/specification.en.md).