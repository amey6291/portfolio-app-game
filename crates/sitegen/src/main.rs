//! Build-time generator: reads `src/data/*.json`, writes HTML and CSS to `dist/`.
//!
//! Usage: sitegen [ROOT] (defaults to the current directory)

mod data;
mod game;
mod levels;
mod resume;
mod stages;

use std::{fs, path::PathBuf};

pub const FAVICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Crect width='16' height='16' rx='3' fill='%230f1a2b'/%3E%3Crect x='3' y='3' width='4' height='4' fill='%2339ff7a'/%3E%3Crect x='9' y='3' width='4' height='4' fill='%232bb3a3'/%3E%3Crect x='3' y='9' width='10' height='4' fill='%23f28c28'/%3E%3C/svg%3E";

/// Stylesheets concatenated into dist/styles.css, in order.
const GAME_CSS: &[&str] = &["base.css", "hero.css", "start.css", "level1.css", "level2.css", "level3.css", "level4.css", "level5.css", "level6.css", "hq.css"];

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    let read = |p: &str| fs::read_to_string(root.join(p)).unwrap_or_else(|e| panic!("{p}: {e}"));

    let skills: data::Skills = serde_json::from_str(&read("src/data/skills.json")).expect("skills.json");
    let profile: data::Profile = serde_json::from_str(&read("src/data/profile.json")).expect("profile.json");
    levels::check(&skills);

    let dist = root.join("dist");
    fs::create_dir_all(dist.join("resume")).unwrap();

    let cx = game::Ctx { skills: &skills, profile: &profile };
    fs::write(dist.join("index.html"), game::page(&cx).into_string() + "\n").unwrap();
    fs::write(dist.join("resume/index.html"), resume::page(&skills, &profile).into_string() + "\n").unwrap();

    let css: String = GAME_CSS.iter().map(|f| read(&format!("assets/css/{f}")) + "\n").collect();
    fs::write(dist.join("styles.css"), minify_css(&css)).unwrap();
    fs::write(dist.join("resume/resume.css"), minify_css(&read("assets/css/resume.css"))).unwrap();
}

/// Conservative CSS minifier: drops comments, collapses whitespace and trims
/// it around braces and semicolons. Spaces inside values (calc, selectors)
/// survive as single spaces.
fn minify_css(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    out.push_str(rest);
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut min = String::with_capacity(collapsed.len());
    for c in collapsed.chars() {
        if matches!(c, '{' | '}' | ';') && min.ends_with(' ') {
            min.pop();
        }
        if c == ' ' && min.ends_with(['{', '}', ';']) {
            continue;
        }
        min.push(c);
    }
    min.replace(";}", "}") + "\n"
}
