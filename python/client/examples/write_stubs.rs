//! Write the type stubs of the extension module `unitycatalog_client._client`.
//!
//! PyO3's introspection (`experimental-inspect`) records the module's classes, functions and
//! signatures in the compiled library; this reads them back and writes one stub per module.
//!
//! ```text
//! cargo build -p unitycatalog-client-py
//! cargo run -p unitycatalog-client-py --example write_stubs -- <cdylib> <stub dir>
//! ```

use std::path::PathBuf;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let (Some(cdylib), Some(out)) = (args.next(), args.next()) else {
        eprintln!("usage: write_stubs <cdylib> <stub dir>");
        std::process::exit(2);
    };
    let module = pyo3_introspection::introspect_cdylib(PathBuf::from(cdylib), "_client")
        .expect("read the introspection data of the extension module");
    let out = PathBuf::from(out);
    let _ = std::fs::remove_dir_all(&out);
    for (path, stub) in pyo3_introspection::module_stub_files(&module) {
        let path = out.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, stub).unwrap();
    }
}
