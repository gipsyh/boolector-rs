use std::env;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let out_dir = env::var("OUT_DIR")
        .map_err(|_| "Environmental variable `OUT_DIR` not defined.".to_string())?;
    let src_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "Environmental variable `CARGO_MANIFEST_DIR` not defined.".to_string())?;

    println!(
        "cargo:rustc-link-search=native={}",
        PathBuf::from(out_dir).join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        PathBuf::from(src_dir).display()
    );
    println!("cargo:rustc-link-lib=static=boolector");
    println!("cargo:rustc-link-lib=static=btor2parser");
    println!("cargo:rustc-link-lib=static=lgl");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    Ok(())
}
