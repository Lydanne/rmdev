<div align="center">
  <p>
    <h1>Rmdev</h1>
  </p>
  <p><img src="./assets/home.png" width="80%" /></p>
  <p>
    <img src="https://img.shields.io/crates/v/rmdev?style=for-the-badge" />
    <img src="https://img.shields.io/github/license/WumaCoder/rmdev?style=for-the-badge" />
  </p>
</div>

## 开始

这是一个工具，用于删除开发环境中的剩余依赖项和缓存文件，例如：node_modules、target 等。

## 特性

- 🌟 批量删除
- 🚀 快速删除
- 🗑 支持多种语言
  - nodejs (node_modules)
  - rust (target)
  - ...

## 安装

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # 安装 cargo

cargo install rmdev
```

## 使用

```shell
rmdev clear ../ -f

# ../ 清除 target 目录
```

## 帮助

```shell
rmdev clear --help
```

## 贡献

我希望这个工具最终能覆盖所有主要语言，但更多的工作需要大家的贡献！

我们可以通过修改 `src/scan_category.rs` 文件来贡献，并在 PR 中说明是什么语言生成的什么工具。

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

整个过程非常简单，欢迎大家贡献。

## 关于

MIT
