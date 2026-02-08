use std::path::{Path, PathBuf};

pub fn cookies_file() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    exe.parent().unwrap_or(Path::new(".")).join("cookies.json")
}

pub fn books_root() -> PathBuf {
    let exe = std::env::current_exe().unwrap();
    exe.parent().unwrap_or(Path::new(".")).join("Books")
}
