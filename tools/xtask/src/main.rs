//! `cargo xtask layers` — enforce the L0-L8 layering rules of TensorLab.
//!
//! Rules (see AGENTS.md):
//! 1. A crate at layer L may only depend on internal crates with layer < L.
//! 2. Same-layer dependencies are forbidden.
//! 3. Crates must never depend on apps; apps may depend on any internal crate.
//! 4. Every layer-map crate must exist in the workspace.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Layer assignments (crate name -> layer). Apps are unlayered.
const LAYERS: &[(&str, u32)] = &[
    ("tl-types", 0),
    ("tl-linalg", 1),
    ("tl-kernel", 2),
    ("tl-parser", 3),
    ("tl-interpreter", 4),
    ("tl-jit", 5),
    ("tl-mex", 6),
    ("tl-toolbox", 7),
    ("tl-commands", 8),
];

/// L0 special case: the only external crates `tl-types` may use.
const L0_ALLOWED_EXTERNAL: &[&str] = &["serde", "serde_json", "runmat-value"];

fn layer_of(crate_name: &str) -> Option<u32> {
    LAYERS
        .iter()
        .find(|(n, _)| *n == crate_name)
        .map(|(_, l)| *l)
}

fn is_app(crate_name: &str) -> bool {
    layer_of(crate_name).is_none() && crate_name != "xtask"
}

fn find_workspace_manifests(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in ["crates", "apps"] {
        let base = root.join(dir);
        if let Ok(entries) = fs::read_dir(&base) {
            for e in entries.flatten() {
                let m = e.path().join("Cargo.toml");
                if m.exists() {
                    out.push(m);
                }
            }
        }
    }
    out.sort();
    out
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("workspace root");

    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0usize;

    // Discover workspace crates: name -> (manifest path, is app dir)
    let mut discovered: BTreeMap<String, bool> = BTreeMap::new(); // name -> is_app
    for manifest in find_workspace_manifests(&root) {
        let text = fs::read_to_string(&manifest).expect("read Cargo.toml");
        let value: toml::Value = text.parse().expect("parse Cargo.toml");
        let name = value["package"]["name"].as_str().map(str::to_string);
        let Some(name) = name else {
            failures.push(format!("{}: missing [package].name", manifest.display()));
            continue;
        };
        let is_app_dir = manifest
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.file_name().map(|n| n == "apps").unwrap_or(false))
            .unwrap_or(false);
        discovered.insert(name, is_app_dir);
    }

    for (name, is_app_dir) in &discovered {
        if name == "xtask" {
            continue;
        }
        let dir = if *is_app_dir { "apps" } else { "crates" };
        let manifest = root.join(dir).join(name).join("Cargo.toml");
        let text = fs::read_to_string(&manifest).expect("read Cargo.toml");
        let value: toml::Value = text.parse().expect("parse Cargo.toml");

        let deps: Vec<String> = ["dependencies", "dev-dependencies"]
            .iter()
            .filter_map(|k| value.get(k).and_then(|v| v.as_table()))
            .flat_map(|t| t.keys().cloned())
            .collect();
        checked += deps.len();

        let self_layer = layer_of(name);

        for dep in deps {
            let Some(&dep_is_app) = discovered.get(&dep) else {
                // external dependency
                if self_layer == Some(0) && !L0_ALLOWED_EXTERNAL.contains(&dep.as_str()) {
                    failures.push(format!(
                        "{name}: L0 may only depend on {L0_ALLOWED_EXTERNAL:?}, found `{dep}`"
                    ));
                }
                continue;
            };
            // internal dependency
            if dep_is_app || is_app(&dep) {
                failures.push(format!("{name}: crate must not depend on app `{dep}`"));
                continue;
            }
            match (self_layer, layer_of(&dep)) {
                (Some(sl), Some(dl)) if dl >= sl => failures.push(format!(
                    "{name}: layer {sl} depends on `{dep}` (layer {dl}) — must be strictly lower"
                )),
                _ => {}
            }
        }
    }

    // Rule 4: layer-map crates must exist in the workspace
    for (lname, _) in LAYERS {
        if !discovered.contains_key(*lname) {
            failures.push(format!("layer map references missing crate `{lname}`"));
        }
    }

    let status = if failures.is_empty() { "OK" } else { "FAILED" };
    println!("xtask layers: {status} ({checked} dependency edges checked)");
    for f in &failures {
        println!("  x {f}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
    println!("  layering invariants hold");
}
