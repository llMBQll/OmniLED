use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("omni_led_api.h")
        .clang_arg("-DMBQ_OMNI_LED_HOST")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate C API bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("c_api_bindings.rs"))
        .expect("Couldn't write bindings!");

    prost_build::compile_protos(&["plugin/plugin.proto"], &["plugin/"])
        .expect("Failed to generate Protouf types");
}
