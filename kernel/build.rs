use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let mut asm_dir = PathBuf::from("asm");
    asm_dir.push(&target_arch);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Determine NASM output format based on architecture
    let nasm_format = match target_arch.as_str() {
        "x86" => "elf32",
        "x86_64" => "elf64",
        arch => panic!("Unsupported architecture: {}", arch),
    };

    for entry in fs::read_dir(&asm_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "asm") {
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let out_path = out_dir.join(format!("{file_stem}.o"));
            println!("cargo:return-if-changed={}", path.display());

            let status = Command::new("nasm")
                .args([
                    "-f",
                    nasm_format,
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
