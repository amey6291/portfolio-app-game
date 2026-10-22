//! The scroll-driven game page (`dist/index.html`): page shell, HUD, hero
//! and the building blocks the levels in `stages.rs` are made of.
//!
//! Everything here is plain HTML. Without the `game` class on <html> (no JS,
//! or prefers-reduced-motion) it renders as a static, stacked page; the wasm
//! engine turns it into the side-scroller. Engine hooks:
//!   data-reveal          pops in when it scrolls into view
//!   data-year            anchor for the HUD years counter
//!   data-jump            the hero jumps over it
//!   data-toast[-title]   one-shot toast when the hero passes it
//!   data-depth           parallax factor for a background layer
//!   data-progress        gets `--p` (0..1) as the hero approaches it
//!   data-countdown       text counts 9..0 over the last screen
//!   data-hero (level)    classes the hero wears there: ride, float, drone
//!   template[data-decor] decorative SVG, inflated just before it's needed
//! CSS reads `--p` with a fallback of 1, so the static page shows set pieces
//! in their finished state.

use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::data::{current_month_index, month_index, Category, Job, Profile, Skills};
use crate::stages;

pub struct Ctx<'a> {
    pub skills: &'a Skills,
    pub profile: &'a Profile,
}

impl Ctx<'_> {
    /// Years from career start to a "YYYY-MM" date, as a HUD anchor value.
    pub fn years_at(&self, ym: &str) -> String {
        self.years_since_start(month_index(ym))
    }

    pub fn years_now(&self) -> String {
        self.years_since_start(current_month_index())
    }

    fn years_since_start(&self, month: i32) -> String {
        let m = month - month_index(&self.profile.career_start);
        format!("{:.2}", m as f64 / 12.0)
    }

    pub fn headline(&self, i: usize) -> &str {
        &self.skills.headline_numbers[i]
    }
}

pub fn page(cx: &Ctx) -> Markup {
    let p = cx.profile;
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (p.name) " · Portfolio" }
                meta name="description" content=(format!("{}, {}. {}", p.name, p.title, p.intro));
                meta name="theme-color" content="#0f1a2b";
                link rel="icon" href=(crate::FAVICON);
                link rel="stylesheet" href="styles.css";
                // Decide before first paint so the static layout never flashes.
                script { (PreEscaped("if(!matchMedia('(prefers-reduced-motion: reduce)').matches)document.documentElement.classList.add('game')")) }
                link rel="modulepreload" href="pkg/engine.js";
            }
            body {
                a.skip-link href="resume/" { "Skip to plain resume" }
                (hud())
                div #toasts aria-live="polite" {}
                main #viewport {
                    div #world {
                        (start(cx))
                        (stages::all(cx))
                    }
                }
                div #hero aria-hidden="true" { (hero()) }
                div #spacer aria-hidden="true" {}
                script type="module" {
                    (PreEscaped("import init from './pkg/engine.js';init().catch(e=>{console.error(e);document.documentElement.classList.remove('game')})"))
                }
            }
        }
    }
}

fn hud() -> Markup {
    html! {
        header #hud {
            div.hud-level {
                span.k { "Level" }
                span #hud-level { "Start" }
            }
            div.hud-progress {
                div.track #hud-track role="progressbar" aria-label="Journey progress"
                    aria-valuemin="0" aria-valuemax="100" aria-valuenow="0" {
                    div #hud-bar {}
                }
            }
            div.hud-years {
                span.k { "Exp" }
                span #hud-years { "0.0" }
                span.k { "yrs" }
            }
            a.hud-resume href="resume/" { "Resume" }
        }
    }
}

fn start(cx: &Ctx) -> Markup {
    let p = cx.profile;
    html! {
        section.level.start #start data-name="Start" aria-label="Start screen" {
            div.bg aria-hidden="true" {
                div.layer.stars data-depth="0.7" {}
                div.layer.skyline data-depth="0.4" {}
            }
            div.stage {
                div.start-card {
                    p.kicker { "Player 1" }
                    h1 { (p.name) }
                    p.role { (p.title) }
                    p.meta { (p.location) " · " (p.experience) " · " (p.domain) }
                    p.intro { (p.intro) }
                    div.start-actions {
                        p.hint { span.arrow aria-hidden="true" { "↓" } " Scroll to start" }
                        a.btn href="resume/" { "Skip to plain resume" }
                    }
                }
            }
            div.ground aria-hidden="true" {}
        }
    }
}

pub fn level_card(num: &str, name: &str, dates: &str) -> Markup {
    html! {
        div.beat.level-card data-reveal {
            p.num { (num) }
            h2 { (name) }
            p.dates { (dates) }
        }
    }
}

pub fn job_sign(cx: &Ctx, n: usize, job: &Job) -> Markup {
    html! {
        article.beat.sign data-reveal data-year=(cx.years_at(&job.start)) {
            div.panel {
                div.screen {
                    p.prompt aria-hidden="true" { "> LOAD JOB_0" (n) ".DAT" }
                    h3 { (job.role) }
                    p.company { (job.company) }
                    p.dates { (job.dates()) }
                    p.summary { (job.summary) }
                }
            }
            div.post aria-hidden="true" {}
        }
    }
}

/// A category as a row of power-ups. Each level styles `.chip` its own way.
pub fn skill_cluster(label: &str, cat: &Category) -> Markup {
    html! {
        section.beat.cluster aria-label=(cat.category) {
            h3.cluster-head data-reveal {
                span.tag { (label) }
                (cat.category)
            }
            ul.items {
                @for s in &cat.skills {
                    li class=(format!("item {}", s.tier.class())) style=(format!("--pow:{}", s.tier.power())) data-reveal {
                        @if let Some(proof) = &s.proof {
                            p.bubble { (proof) }
                        }
                        span.chip aria-hidden="true" {}
                        span.name { (s.skill) }
                        span.bar aria-hidden="true" { i {} }
                        span.tier { (s.tier.label()) }
                    }
                }
            }
        }
    }
}

/// A headline number as an in-world plaque that also fires a toast.
pub fn achievement(cx: &Ctx, i: usize) -> Markup {
    let text = cx.headline(i);
    html! {
        div.beat.ach data-reveal data-toast=(text) {
            span.ach-icon aria-hidden="true" { "★" }
            p.k { "Achievement unlocked" }
            p.ach-text { (text) }
        }
    }
}

/// The hero: an engineer with a lanyard badge and backpack, built from
/// parts the CSS animates (idle, walk, jump), plus balloons for the float
/// level and a helper drone that joins late in the game.
fn hero() -> Markup {
    PreEscaped(r##"<svg class="hero-svg" viewBox="0 0 64 100" width="64" height="100">
<g class="balloons"><path d="M38 18 L20 -12 M38 18 L36 -24 M38 18 L52 -10" stroke="#f4f1e8" stroke-width="1"/><circle cx="20" cy="-22" r="11" fill="#ff7a59"/><circle cx="36" cy="-36" r="12" fill="#2bb3a3"/><circle cx="52" cy="-20" r="11" fill="#ffd166"/><circle cx="16" cy="-26" r="3" fill="#fff" opacity=".6"/><circle cx="32" cy="-41" r="3" fill="#fff" opacity=".6"/></g>
<g class="body">
<rect x="10" y="38" width="13" height="26" rx="4" fill="#f28c28"/>
<g class="leg leg-b"><rect x="26" y="62" width="9" height="30" rx="4" fill="#1b2740"/><rect x="25" y="88" width="15" height="7" rx="3" fill="#0d0f14"/></g>
<g class="arm arm-b"><rect x="29" y="40" width="8" height="25" rx="4" fill="#23998b"/><circle cx="33" cy="66" r="4" fill="#9a6244"/></g>
<rect x="20" y="36" width="24" height="31" rx="7" fill="#2bb3a3"/>
<path d="M27 37 L32 50 L37 37" fill="none" stroke="#f4f1e8" stroke-width="1.6"/>
<rect x="29" y="49" width="7" height="9" rx="1.5" fill="#f4f1e8"/>
<g class="leg leg-f"><rect x="28" y="62" width="9" height="30" rx="4" fill="#24324f"/><rect x="28" y="88" width="15" height="7" rx="3" fill="#13161d"/></g>
<g class="head"><circle cx="34" cy="21" r="13" fill="#b97a56"/><path d="M21 20 C21 9 31 5 38 7 C45 8 48 13 47 18 C43 14 37 13 33 14 C29 15 25 18 21 20Z" fill="#2a1d17"/><circle cx="40" cy="21" r="1.7" fill="#1a1412"/><path d="M38 27 Q41 29 44 26" fill="none" stroke="#5c3424" stroke-width="1.4" stroke-linecap="round"/></g>
<g class="arm arm-f"><rect x="30" y="40" width="8" height="25" rx="4" fill="#2bb3a3"/><circle cx="34" cy="66" r="4" fill="#b97a56"/></g>
</g>
<g class="drone"><path d="M56 -14 h22 M60 -14 v4 M74 -14 v4" stroke="#c9d3e6" stroke-width="2" stroke-linecap="round"/><rect x="58" y="-10" width="18" height="11" rx="5" fill="#e9ecf5"/><circle cx="70" cy="-4.5" r="2.6" fill="#6ef3ff"/></g>
</svg>"##.to_string())
}
