//! `solite-build bundle` — thin CLI wrapper over the library's `bundle` API.

use std::path::Path;

use solite_build::bundle::{BundleError, bundle_to_file};

pub fn run(src_dir: &Path, out: &Path) -> Result<(), BundleError> {
    let generated = bundle_to_file(src_dir, out)?;
    println!(
        "Wrote {} module(s) to {} (entry: {})",
        generated.modules.len(),
        out.display(),
        generated.entry
    );
    Ok(())
}
