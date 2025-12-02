use sha2::{Digest, Sha256};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use xz2::read::XzDecoder;

const LINUX_IMAGE_VERSION: &str = "v0.20.0";
const LINUX_VERSION: &str = "6.5.13-ctsi-1";
const LINUX_HEADERS_SHA256: [u8; 32] =
    hex_literal::hex!("2723435e8b45d8fb7a79e9344f6dc517b3dbc08e03ac17baab311300ec475c08");

const LIBCMT_PATH: &str = "machine-guest-tools/sys-utils/libcmt";

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let is_riscv_target = target.starts_with("riscv64");
    let linux_headers_path = if is_riscv_target {
        Some(resolve_linux_headers_path(&out_dir))
    } else {
        None
    };

    // 2. Prepare Source (Sandbox)
    let original_src = manifest_dir.join(LIBCMT_PATH);
    let build_dir = out_dir.join("libcmt_build");

    // Always clean and recopy to ensure no stale artifacts
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).unwrap();
    }
    copy_dir_recursive(&original_src, &build_dir).expect("Failed to copy source");

    let headers_path = build_dir.join("include/libcmt");

    // 3. Build with Make
    // We use C_INCLUDE_PATH so we don't have to fight the Makefile's CFLAGS.
    let lib_output_dir = if is_riscv_target {
        let mut make = Command::new("make");
        make.current_dir(&build_dir)
            .env("TOOLCHAIN_PREFIX", "riscv64-unknown-linux-musl-");
        if let Some(path) = &linux_headers_path {
            make.env("C_INCLUDE_PATH", path);
        }
        let status = make.status().expect("Failed to execute make");

        if !status.success() {
            panic!("libcmt Makefile failed");
        }

        build_dir.join("build/riscv64")
    } else {
        println!("Skipping real build on host; building mock");

        // Mock build for Host / LSP
        let status = Command::new("make")
            .current_dir(&build_dir)
            .arg("mock")
            .status()
            .expect("Failed to execute make");

        if !status.success() {
            panic!("libcmt Makefile mock failed");
        }

        build_dir.join("build/mock")
    };

    // 4. Generate Bindings
    // Bindgen needs to know about the headers too!
    let mut bindgen = bindgen::Builder::default()
        .header(headers_path.join("rollup.h").to_str().unwrap())
        .use_core()
        .ctypes_prefix("core::ffi");
    if let Some(path) = &linux_headers_path {
        bindgen = bindgen
            .clang_arg(format!("-I{}", path.display()))
            .clang_arg("--target=riscv64-unknown-linux-musl");
    }
    let bindings = bindgen.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // 5. Link
    // Sanity check to save hours of debugging
    if !lib_output_dir.join("libcmt.a").exists() {
        panic!(
            "Build succeeded but libcmt.a not found at expected path: {}",
            lib_output_dir.display()
        );
    }

    println!(
        "cargo:rustc-link-search=native={}",
        lib_output_dir.display()
    );
    println!("cargo:rustc-link-lib=static=cmt");
    println!("cargo:rerun-if-changed={}", LIBCMT_PATH);
    println!("cargo:rerun-if-env-changed=LIBCMT_SYS_LINUX_HEADERS_DIR");
}

fn resolve_linux_headers_path(out_dir: &Path) -> PathBuf {
    if let Ok(path) = env::var("LIBCMT_SYS_LINUX_HEADERS_DIR") {
        let linux_headers_path = PathBuf::from(path);
        assert!(
            linux_headers_path.exists(),
            "LIBCMT_SYS_LINUX_HEADERS_DIR does not exist: {}",
            linux_headers_path.display()
        );
        return linux_headers_path;
    }

    let linux_headers_path = out_dir.join("usr/riscv64-linux-gnu/include");
    if !linux_headers_path.exists() {
        download_and_extract_headers(out_dir);
    }

    assert!(
        linux_headers_path.exists(),
        "linux headers path missing after download/extract: {}",
        linux_headers_path.display()
    );

    linux_headers_path
}

fn download_and_extract_headers(out_dir: &Path) {
    let url = format!(
        "https://github.com/cartesi/machine-linux-image/releases/download/{LINUX_IMAGE_VERSION}/linux-libc-dev-riscv64-cross-{LINUX_VERSION}-{LINUX_IMAGE_VERSION}.deb"
    );

    println!("Downloading Linux headers...");

    let data = reqwest::blocking::get(url)
        .expect("Failed to download headers")
        .bytes()
        .expect("Failed to read bytes");

    // Checksum
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let digest: [u8; 32] = hasher.finalize().into();
    assert_eq!(digest, LINUX_HEADERS_SHA256, "Checksum mismatch!");

    // Extract .deb -> ar -> data.tar.xz -> tar -> out_dir
    let mut archive = ar::Archive::new(data.as_ref());
    while let Some(Ok(entry)) = archive.next_entry() {
        if entry.header().identifier() == b"data.tar.xz" {
            let xz = XzDecoder::new(entry);
            let mut tar = tar::Archive::new(xz);
            tar.unpack(out_dir).expect("Failed to unpack tar");
            return;
        }
    }

    panic!("Corrupt .deb: data.tar.xz not found");
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
