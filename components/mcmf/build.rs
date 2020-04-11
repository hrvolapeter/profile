fn main() {
    let config = cc::Build::new()
        .opt_level(3)
        .cpp(true)
        .file("bindings.cpp")
        .include(".")
        .compile("libflow.a");
    println!("cargo:rerun-if-changed=lemon");
}
