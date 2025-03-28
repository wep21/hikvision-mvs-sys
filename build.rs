use std::env;
use std::path::PathBuf;

fn main() {
    let sdk_include_path = "/opt/MVS/include";
    let sdk_lib_path = "/opt/MVS/lib/64";

    // Tell Cargo to link against the MVS SDK library
    println!("cargo:rustc-link-search=native={}", sdk_lib_path);
    println!("cargo:rustc-link-lib=dylib=MvCameraControl");

    // Ensure Cargo rebuilds if these headers change
    println!(
        "cargo:rerun-if-changed={}/MvCameraControl.h",
        sdk_include_path
    );
    println!("cargo:rerun-if-changed={}/CameraParams.h", sdk_include_path);

    // Generate Rust bindings
    let bindings = bindgen::Builder::default()
        .header(format!("{}/MvCameraControl.h", sdk_include_path))
        .header(format!("{}/CameraParams.h", sdk_include_path))
        .clang_arg(format!("-I{}", sdk_include_path)) // Include directory
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the `OUT_DIR`
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
