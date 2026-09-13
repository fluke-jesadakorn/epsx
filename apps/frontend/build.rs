use std::{fs, path::PathBuf};

fn main() {
    const STYLESHEET: &str = "public/dist/tailwind.css";
    println!("cargo:rerun-if-changed={STYLESHEET}");
    let path = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"))
        .join(STYLESHEET);
    let css = fs::read_to_string(&path).expect("frozen frontend stylesheet must be committed");
    assert!(
        css.len() > 100_000 && css.contains("--tw-") && css.contains(".dark"),
        "frozen frontend stylesheet is incomplete"
    );
}
