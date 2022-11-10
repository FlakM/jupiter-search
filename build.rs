
fn main() {

    let mut cflags = vec!["-I.              -O3 -std=c11 -pthread"];
    let mut cxxflags = vec!["-I. -I./examples -O3 -std=c++11 -pthread"];

    #[cfg(target_os = "macos")]
    configure_mac(&mut cflags, &mut cxxflags);

    #[cfg(target_os = "linux")]
    configure_linux(&mut cflags, &mut cxxflags);


    
    //println!("cargo:rustc-link-search=whisper.cpp");
    cc::Build::new()
        .file("./whisper.cpp/ggml.c")
        .opt_level(3)
        .flag(&cflags.join(" "))
        .warnings(false)
        .compile("ggml.o");

    cc::Build::new()
        .cpp(true) // Switch to C++ library compilation.
        .file("./whisper.cpp/whisper.cpp")
        .opt_level(3)
        .flag(&cxxflags.join(" "))
        .warnings(false)
        .cpp_link_stdlib("stdc++") // use libstdc++
        .compile("whisper.o");


}

#[allow(dead_code)]
fn configure_linux(cflags: &mut Vec<&str>, _cxxflags: &mut Vec<&str>) {
    cflags.push("-mavx -mavx2 -mfma -mf16c");
 //   println!("cargo:rustc-link-arg=-lstdc++");
}


#[allow(dead_code)]
fn configure_mac(cflags: &mut Vec<&str>, _cxxflags: &mut Vec<&str>) {
    cflags.push("-mavx -mavx2 -mfma -mf16c");

    println!("cargo:rustc-link-lib=framework=Accelerate");
//    println!("cargo:rustc-link-arg=-lc++");
}
