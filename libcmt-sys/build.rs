use bytes::Buf;
use hex_literal::hex;
use sha2::{Digest, Sha512};
use std::path::PathBuf;
use xz2::read::XzDecoder;

const LIBCMT_URL: &str = "https://github.com/cartesi/machine-emulator-tools/releases/download/v0.15.0/libcmt-v0.15.0-dev.deb";
const LIBCMT_CHECKSUM: [u8; 64] = hex!("b8578f27279ebeecd10d68d037cac403e77a10553d54211d4862cebdb202520fec81483adf969f01bb4abe5cca7ecb6b980e29e881cc002e57b26e3a5a48ad76");

fn main() {
    download_libcmt_a();

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let libcmt_dir_path = PathBuf::from("./machine-emulator-tools/sys-utils/libcmt/src/")
        .canonicalize()
        .expect("cannot canonicalize path");

    let libcmt_bindings = bindgen::Builder::default()
        .header(libcmt_dir_path.join("rollup.h").to_str().unwrap())
        .generate()
        .expect("Unable to generate libcmt bindings");

    libcmt_bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write libcmt bindings");

    println!("cargo:rerun-if-changed=build.rs");
}

fn download_libcmt_a() {
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // get
    let data = reqwest::blocking::get(LIBCMT_URL)
        .expect("error downloading libcmt")
        .bytes()
        .expect("error getting libcmt request body");

    // checksum
    let mut hasher = Sha512::new();
    hasher.update(&data);
    let result = hasher.finalize();
    assert_eq!(result[..], LIBCMT_CHECKSUM, "libcmt checksum failed");

    let mut archive = ar::Archive::new(data.reader());
    let entry = loop {
        if let Some(Ok(entry)) = archive.next_entry() {
            if entry.header().identifier() == "data.tar.xz".as_bytes() {
                break entry;
            }
        } else {
            panic!("file not found")
        }
    };

    // extract
    let xz = XzDecoder::new(entry);
    let mut inner_archive = tar::Archive::new(xz);
    let mut found = false;
    for file in inner_archive.entries().unwrap() {
        let mut libcmt = file.expect("error opening file");
        let libcmt_path = libcmt.header().path().expect("could not get archive path");
        let libcmt_expected_path = "./usr/riscv64-linux-gnu/lib/libcmt.a";

        if libcmt_path.to_str().expect("invalid utf8 string") == libcmt_expected_path {
            libcmt
                .unpack(out_path.join("libcmt.a"))
                .expect("could not copy libcmt.a");

            found = true;
            break;
        }
    }

    if !found {
        panic!("could not find libcmt.a");
    }

    // compiler flags
    println!("cargo:rustc-link-search={}", out_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static=cmt");
}
