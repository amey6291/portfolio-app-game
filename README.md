# Portfolio: a scroll-driven side-scroller

My portfolio as a platformer. Scrolling down walks the hero to the right through
seven levels, from mainframe to cloud to AI, collecting skills as power-ups.
Visitors who prefer reduced motion get a static page, and `/resume/` is a
printable resume built from the same data.

Built in Rust: a build-time generator writes the HTML, and a small
WebAssembly engine runs the scrolling. The output is plain static files in
`dist/` that any static host can serve.

## Editing content

| What | Where |
| --- | --- |
| Skills, tiers, proof points, certifications, headline numbers | `src/data/skills.json` |
| Name, title, jobs, intro, contact links | `src/data/profile.json` |
| Which category and achievement goes in which level | `crates/sitegen/src/levels.rs` |

The build fails if a category or headline number is not placed in exactly
one level.

## Build and run

Needs Rust (stable), the wasm target and the matching wasm-bindgen CLI.
`wasm-opt` (binaryen) is optional and shrinks the wasm by about a third.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.100 --locked

cargo xtask build        # -> dist/
cargo xtask dev          # build, serve on http://127.0.0.1:8080, rebuild on change
cargo xtask serve        # serve dist/ as-is
```

Checks (Python 3 with Playwright, using an installed Chrome):

```sh
python3 tests/coverage.py              # every skill/cert/number appears on both pages
python tests/shots.py                  # screenshots at desktop and phone width
python tests/shots.py "" "#level-3"    # screenshots across a single level
```

## Hosting

`dist/` is self-contained and uses relative paths, so it works at a domain
root or under a sub-path such as GitHub Pages' `/<repo>/`.

- **GitHub Pages:** `.github/workflows/pages.yml` builds and deploys on every
  push to `main`. One-time setup: repo Settings → Pages → Source:
  "GitHub Actions". The site appears at `https://<user>.github.io/<repo>/`, or
  at `https://<user>.github.io/` if the repo is named `<user>.github.io`.
- **Netlify, Cloudflare Pages:** build command `cargo xtask build`, publish
  directory `dist`. Or build locally and upload `dist/`.
- **S3 + CloudFront:** upload `dist/`. `engine_bg.wasm` must be served as
  `application/wasm`.
- **Docker:** `docker build -t portfolio . && docker run -p 8080:80 portfolio`
  serves it with static-web-server, with compression on.

## How it works

- `crates/sitegen`: reads the JSON and writes `dist/index.html`,
  `dist/resume/index.html` and the CSS (concatenated and minified from
  `assets/css/`). All text is real HTML.
- `crates/engine`: wasm-bindgen + web-sys. The body gets a spacer as tall as
  the world is wide, and each frame the world is translated by `-scrollY`. The
  hero stays at a fixed screen x, so everything else (reveals, toasts, HUD,
  jumps, parallax and set-piece progress) follows from the hero's position in
  the world. Scrolling backwards needs no extra handling. The markup hooks are
  listed at the top of `crates/sitegen/src/game.rs`.
- `xtask`: the build, the dev server and the file watcher, using only the
  standard library.
- **Reduced motion and no-JS:** an inline script adds `class="game"` only when
  motion is allowed. Without it, the same HTML lays out as a stacked page, and
  set pieces (boss, machines, incident) show their finished state.
