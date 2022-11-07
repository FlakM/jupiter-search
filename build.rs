fn main() {
    println!("cargo:rustc-link-search=whisper.cpp");
    println!("cargo:rustc-link-arg=-lstdc++");
    
}
