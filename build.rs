use std::{env, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winres::WindowsResource::new();
        let icon = Path::new("assets/icon.ico");
        if icon.exists() {
            println!("cargo:rerun-if-changed=assets/icon.ico");
            res.set_icon("assets/icon.ico");
        }
        res.set("ProductName", "Nebula")
            .set("FileDescription", "Nebula Discord Client");
        res.compile().expect("failed to compile Windows resources");
    }
}
