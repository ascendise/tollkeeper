use std::{
    env, fs,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

const TEMPLATES_DIR: &str = "templates";

fn main() {
    println!("Copy package(app) assets to target");
    let target_dir_path = env::var("OUT_DIR").unwrap();
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let templates_dir: PathBuf = [&cargo_manifest_dir, TEMPLATES_DIR].iter().collect();
    let base_path = PathBuf::from(cargo_manifest_dir);
    copy_dir(&templates_dir, &PathBuf::from(target_dir_path), &base_path);
}

fn copy_dir(src_dir_path: &Path, target_dir_path: &Path, base_path: &Path) {
    println!(
        "Copying {} into {}: Base Path '{}'",
        src_dir_path.display(),
        target_dir_path.display(),
        base_path.display()
    );
    for entry in WalkDir::new(src_dir_path) {
        let entry = entry.unwrap();
        if entry.metadata().unwrap().is_dir() {
            let src_dir = entry.path();
            let target_dir = target_dir_path.join(src_dir.strip_prefix(base_path).unwrap());
            println!(
                "Copy directory {} to {}",
                src_dir.display(),
                target_dir.display()
            );
            fs::create_dir_all(target_dir).unwrap();
        } else {
            let src_file = entry.path();
            let target_file = target_dir_path.join(src_file.strip_prefix(base_path).unwrap());
            println!(
                "Copy file {} to {}",
                src_file.display(),
                target_file.display()
            );
            fs::copy(src_file, target_file).unwrap();
        }
    }
}
