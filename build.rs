fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("uefi") {
        println!("cargo:rustc-cfg=efi");
    }
    println!("cargo:rustc-check-cfg=cfg(efi)");
}