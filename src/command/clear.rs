use std::{
    collections::HashSet,
    fs, io,
    os::unix::process,
    path::{Path, PathBuf},
    process::exit,
    sync::{Arc, Mutex, RwLock},
    thread::{self, spawn},
};

use crate::{
    scan_category,
    ui::{self, UI},
};

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
pub struct ScanRow {
    pub path: PathBuf,
    pub project: String,
    pub cate: String,
    pub size: u64, // Bytes
}

impl ScanRow {
    pub fn ref_data(&self) -> [String; 4] {
        [
            self.project.clone(),
            self.cate.as_str().to_string(),
            format!("{:.2}GB", (self.size as f64) / 1024.0 / 1024.0 / 1024.0),
            format!("{:?}", self.path.to_str().unwrap()),
        ]
    }

    pub fn ref_head() -> [&'static str; 4] {
        ["Project", "Cate", "Size", "Path"]
    }
}

async fn scan_target(path: PathBuf, rows: Arc<Mutex<Vec<ScanRow>>>) -> io::Result<()> {
    let mut stack = vec![path];
    let visited = Arc::new(RwLock::new(HashSet::new()));

    while let Some(path) = stack.pop() {
        // sleep(std::time::Duration::from_nanos(1)).await;
        if path.is_dir() {
            // println!("Directory: {:?}", path);
            let _ = tokio::spawn({
                let path = path.clone();
                let scan_rows = rows.clone();
                let visited = visited.clone();
                async move {
                    let is_skip = path.join("rmdev.skip").exists();
                    if is_skip {
                        return;
                    }

                    for cate in scan_category::STRATEGY.iter() {
                        if cate.access_keyfile(&path) {
                            let mut scan_rows = scan_rows.lock().unwrap();
                            let path = path.clone();
                            let project = path.file_name().unwrap().to_str().unwrap().to_string();
                            scan_rows.push(ScanRow {
                                path: path.clone(),
                                cate: cate.ident(),
                                size: get_directory_size(&path, visited).unwrap(),
                                // size: scan_size(path, &mut HashSet::new()).unwrap(),
                                // size: 0,
                                project,
                            });
                            return;
                        }
                    }
                }
            })
            .await?;
            if path.ends_with(".git") {
                continue;
            }
            if scan_category::STRATEGY
                .iter()
                .find(|cate| cate.rm_keyfile(&path))
                .is_some()
            {
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
        }
    }

    Ok(())
}

fn scan_size(path: PathBuf, visited: &mut HashSet<PathBuf>) -> io::Result<u64> {
    let mut stack: Vec<PathBuf> = vec![path];
    let mut size = 0;

    while let Some(path) = stack.pop() {
        thread::sleep(std::time::Duration::from_nanos(1));
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

type Visited = Arc<RwLock<HashSet<PathBuf>>>;

/// 递归计算目录的总大小（以字节为单位），并避免死循环。
fn get_directory_size(path: &Path, visited: Visited) -> io::Result<u64> {
    use rayon::prelude::*;

    let mut total_size = 0;

    // 检查是否已经访问过该目录
    {
        let mut visited_set = visited.write().unwrap();
        if !visited_set.insert(path.to_path_buf()) {
            return Ok(0); // 如果目录已经被访问过，返回 0
        }
    }

    let entries: Vec<_> = fs::read_dir(path)?.collect::<Result<Vec<_>, io::Error>>()?;

    // 使用 Rayon 并行处理目录和文件
    let dir_sizes: u64 = entries
        .par_iter()
        .map(|entry| {
            let entry_path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                // 如果条目是一个目录，则递归计算该目录的大小
                get_directory_size(&entry_path, Arc::clone(&visited))
            } else {
                // 如果条目是一个文件，则返回该文件的大小
                Ok(metadata.len())
            }
        })
        .sum::<Result<u64, io::Error>>()?;

    total_size += dir_sizes;

    Ok(total_size)
}
