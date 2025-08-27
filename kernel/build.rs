use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let asm_dir = PathBuf::from("asm");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    for entry in fs::read_dir(&asm_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |ext| ext == "asm") {
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let out_path = out_dir.join(format!("{file_stem}.o"));
            println!("cargo:return-if-changed={}", path.display());

            let status = Command::new("nasm")
                .args([
                    "-f",
                    "elf32",
                    path.to_str().unwrap(),
                    "-o",
                    out_path.to_str().unwrap(),
                ])
                .status()
                .expect("Failed to run nasm");

            assert!(status.success(), "NASM failed on {}", path.display());

            println!("cargo:rustc-link-arg={}", out_path.display());
        }
    }
}
