extern crate wasm_gc;
extern crate zip;

use std::fs;
use std::io;
use std::process::Command;

fn main() {
    assert!(Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target=wasm32-unknown-unknown")
        .status()
        .expect("failed to run cargo")
        .success(), "failed to build crate");

    let wasm_module = fs::read("./target/wasm32-unknown-unknown/release/primitive_game.wasm")
        .expect("failed to read .wasm file");

    let wasm_module = wasm_gc::garbage_collect_slice(&wasm_module)
        .expect("failed to wasm-gc wasm module");

    let output_file = fs::File::create("./target/game.zip")
        .expect("failed to create output file");

    write_zip(output_file, &wasm_module)
        .expect("failed to write zip");
}

fn write_zip(output_file: fs::File, mut wasm: &[u8]) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipWriter::new(output_file);
    zip.add_directory("assets/", zip::write::FileOptions::default())?;
    zip.start_file("code.wasm", zip::write::FileOptions::default())?;
    io::copy(&mut wasm, &mut zip)?;
    Ok(())
}
