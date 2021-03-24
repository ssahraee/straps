fn main() {
    println!("cargo:rerun-if-changed=src/beta_inverse_wrapper.cpp");
    cc::Build::new()
        .cpp(true)
        .flag("--std=c++11")
        .file("src/beta_inverse_wrapper.cpp")
        .compile("beta_utils");
}
