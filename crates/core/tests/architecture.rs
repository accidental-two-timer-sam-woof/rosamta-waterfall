use std::{fs, path::Path};

use syn::{ItemUse, UseTree, visit::Visit};

const BASE: &[&str] = &["diag", "name", "host", "path", "fs", "layout"];

// Module dependencies from detailed design §3.2.
const MODULES: &[(&str, &[&str])] = &[
    ("diag", &["name", "path"]),
    ("name", &[]),
    ("host", &[]),
    ("path", &["host"]),
    ("fs", &["diag", "path"]),
    ("layout", &["diag", "host", "path"]),
    ("config", &[]),
    ("docedit", &["diag"]),
    ("resolve", &["config"]),
    ("template", &["config"]),
    ("state", &[]),
    ("plan", &["config", "resolve", "template", "state"]),
    (
        "preflight",
        &["config", "resolve", "template", "state", "plan"],
    ),
    ("recovery", &["state"]),
    ("exec", &["state", "plan", "recovery"]),
    (
        "apply",
        &[
            "config",
            "resolve",
            "template",
            "state",
            "plan",
            "preflight",
            "recovery",
            "exec",
        ],
    ),
    ("manage", &["config", "docedit", "state"]),
    ("query", &["config", "resolve", "template"]),
    ("init", &["config", "docedit", "apply"]),
];

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "architecture test locates Rust source files"
)]
fn core_modules_follow_the_design_dependency_map() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let library = parse(&source.join("lib.rs"));
    let actual: Vec<_> = library
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(module) => Some(module.ident.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(
        actual.len(),
        MODULES.len(),
        "review new modules against 05 §3.2"
    );

    for &(module, allowed) in MODULES {
        assert!(
            actual.iter().any(|name| name == module),
            "missing module {module}"
        );
        let mut check = DependencyCheck {
            module,
            allowed: allowed.to_vec(),
        };
        if !BASE.contains(&module) && module != "docedit" {
            check.allowed.extend_from_slice(BASE);
        }
        let path = source.join(module);
        let file = path.with_extension("rs");
        if file.is_file() {
            check.visit_file(&parse(&file));
        }
        if path.is_dir() {
            check.visit_directory(&path);
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "architecture test reads Rust source files"
)]
fn parse(path: &Path) -> syn::File {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    syn::parse_file(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

struct DependencyCheck<'a> {
    module: &'a str,
    allowed: Vec<&'a str>,
}

impl DependencyCheck<'_> {
    #[expect(
        clippy::disallowed_methods,
        reason = "architecture test walks Rust source directories"
    )]
    fn visit_directory(&mut self, path: &Path) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                self.visit_directory(&entry.path());
            } else if entry.path().extension().is_some_and(|ext| ext == "rs") {
                self.visit_file(&parse(&entry.path()));
            }
        }
    }

    fn require_dependency(&self, name: &str) {
        assert!(
            name == self.module || self.allowed.contains(&name),
            "{} cannot depend on {name} (05 §3.2)",
            self.module
        );
    }

    fn check_import(&self, tree: &UseTree, parent: Option<&str>) {
        let segment = match tree {
            UseTree::Group(group) => {
                for item in &group.items {
                    self.check_import(item, parent);
                }
                return;
            }
            UseTree::Path(path) => path.ident.to_string(),
            UseTree::Name(name) => name.ident.to_string(),
            UseTree::Rename(rename) => rename.ident.to_string(),
            UseTree::Glob(_) => "*".to_owned(),
        };
        if parent == Some("crate") {
            self.require_dependency(&segment);
        }
        assert!(
            parent != Some("super") || segment != "super",
            "{} uses super::super",
            self.module
        );
        if let UseTree::Path(path) = tree {
            self.check_import(&path.tree, Some(&segment));
        }
    }
}

impl<'ast> Visit<'ast> for DependencyCheck<'_> {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.check_import(&item.tree, None);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let mut segments = path.segments.iter();
        if let (Some(first), Some(second)) = (segments.next(), segments.next()) {
            if first.ident == "crate" {
                self.require_dependency(&second.ident.to_string());
            }
            assert!(
                first.ident != "super" || second.ident != "super",
                "{} uses super::super",
                self.module
            );
        }
        syn::visit::visit_path(self, path);
    }
}
