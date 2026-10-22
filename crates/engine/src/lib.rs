//! Scroll-driven side-scroller engine.
//!
//! Vertical scroll distance maps 1:1 to horizontal world distance: the page
//! body gets a spacer `worldWidth - vw + vh` tall, and each frame the world
//! strip is translated by `-scrollY`. The hero stays at a fixed screen x, so
//! its world position is `scrollY + heroX`. Everything else (reveals, toasts,
//! HUD, jumps, parallax, set-piece progress) is a pure function of that
//! position, which makes scrolling backwards free.

use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Document, Element, HtmlElement, HtmlTemplateElement, Window};

/// How far (as a fraction of the viewport) ahead of the left edge things reveal.
const REVEAL_AT: f64 = 0.86;
/// Scroll idle time before the walk cycle stops, in ms.
const WALK_IDLE_MS: f64 = 140.0;
const JUMP_HEIGHT: f64 = 120.0;
/// Float levels lift the hero by this fraction of the viewport height
/// (higher on phones, where power-ups stack in two rows).
const FLOAT_HEIGHT: f64 = 0.32;
const FLOAT_HEIGHT_NARROW: f64 = 0.55;
const TOAST_MS: i32 = 4200;

struct Marker {
    el: HtmlElement,
    x: f64,
    w: f64,
    done: bool,
}

struct Level {
    x: f64,
    w: f64,
    name: String,
    /// Classes the hero wears in this level (`data-hero`), e.g. "ride".
    hero: String,
    layers: Vec<(HtmlElement, f64)>,
    decor: Vec<HtmlTemplateElement>,
}

struct Game {
    win: Window,
    doc: Document,
    world: HtmlElement,
    spacer: HtmlElement,
    hero: HtmlElement,
    hud_level: Element,
    hud_bar: HtmlElement,
    hud_track: Element,
    hud_years: Element,
    toasts: Element,

    levels: Vec<Level>,
    reveals: Vec<Marker>,
    toast_marks: Vec<Marker>,
    jumps: Vec<Marker>,
    progress: Vec<Marker>,
    countdowns: Vec<HtmlElement>,
    years: Vec<(f64, f64)>,

    vw: f64,
    vh: f64,
    hero_x: f64,
    max_scroll: f64,
    last_scroll: f64,
    last_move: f64,
    walking: bool,
    facing_left: bool,
    level: usize,
    years_text: String,
    dirty: bool,
}

fn html_el(doc: &Document, sel: &str) -> Result<HtmlElement, JsValue> {
    doc.query_selector(sel)?
        .ok_or_else(|| JsValue::from_str(&format!("missing {sel}")))?
        .dyn_into()
        .map_err(|_| JsValue::from_str("not an HtmlElement"))
}

fn all(root: &Element, sel: &str) -> Vec<HtmlElement> {
    let Ok(list) = root.query_selector_all(sel) else { return vec![] };
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into::<HtmlElement>().ok())
        .collect()
}

impl Game {
    fn new(win: Window, doc: Document) -> Result<Self, JsValue> {
        Ok(Game {
            world: html_el(&doc, "#world")?,
            spacer: html_el(&doc, "#spacer")?,
            hero: html_el(&doc, "#hero")?,
            hud_level: html_el(&doc, "#hud-level")?.into(),
            hud_bar: html_el(&doc, "#hud-bar")?,
            hud_track: html_el(&doc, "#hud-track")?.into(),
            hud_years: html_el(&doc, "#hud-years")?.into(),
            toasts: html_el(&doc, "#toasts")?.into(),
            win,
            doc,
            levels: vec![],
            reveals: vec![],
            toast_marks: vec![],
            jumps: vec![],
            progress: vec![],
            countdowns: vec![],
            years: vec![],
            vw: 0.0,
            vh: 0.0,
            hero_x: 0.0,
            max_scroll: 0.0,
            last_scroll: f64::NAN,
            last_move: f64::NEG_INFINITY,
            walking: false,
            facing_left: false,
            level: usize::MAX,
            years_text: String::new(),
            dirty: true,
        })
    }

    /// Measures the world. Safe to call repeatedly (resize, font load);
    /// one-shot state such as fired toasts survives.
    fn layout(&mut self) {
        let win = &self.win;
        self.vw = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1024.0);
        let vh = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(768.0);
        self.vh = vh;

        // Positions are measured relative to the world's own box, so the
        // current translate cancels out.
        let origin = self.world.get_bounding_client_rect().left();
        let span = |el: &Element| {
            let r = el.get_bounding_client_rect();
            (r.left() - origin, r.width())
        };
        let world_w = self.world.scroll_width() as f64;
        self.max_scroll = (world_w - self.vw).max(0.0);
        let _ = self
            .spacer
            .style()
            .set_property("height", &format!("{}px", self.max_scroll + vh));

        let hr = self.hero.get_bounding_client_rect();
        self.hero_x = hr.left() + hr.width() / 2.0;

        let world: &Element = &self.world;
        self.levels = all(world, ".level")
            .into_iter()
            .map(|el| {
                let (x, w) = span(&el);
                let layers = all(&el, "[data-depth]")
                    .into_iter()
                    .map(|l| {
                        let d = l.dataset().get("depth").and_then(|d| d.parse().ok()).unwrap_or(0.0);
                        (l, d)
                    })
                    .collect();
                let decor = all(&el, "template[data-decor]")
                    .into_iter()
                    .filter_map(|t| t.dyn_into().ok())
                    .collect();
                let d = el.dataset();
                Level {
                    x,
                    w,
                    name: d.get("name").unwrap_or_default(),
                    hero: d.get("hero").unwrap_or_default(),
                    layers,
                    decor,
                }
            })
            .collect();

        let remeasure = |old: &mut Vec<Marker>, sel: &str| {
            let done: Vec<bool> = old.iter().map(|m| m.done).collect();
            *old = all(world, sel)
                .into_iter()
                .enumerate()
                .map(|(i, el)| {
                    let (x, w) = span(&el);
                    Marker { el, x, w, done: done.get(i).copied().unwrap_or(false) }
                })
                .collect();
        };
        remeasure(&mut self.reveals, "[data-reveal]");
        remeasure(&mut self.toast_marks, "[data-toast]");
        remeasure(&mut self.jumps, "[data-jump]");
        remeasure(&mut self.progress, "[data-progress]");
        self.countdowns = all(world, "[data-countdown]");

        self.years = all(world, "[data-year]")
            .into_iter()
            .filter_map(|el| {
                let y: f64 = el.dataset().get("year")?.parse().ok()?;
                Some((span(&el).0, y))
            })
            .collect();
        self.years.sort_by(|a, b| a.0.total_cmp(&b.0));
        self.dirty = true;
    }

    fn frame(&mut self, now: f64) {
        let scroll = self.win.scroll_y().unwrap_or(0.0).clamp(0.0, self.max_scroll);
        let moved = (scroll - self.last_scroll).abs() > 0.5 || self.last_scroll.is_nan();
        if moved {
            if !self.last_scroll.is_nan() {
                self.last_move = now;
                self.set_facing(scroll < self.last_scroll);
            }
            self.last_scroll = scroll;
        }
        self.set_walking(now - self.last_move < WALK_IDLE_MS);
        if !moved && !self.dirty {
            return;
        }
        self.dirty = false;

        let _ = self
            .world
            .style()
            .set_property("transform", &format!("translate3d({}px,0,0)", -scroll.round()));

        let hero_wx = scroll + self.hero_x;
        self.update_levels(scroll, hero_wx);
        self.update_lift(hero_wx);
        self.update_progress(scroll, hero_wx);

        let edge = scroll + self.vw * REVEAL_AT;
        for m in self.reveals.iter_mut().filter(|m| !m.done && m.x < edge) {
            m.done = true;
            let _ = m.el.class_list().add_1("on");
        }

        let mut fire = vec![];
        for m in self.toast_marks.iter_mut().filter(|m| !m.done && m.x <= hero_wx) {
            m.done = true;
            let d = m.el.dataset();
            fire.push((d.get("toastTitle").unwrap_or_else(|| "Achievement unlocked".into()), d.get("toast").unwrap_or_default()));
        }
        for (title, text) in fire {
            self.toast(&title, &text);
        }

        let pct = if self.max_scroll > 0.0 { scroll / self.max_scroll * 100.0 } else { 0.0 };
        let _ = self.hud_bar.style().set_property("transform", &format!("scaleX({:.4})", pct / 100.0));
        let _ = self.hud_track.set_attribute("aria-valuenow", &format!("{}", pct.round()));
        self.update_years(hero_wx);
    }

    /// Vertical hero offset: jumps over obstacles, plus the float levels'
    /// rise and descent.
    fn update_lift(&self, hero_wx: f64) {
        let jump = self
            .jumps
            .iter()
            .filter_map(|m| {
                // Take off half a body-length before the obstacle, land after it.
                let half = m.w / 2.0 + 70.0;
                let t = (hero_wx - (m.x + m.w / 2.0)) / half;
                (t.abs() < 1.0).then(|| JUMP_HEIGHT * (1.0 - t * t))
            })
            .fold(0.0, f64::max);
        let float = self
            .levels
            .get(self.level)
            .filter(|l| l.hero.split(' ').any(|c| c == "float"))
            .map_or(0.0, |l| {
                let t = (hero_wx - l.x) / l.w;
                let h = if self.vw < 700.0 { FLOAT_HEIGHT_NARROW } else { FLOAT_HEIGHT };
                self.vh * h * smoothstep(0.0, 0.12, t) * (1.0 - smoothstep(0.86, 0.98, t))
            });
        let lift = jump.max(float);
        let _ = self
            .hero
            .style()
            .set_property("transform", &format!("translate3d(0,{}px,0)", -lift.round()));
        let _ = self.hero.class_list().toggle_with_force("jumping", jump > 4.0);
    }

    /// Set pieces read `--p`: 0 as the element enters the screen, 1 when the
    /// hero reaches its left edge (so the finished state is seen in full).
    /// `data-countdown` elements count 9..0 over the last screen of the journey.
    fn update_progress(&self, scroll: f64, hero_wx: f64) {
        let lead = self.vw * 0.6;
        for m in &self.progress {
            if m.x + m.w < scroll - self.vw || m.x > scroll + 2.0 * self.vw {
                continue;
            }
            let p = ((hero_wx - (m.x - lead)) / lead).clamp(0.0, 1.0);
            let _ = m.el.style().set_property("--p", &format!("{p:.3}"));
        }
        let left = ((self.max_scroll - scroll) / self.vw).clamp(0.0, 1.0);
        for el in &self.countdowns {
            el.set_text_content(Some(&format!("{}", (left * 9.0).ceil())));
        }
    }

    fn update_levels(&mut self, scroll: f64, hero_wx: f64) {
        let current = self.levels.iter().rposition(|l| l.x <= hero_wx).unwrap_or(0);
        if current != self.level {
            self.level = current;
            if let Some(l) = self.levels.get(current) {
                self.hud_level.set_text_content(Some(&l.name));
                let list = self.hero.class_list();
                for old in self.levels.iter().flat_map(|l| l.hero.split_whitespace()) {
                    let _ = list.remove_1(old);
                }
                for c in l.hero.split_whitespace() {
                    let _ = list.add_1(c);
                }
            }
        }
        let view = (scroll - self.vw, scroll + 2.0 * self.vw);
        for l in &mut self.levels {
            // Lazy decor: inflate templates a screen before the level arrives.
            if l.x < scroll + 2.0 * self.vw && !l.decor.is_empty() {
                for t in l.decor.drain(..) {
                    if let Some(parent) = t.parent_node() {
                        let _ = parent.append_child(&t.content().clone_node_with_deep(true).unwrap());
                    }
                }
            }
            if l.x + l.w < view.0 || l.x > view.1 {
                continue;
            }
            for (el, depth) in &l.layers {
                let dx = (scroll - l.x) * depth;
                let _ = el.style().set_property("transform", &format!("translate3d({}px,0,0)", dx.round()));
            }
        }
    }

    fn update_years(&mut self, hero_wx: f64) {
        let y = match self.years.iter().position(|&(x, _)| x > hero_wx) {
            None => self.years.last().map_or(0.0, |a| a.1),
            Some(0) => 0.0,
            Some(i) => {
                let (x0, y0) = self.years[i - 1];
                let (x1, y1) = self.years[i];
                y0 + (y1 - y0) * ((hero_wx - x0) / (x1 - x0))
            }
        };
        let text = format!("{y:.1}");
        if text != self.years_text {
            self.hud_years.set_text_content(Some(&text));
            self.years_text = text;
        }
    }

    fn set_walking(&mut self, on: bool) {
        if on != self.walking {
            self.walking = on;
            let _ = self.hero.class_list().toggle_with_force("walking", on);
        }
    }

    fn set_facing(&mut self, left: bool) {
        if left != self.facing_left {
            self.facing_left = left;
            let _ = self.hero.class_list().toggle_with_force("left", left);
        }
    }

    fn toast(&self, title: &str, text: &str) {
        let Ok(el) = self.doc.create_element("div") else { return };
        el.set_class_name("toast");
        let (Ok(t), Ok(p)) = (self.doc.create_element("strong"), self.doc.create_element("span")) else { return };
        t.set_text_content(Some(title));
        p.set_text_content(Some(text));
        let _ = el.append_with_node_2(&t, &p);
        let _ = self.toasts.append_child(&el);

        let gone = el.clone();
        let cb = Closure::once_into_js(move || gone.remove());
        let _ = self
            .win
            .set_timeout_with_callback_and_timeout_and_arguments_0(cb.unchecked_ref(), TOAST_MS);
    }

    /// Keyboard focus inside the fixed viewport would scroll it sideways;
    /// instead, scroll the page so the focused element is on screen.
    fn follow_focus(&self, target: &Element) {
        let origin = self.world.get_bounding_client_rect().left();
        let x = target.get_bounding_client_rect().left() - origin;
        let opts = web_sys::ScrollToOptions::new();
        opts.set_top((x - self.vw * 0.4).clamp(0.0, self.max_scroll));
        opts.set_behavior(web_sys::ScrollBehavior::Instant);
        self.win.scroll_to_with_scroll_to_options(&opts);
        if let Ok(vp) = html_el(&self.doc, "#viewport") {
            vp.set_scroll_left(0);
        }
    }
}

fn smoothstep(a: f64, b: f64, t: f64) -> f64 {
    let x = ((t - a) / (b - a)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let win = web_sys::window().ok_or("no window")?;
    let doc = win.document().ok_or("no document")?;
    let root = doc.document_element().ok_or("no root")?;
    if !root.class_list().contains("game") {
        return Ok(()); // reduced motion: static page
    }

    let game = Rc::new(RefCell::new(Game::new(win.clone(), doc.clone())?));
    game.borrow_mut().layout();

    let relayout = {
        let g = game.clone();
        Closure::<dyn FnMut()>::new(move || g.borrow_mut().layout())
    };
    win.add_event_listener_with_callback("resize", relayout.as_ref().unchecked_ref())?;
    win.add_event_listener_with_callback("load", relayout.as_ref().unchecked_ref())?;
    relayout.forget();

    let on_key = {
        let w = win.clone();
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
            let step = w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(800.0) * 0.5;
            let dy = match e.key().as_str() {
                "ArrowRight" => step,
                "ArrowLeft" => -step,
                _ => return,
            };
            e.prevent_default();
            w.scroll_by_with_x_and_y(0.0, dy);
        })
    };
    win.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref())?;
    on_key.forget();

    let on_focus = {
        let g = game.clone();
        Closure::<dyn FnMut(web_sys::FocusEvent)>::new(move |e: web_sys::FocusEvent| {
            if let Some(t) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) {
                if t.closest("#world").ok().flatten().is_some() {
                    g.borrow().follow_focus(&t);
                }
            }
        })
    };
    doc.add_event_listener_with_callback("focusin", on_focus.as_ref().unchecked_ref())?;
    on_focus.forget();

    // One rAF loop; frames with no scroll change exit early.
    let tick: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let tick2 = tick.clone();
    let w2 = win.clone();
    *tick2.borrow_mut() = Some(Closure::new(move |t: f64| {
        game.borrow_mut().frame(t);
        let _ = w2.request_animation_frame(tick.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }));
    win.request_animation_frame(tick2.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    root.class_list().add_1("ready")?;
    Ok(())
}
