//! Assembles the landing page for GitHub Pages: minifies `index.html` and copies
//! everything else from `web/` into `dist/` verbatim. Dependency-free (std only)
//! so it stays on-stack and adds nothing to the library's test build.
//!
//! Output goes to `dist/` (not `docs/`, which holds the README gallery samples).
//! The GitHub Pages workflow publishes `dist/`.
//!
//! Run with: `cargo run --example build_site` (after `cargo run --example site`
//! and the wasm-pack build have populated `web/`).

use std::fs;
use std::path::Path;

/// Collapse whitespace and drop comments in HTML markup, leaving the content of
/// `<script>` and `<style>` blocks untouched.
fn minify_html(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut index = 0;
    while index < html.len() {
        let next = ["<script", "<style"]
            .iter()
            .filter_map(|tag| lower[index..].find(tag).map(|position| index + position))
            .min();
        match next {
            Some(start) => {
                out.push_str(&collapse_markup(&html[index..start]));
                let close = if lower[start..].starts_with("<script") {
                    "</script>"
                } else {
                    "</style>"
                };
                let end = lower[start..]
                    .find(close)
                    .map(|position| start + position + close.len())
                    .unwrap_or(html.len());
                out.push_str(&html[start..end]);
                index = end;
            }
            None => {
                out.push_str(&collapse_markup(&html[index..]));
                break;
            }
        }
    }
    out
}

fn collapse_markup(segment: &str) -> String {
    // Strip `<!-- … -->` comments.
    let mut stripped = String::with_capacity(segment.len());
    let mut rest = segment;
    while let Some(start) = rest.find("<!--") {
        stripped.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => {
                rest = "";
                break;
            }
        }
    }
    stripped.push_str(rest);

    // Collapse every whitespace run to a single space.
    let mut out = String::with_capacity(stripped.len());
    let mut previous_was_space = false;
    for character in stripped.chars() {
        if character.is_whitespace() {
            if !previous_was_space {
                out.push(' ');
                previous_was_space = true;
            }
        } else {
            out.push(character);
            previous_was_space = false;
        }
    }
    out
}

fn copy_tree(source: &Path, destination: &Path, minified: &mut usize, copied: &mut usize) {
    for entry in fs::read_dir(source).expect("read source dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let name = entry.file_name();

        // Never carry wasm-pack's `.gitignore` (which would hide dist/ from git).
        if name == ".gitignore" {
            continue;
        }

        let target = destination.join(&name);
        if path.is_dir() {
            fs::create_dir_all(&target).expect("create dir");
            copy_tree(&path, &target, minified, copied);
        } else if name == "index.html" {
            let html = fs::read_to_string(&path).expect("read html");
            let small = minify_html(&html);
            fs::write(&target, &small).expect("write html");
            println!(
                "  minify {} : {} -> {} bytes",
                name.to_string_lossy(),
                html.len(),
                small.len()
            );
            *minified += 1;
        } else {
            fs::copy(&path, &target).expect("copy file");
            *copied += 1;
        }
    }
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("web");
    let output = root.join("dist");

    if output.exists() {
        fs::remove_dir_all(&output).expect("clean dist");
    }
    fs::create_dir_all(&output).expect("create dist");

    let mut minified = 0;
    let mut copied = 0;
    copy_tree(&source, &output, &mut minified, &mut copied);
    println!("built dist/: {minified} minified, {copied} copied verbatim");
}
