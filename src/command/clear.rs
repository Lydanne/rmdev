use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use crate::ui;

#[derive(clap::Parser, Debug)]
pub struct Clear {
    /// eg: qxg
    pub target: String,

    /// force to clean
    #[clap(short, long)]
    pub force: bool,

    /// ci env
    #[clap(short, long)]
    pub ci: bool,
}

impl Clear {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rows = scan_target(self.target.clone().into())?;
        // println!("Scan rows: {:?}", rows);

        ui::boot(rows)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum ScanCate {
    Npm,
    Cargo,
}

impl ScanCate {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Npm => "NPM",
            Self::Cargo => "Cargo",
        }
    }
}

#[derive(Debug)]
pub struct ScanRow {
    pub path: PathBuf,
    pub project: String,
    pub cate: ScanCate,
    pub size: u64, // Bytes
}

impl ScanRow {
    pub fn ref_data(&self) -> [String; 4] {
        [
            self.project.clone(),
            self.cate.as_str().to_string(),
            "-".to_string(),
            format!("{:?}", self.path.to_str().unwrap()),
        ]
    }

    pub fn ref_head() -> [&'static str; 4] {
        ["Project", "Cate", "Size", "Path"]
    }
}

fn scan_target(path: PathBuf) -> io::Result<Vec<ScanRow>> {
    let mut stack = vec![path];
    let mut scan_rows = vec![];

    while let Some(path) = stack.pop() {
        if path.is_dir() {
            // println!("Directory: {:?}", path);

            let is_npm = path.join("package.json").exists();
            let is_cargo = path.join("Cargo.toml").exists();

            if is_npm {
                let path = path.clone();
                let project = path.file_name().unwrap().to_str().unwrap().to_string();
                scan_rows.push(ScanRow {
                    path: path.clone(),
                    cate: ScanCate::Npm,
                    // size: scan_size(path_clone, &mut HashSet::new())?,
                    size: 0,
                    project,
                });
            } else if is_cargo {
                let path = path.clone();
                let project = path.file_name().unwrap().to_str().unwrap().to_string();

                scan_rows.push(ScanRow {
                    path: path.clone(),
                    cate: ScanCate::Cargo,
                    // size: scan_size(path_clone, &mut HashSet::new())?,
                    size: 0,
                    project,
                });
            }
            if path.ends_with("node_modules") || path.ends_with("target") {
                continue;
            }
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                stack.push(entry_path);
            }
        } else {
            // println!("File: {:?}", path);
        }
    }

    Ok(scan_rows)
}

fn scan_size(path: PathBuf, visited: &mut HashSet<PathBuf>) -> io::Result<u64> {
    let mut stack = vec![path];
    let mut size = 0;

    while let Some(path) = stack.pop() {
        if visited.contains(&path) {
            continue;
        }
        visited.insert(path.clone());
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                stack.push(entry_path);
            }
        } else {
            if let Ok(path) = path.metadata() {
                size += path.len();
            }
        }
    }

    Ok(size)
}
