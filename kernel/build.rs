use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let mut asm_dir = PathBuf::from("asm");
    asm_dir.push(env::var("CARGO_CFG_TARGET_ARCH").unwrap());
    println!("cargo:warning=Looking for assembly files in {}", asm_dir.display());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    for entry in fs::read_dir(&asm_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "asm") {
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
