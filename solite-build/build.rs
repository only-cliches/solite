//! Refresh the vendored runtime type definitions.
//!
//! `templates/runtime.d.ts` is a committed copy of the canonical
//! `js/types/solite-runtime.d.ts` (which lives at the repo root, outside this
//! crate, so it can be packaged for `cargo publish`). When building inside the
//! repo this script copies the canonical file over the vendored copy, so the
//! committed copy — and therefore every `cargo publish` — stays current. When
//! built as a published/registry dependency the canonical file is absent and
//! this is a no-op.

use std::path::Path;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let canonical = Path::new(&manifest).join("../js/types/solite-runtime.d.ts");
    let vendored = Path::new(&manifest).join("templates/runtime.d.ts");

    // No canonical source (e.g. building from a published crate): keep the
    // committed vendored copy as-is.
    if !canonical.exists() {
        return;
    }

    println!("cargo:rerun-if-changed={}", canonical.display());

    let fresh = std::fs::read(&canonical).expect("read canonical runtime.d.ts");
    let current = std::fs::read(&vendored).unwrap_or_default();
    // Only write when stale, so a clean tree stays clean (no needless dirtying).
    if current != fresh {
        std::fs::write(&vendored, fresh).expect("refresh vendored runtime.d.ts");
    }
}
