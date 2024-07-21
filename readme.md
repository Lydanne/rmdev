<div align="center">
  <p>
    <h1>Rmdev</h1>
  </p>
  <p><img src="./assets/home.png" width="80%" /></p>
  <p>
    <img src="https://img.shields.io/crates/v/rmdev?style=for-the-badge" />
    <img src="https://img.shields.io/github/license/WumaCoder/rmdev?style=for-the-badge" />
  </p>
  <p>
    <a href="./readme-zh.md">中文</a>
  </p>
</div>

## Hello

This is a tool to delete the remaining dependencies and cache files in the development environment, eg: nodule_modules、target...

## Feature

- 🌟 batch deletion
- 🚀 fast deletion
- 🗑 support multi lang
  - nodejs (nodule_modules)
  - rust (target)
  - ...

## Install

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # install cargo

cargo install rmdev
```

## Use

```shell
rmdev clear ../ -f

# ../ clear target dir
```

## Help

```shell
rmdev clear --help
```

## Contribute

I hope that this tool will eventually cover all the major languages, but more of this work needs to be contributed by all of you!

We can contribute by modifying the `src/scan_category.rs` file and writing in the PR what tool in what language generated the.

```rs
use std::path::Path;

use once_cell::sync::Lazy;

pub(crate) static STRATEGY: Lazy<Vec<ScanCate>> =
    Lazy::new(|| vec![ScanCate::Npm, ScanCate::Cargo]);

#[derive(Debug, Clone)]
pub enum ScanCate {
    Npm,
    Cargo,
}

impl ScanCate {
    pub(crate) fn access_keyfile(&self, path: &Path) -> bool {
        match self {
            Self::Npm => path.join("package.json").exists(),
            Self::Cargo => path.join("Cargo.toml").exists(),
        }
    }

    pub(crate) fn rm_keyfile(&self, path: &Path) -> bool {
        match self {
            Self::Npm => path.to_str().unwrap().contains("node_modules"),
            Self::Cargo => path.to_str().unwrap().contains("target"),
        }
    }

    pub(crate) fn ident(&self) -> String {
        match self {
            Self::Npm => "NPM",
            Self::Cargo => "Cargo",
        }
        .to_string()
    }
}


```

The whole process is still very easy, everyone is welcome to contribute.

## About

MIT
