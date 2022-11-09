fn main() {
    println!("cargo:rustc-link-search=whisper.cpp");

    #[cfg(target_os = "macos")]
    configure_mac();

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-lstdc++");
}

fn configure_mac() {
    println!("cargo:rustc-link-lib=framework=Accelerate");
    println!("cargo:rustc-link-arg=-lc++");
}
