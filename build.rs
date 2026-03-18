use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    compile_asm_files();
    build_mutliboot_header();
}

fn compile_asm_files() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let mut asm_dir = PathBuf::from("asm/");
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
            println!("cargo:rerun-if-changed={}", path.display());

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

fn build_mutliboot_header() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    use multiboot2_header::*;
    let arch_header_tag = match target_arch.as_str() {
        "x86" | "x86_64" => Some(HeaderTagISA::I386),
        _ => None,
    };
    if let Some(arch_tag) = arch_header_tag {
        let header = Builder::new(arch_tag)
            .information_request_tag(InformationRequestHeaderTag::new(
                    HeaderTagFlag::Optional,
                    &[
                        MbiTagType::Cmdline.into(),
                        MbiTagType::BootLoaderName.into(),
                        MbiTagType::BasicMeminfo.into(),
                        MbiTagType::Mmap.into(),
                        MbiTagType::Framebuffer.into(),
                        MbiTagType::EfiMmap.into(),
                        MbiTagType::LoadBaseAddr.into(),
                        MbiTagType::End.into(),
                    ],
            )).build();

        // Write the header to a binary file
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let header_path = out_dir.join("multiboot_header.bin");

        // Convert the entire header structure to bytes
        let header_ref = header.as_ref().header();
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                header_ref as *const _ as *const u8,
                header_ref.length() as usize,
            )
        };

        fs::write(&header_path, header_bytes)
            .expect("Failed to write multiboot header");

        // The terminating tag
        let end_tag: [u8; 8] = [
            0x00, 0x00,  // type = 0
            0x00, 0x00,  // flags = 0  
            0x08, 0x00, 0x00, 0x00,  // size = 8
        ];

        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&header_path)
            .expect("Failed to write multiboot header");
        use std::io::Write;
        file.write_all(&end_tag).expect("Failed to write multiboot header");

        println!("cargo:rerun-if-changed=build.rs");
    } else {
        println!("cargo:warning=target arch is not supported by multiboot2 protocol!");
    }
}
