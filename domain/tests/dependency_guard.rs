//! Architecture guard: the DDD dependency rule, enforced as a test.
//! `domain` must be GUI/IO-free; `application` may depend only on `domain`.

// Union of forbidden GUI/webview/IO crates. Kept even after a dep is dropped from
// the workspace — forbidding an absent crate is free and future-proof.
const GUI_IO: &[&str] = &["iced", "tauri", "tiff", "ureq", "rfd", "image"];

fn deps_section(manifest: &str) -> String {
    // crude but sufficient: everything after [dependencies]
    manifest.split("[dependencies]").nth(1).unwrap_or("").to_string()
}

#[test]
fn domain_has_no_gui_or_io_dependencies() {
    let manifest = include_str!("../Cargo.toml");
    let deps = deps_section(manifest);
    for c in GUI_IO {
        assert!(!deps.contains(c), "domain must not depend on `{c}` (DDD purity)");
    }
}

#[test]
fn application_depends_only_on_domain() {
    let manifest = include_str!("../../application/Cargo.toml");
    let deps = deps_section(manifest);
    assert!(deps.contains("domain"), "application should depend on domain");
    assert!(!deps.contains("infrastructure"), "application must not depend on infrastructure");
    for c in GUI_IO {
        assert!(!deps.contains(c), "application must not depend on `{c}`");
    }
}
