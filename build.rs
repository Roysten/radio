fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_ARCH");
    match target_os.as_ref().map(|x| &**x) {
        Ok("arm") => println!("cargo:rustc-link-search=/mnt/data/libmpv_musl"),
        _ => (),
    }
    println!("cargo:rustc-link-lib=dylib=mpv");
}
