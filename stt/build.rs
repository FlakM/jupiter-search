#[derive(Default)]
struct BuildCtx {
    cflags: Vec<&'static str>,
    cxxflags: Vec<&'static str>,
    std_lib: &'static str,
}

fn main() {
    let mut config = BuildCtx {
        cflags: vec!["-I.              -O3 -pthread"],
        cxxflags: vec!["-I. -I./examples -O3 -pthread"],
        ..Default::default()
    };

    #[cfg(target_os = "macos")]
    configure_mac(&mut config);

    #[cfg(target_os = "linux")]
    configure_linux(&mut config);

    cc::Build::new()
        .file("./whisper.cpp/ggml.c")
        .flag(&config.cflags.join(" "))
        .opt_level(3)
        .warnings(false)
        .compile("ggml.o");

    cc::Build::new()
        .cpp(true) // Switch to C++ library compilation.
        .file("./whisper.cpp/whisper.cpp")
        .flag(&config.cxxflags.join(" "))
        .opt_level(3)
        .warnings(false)
        .cpp_link_stdlib(config.std_lib) // use libstdc++
        .compile("whisper.o");
}

#[allow(dead_code)]
fn configure_linux(config: &mut BuildCtx) {
    config.cflags.push("-mfma -mf16c -mavx -mavx2 ");
    //   println!("cargo:rustc-link-arg=-lstdc++");
    config.std_lib = "stdc++";
}

#[allow(dead_code)]
fn configure_mac(config: &mut BuildCtx) {
    config.cflags.push("-mavx -mavx2 -mfma -mf16c");
    config.std_lib = "c++";
    println!("cargo:rustc-link-lib=framework=Accelerate");
    //    println!("cargo:rustc-link-arg=-lc++");
}
