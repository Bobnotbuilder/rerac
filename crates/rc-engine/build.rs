fn main() {
    println!("cargo:rerun-if-changed=src/ios_gamecontroller.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("ios") {
        cc::Build::new()
            .file("src/ios_gamecontroller.m")
            .flag("-fobjc-arc")
            .flag("-fmodules")
            .compile("rerac_gamecontroller");
        println!("cargo:rustc-link-lib=framework=GameController");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=UIKit");
    }
}
