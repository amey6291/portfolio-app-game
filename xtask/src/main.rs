//! Project tasks.
//!
//!   cargo xtask build          sitegen + wasm engine -> dist/
//!   cargo xtask dev [PORT]     build, serve dist/, rebuild on changes
//!   cargo xtask serve [PORT]   serve dist/ as-is

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{exit, Command},
    thread,
    time::{Duration, SystemTime},
};

const WASM: &str = "target/wasm32-unknown-unknown/release/engine.wasm";

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    std::env::set_current_dir(&root).unwrap();
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    let port: u16 = args.next().and_then(|p| p.parse().ok()).unwrap_or(8080);
    match cmd.as_str() {
        "build" => {
            if !build() {
                exit(1)
            }
        }
        "dev" => {
            build();
            thread::spawn(watch);
            serve(port);
        }
        "serve" => serve(port),
        _ => {
            eprintln!("usage: cargo xtask <build|dev|serve> [port]");
            exit(2)
        }
    }
}

fn run(cmd: &mut Command) -> bool {
    match cmd.status() {
        Ok(s) if s.success() => true,
        Ok(_) => false,
        Err(e) => {
            eprintln!("xtask: {:?}: {e}", cmd.get_program());
            false
        }
    }
}

fn build() -> bool {
    let ok = run(Command::new("cargo").args(["run", "-q", "--release", "-p", "sitegen", "--", "."]))
        && run(Command::new("cargo").args([
            "build", "-q", "--release", "-p", "engine", "--target", "wasm32-unknown-unknown",
        ]))
        && run(Command::new("wasm-bindgen").args([
            WASM, "--target", "web", "--no-typescript", "--out-dir", "dist/pkg",
        ]))
        && optimize_wasm();
    if ok {
        let size = |p: &str| fs::metadata(p).map(|m| m.len() / 1024).unwrap_or(0);
        eprintln!(
            "xtask: built dist/ (index {} KB, css {} KB, wasm {} KB)",
            size("dist/index.html"),
            size("dist/styles.css"),
            size("dist/pkg/engine_bg.wasm")
        );
    } else {
        eprintln!("xtask: build failed");
    }
    ok
}

/// Shrinks the wasm with binaryen's wasm-opt when it's installed.
/// The feature flags match what rustc's wasm32 target emits by default.
fn optimize_wasm() -> bool {
    let out = "dist/pkg/engine_bg.wasm";
    let mut cmd = Command::new("wasm-opt");
    cmd.args(["-Oz", "--strip-debug", out, "-o", out]).args([
        "--enable-bulk-memory",
        "--enable-nontrapping-float-to-int",
        "--enable-sign-ext",
        "--enable-mutable-globals",
        "--enable-reference-types",
        "--enable-multivalue",
    ]);
    match cmd.status() {
        Ok(s) => s.success(),
        Err(_) => {
            eprintln!("xtask: wasm-opt not found, skipping wasm size optimization");
            true
        }
    }
}

/// Polls source mtimes and rebuilds on change.
fn watch() {
    fn newest(dir: &Path) -> SystemTime {
        let mut t = SystemTime::UNIX_EPOCH;
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            let m = if p.is_dir() { newest(&p) } else { e.metadata().and_then(|m| m.modified()).unwrap_or(t) };
            t = t.max(m);
        }
        t
    }
    let stamp = || ["src", "assets", "crates"].iter().map(|d| newest(Path::new(d))).max().unwrap();
    let mut last = stamp();
    loop {
        thread::sleep(Duration::from_millis(500));
        let now = stamp();
        if now > last {
            last = now;
            build();
        }
    }
}

fn serve(port: u16) {
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("xtask: port {port}: {e}");
        exit(1)
    });
    eprintln!("xtask: serving dist/ at http://127.0.0.1:{port}/");
    for stream in listener.incoming().flatten() {
        thread::spawn(|| handle(stream));
    }
}

fn handle(mut s: TcpStream) {
    let mut line = String::new();
    let mut r = BufReader::new(&s);
    if r.read_line(&mut line).is_err() {
        return;
    }
    // Drain headers.
    let mut h = String::new();
    while r.read_line(&mut h).map(|n| n > 2).unwrap_or(false) {
        h.clear();
    }
    let path = line.split_whitespace().nth(1).unwrap_or("/");
    let path = path.split(['?', '#']).next().unwrap();
    let mut file = PathBuf::from("dist");
    for seg in path.split('/').filter(|s| !s.is_empty() && *s != "..") {
        file.push(seg);
    }
    if file.is_dir() {
        if !path.ends_with('/') {
            let _ = write!(s, "HTTP/1.1 301 Moved\r\nLocation: {path}/\r\nContent-Length: 0\r\n\r\n");
            return;
        }
        file.push("index.html");
    }
    let (status, body) = match fs::read(&file) {
        Ok(b) => ("200 OK", b),
        Err(_) => ("404 Not Found", b"not found".to_vec()),
    };
    let mime = match file.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };
    let _ = write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = s.write_all(&body);
}
