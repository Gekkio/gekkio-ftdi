use bindgen::Bindings;
use failure::{format_err, Error};

pub fn generate_bindings() -> Result<Bindings, Error> {
    let bindings = bindgen::builder()
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(clippy::all)]")
        .header_contents("wrapper.h", include_str!("wrapper.h"))
        .rust_target(bindgen::LATEST_STABLE_RUST)
        .whitelist_function("ftdi_.*")
        .whitelist_type("ftdi_.*")
        .layout_tests(true)
        .derive_debug(true)
        .impl_debug(true)
        .rustfmt_bindings(true)
        .generate()
        .map_err(|_| format_err!("Failed to generate bindings"))?;
    Ok(bindings)
}
