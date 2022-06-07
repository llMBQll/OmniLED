extern crate cmake;

fn main() {
    let dst = cmake::build("");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=clock_impl");
}