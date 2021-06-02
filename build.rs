use std::env;
fn main() {
    let mex_lib = env::var("MEX_LIB_NAME");
    let path = env::var("MEX_LIB_PATH"); //"/usr/lib/x86_64-linux-gnu/";
    println!("cargo:rustc-link-search=native={}", path.unwrap());
    println!("cargo:rustc-link-lib={}", mex_lib.unwrap());
}