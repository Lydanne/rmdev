use std::{
    collections::HashSet,
    fs, io,
    os::unix::process,
    path::{Path, PathBuf},
    process::exit,
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::ui::{self, UI};

#[derive(clap::Parser, Debug)]
pub struct Clear {
    /// scan target dir
    pub target: String,

    // /// force clean all
    // #[clap(short, long)]
    // pub force: bool,
    /// ci env
    #[clap(short, long)]
    pub ci: bool,
}

impl Clear {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rows = Arc::new(Mutex::new(Vec::new()));

        if self.ci {
            scan_target(self.target.clone().into(), rows.clone()).await?;
            println!("Clear {} project cache.", rows.lock().unwrap().len());
        } else {
            // println!("Scan rows: {:?}", rows);
            let th = spawn({
                let rows = rows.clone();
                move || {
                    let code = ui::boot(UI { rows }).unwrap();
                    if code != 0 {
                        exit(0);
                    }
                    return code;
                }
            });

            scan_target(self.target.clone().into(), rows.clone()).await?;

            let code = th.join().unwrap();

            if code == 0 {
                println!("Clear {} project cache.", rows.lock().unwrap().len());
                for row in rows.lock().unwrap().iter() {
                    println!("{:?}", row);
                }
            }
        }

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

async fn scan_target(path: PathBuf, rows: Arc<Mutex<Vec<ScanRow>>>) -> io::Result<()> {
    let mut stack = vec![path];
    let mut visited = Vec::new();

    while let Some(path) = stack.pop() {
        if path.is_dir() {
            // println!("Directory: {:?}", path);
            let task = tokio::spawn({
                let path = path.clone();
                let scan_rows = rows.clone();
                async move {
                    let is_npm = path.join("package.json").exists();
                    let is_cargo = path.join("Cargo.toml").exists();
                    let is_skip = path.join("rmdev.skip").exists();
                    if is_skip {
                        return;
                    }

                    if is_npm {
                        let mut scan_rows = scan_rows.lock().unwrap();
                        let path = path.clone();
                        let project = path.file_name().unwrap().to_str().unwrap().to_string();
                        scan_rows.push(ScanRow {
                            path: path.clone(),
                            cate: ScanCate::Npm,
                            // size: scan_size(path_clone, &mut HashSet::new())?,
                            size: 0,
                            project,
                        });
                        // println!("Scan: {:?} {}", path, scan_rows.len());
                    } else if is_cargo {
                        let mut scan_rows = scan_rows.lock().unwrap();
                        let path = path.clone();
                        let project = path.file_name().unwrap().to_str().unwrap().to_string();

                        scan_rows.push(ScanRow {
                            path: path.clone(),
                            cate: ScanCate::Cargo,
                            // size: scan_size(path_clone, &mut HashSet::new())?,
                            size: 0,
                            project,
                        });
                        // println!("Scan: {:?} {}", path, scan_rows.len());
                    }
                }
            });
            visited.push(task);
            if path.ends_with("node_modules") || path.ends_with("target") {
                continue;
            }
            let dir = fs::read_dir(path);
            if let Ok(dir) = dir {
                for entry in dir {
                    let entry = entry;
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        stack.push(entry_path);
                    }
                }
            }
        } else {
            // println!("File: {:?}", path);
        }
    }

    for v in visited.drain(..) {
        v.await?;
    }

    Ok(())
}

fn scan_size(path: PathBuf, visited: &mut HashSet<PathBuf>) -> io::Result<u64> {
    let mut stack: Vec<PathBuf> = vec![path];
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
