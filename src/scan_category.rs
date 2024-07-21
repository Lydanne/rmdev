use std::path::Path;

use once_cell::sync::Lazy;

pub(crate) static STRATEGY: Lazy<Vec<ScanCate>> =
    Lazy::new(|| vec![ScanCate::Npm, ScanCate::Cargo]);

#[derive(Debug)]
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
            Self::Npm => path.join("node_modules").exists(),
            Self::Cargo => path.join("target").exists(),
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
