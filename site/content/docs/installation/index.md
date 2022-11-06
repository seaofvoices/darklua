---
title: Installation
description: How to install and run darklua
group: Guides
order: 1
---

## Foreman

If you are already using [Foreman](https://github.com/Roblox/foreman), then installing darklua is as simple as adding this line in the `foreman.toml` file:

```toml
darklua = { github = "seaofvoices/darklua", version = "=0.8.0" }
```

## Download a Release

Pre-built binaries are also available in the [releases page](https://github.com/seaofvoices/darklua/releases) on GitHub. Download the appropriate file for you OS and unzip it, then place the executable where it makes sense for you.

If you are unfamiliar with command line tools and not sure where to start, search how to place the executable so that you can run it from anywhere (for example on [windows](https://lmgtfy.app/?q=add+executable+in+path+windows) or [mac](https://lmgtfy.app/?q=add+executable+in+path+mac))

## Cargo

If you are familiar with the Rust environment, you can also build darklua from source using [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```
cargo install darklua
```

If you want to use the lastest darklua available, install it using the git url:

```
cargo install --git https://github.com/seaofvoices/darklua.git
```
