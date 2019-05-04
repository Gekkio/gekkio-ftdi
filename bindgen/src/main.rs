use std::env;

use gekkio_ftdi_bindgen::generate_bindings;

fn main() {
    let cwd = env::current_dir().expect("Failed to read current directory");
    let sys_path = cwd
        .join("..")
        .join("sys")
        .canonicalize()
        .expect("Failed to find gekkio-ftdi-sys directory");
    let bindings = generate_bindings().expect("Failed to generate bindings");
    let output_path = sys_path.join("src").join("bindings.rs");
    bindings
        .write_to_file(&output_path)
        .expect("Failed to write bindings");
    println!("Wrote bindings to {}", output_path.to_string_lossy());
}
