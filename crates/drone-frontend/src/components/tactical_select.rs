//! # TacticalSelect
//!
//! A web-native dropdown in the HUD's own visual language. Deliberately NOT a
//! `<select>`: browsers hand that element to the OS, and on macOS you get a
//! Cocoa popup that ignores every CSS rule here. This is a button plus a
//! panel, fully styled, keyboard-navigable (Enter/Space open, ↑↓ move,
//! Enter selects, Esc closes), closes on outside click, and exposes the ARIA
//! listbox pattern so it is a real control, not a div that looks like one.
//!
//! Generic over the option key so the mission selector, and any later
//! dropdown, share one implementation. Same DOM and classes as the Leptos
//! version, so `main.css` is untouched.

use dioxus::prelude::*;
use wasm_bindgen::JsCast;

/// One selectable option: a stable key and its display label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectOption<K: Copy + PartialEq + 'static> {
    pub key: K,
    pub label: &'static str,
}

/// Visual accent for the control. Danger reads as a mode switch (red); accent
/// is the standard HUD green.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelectTone {
    #[default]
    Accent,
    Danger,
}

impl SelectTone {
    fn class(self) -> &'static str {
        match self {
            SelectTone::Accent => "tone-accent",
            SelectTone::Danger => "tone-danger",
        }
    }
}

#[component]
pub fn TacticalSelect<K: Copy + PartialEq + 'static>(
    /// Small caps label above the current value, e.g. "THEATER".
    label: String,
    /// The options, in display order.
    options: Vec<SelectOption<K>>,
    /// Currently selected key (owned by the caller's state).
    value: Signal<K>,
    #[props(default)] tone: SelectTone,
) -> Element {
    let mut value = value;
    let mut open = use_signal(|| false);
    // Keyboard cursor while open; -1 = follow the selected value.
    let mut cursor = use_signal(|| -1_i32);
    // Unique id per instance so the outside-click test can find its own root.
    let root_id = use_hook(|| format!("tsel-{}", uuid::Uuid::new_v4().simple()));

    let selected_index = {
        let options = options.clone();
        move || options.iter().position(|o| o.key == *value.peek()).map(|i| i as i32).unwrap_or(0)
    };
    let current_label = options.iter().find(|o| o.key == value()).map(|o| o.label).unwrap_or("—");

    // Close on outside click: a document-level listener installed once per
    // open, checking whether the event target lives inside our root node.
    {
        let root_id = root_id.clone();
        use_effect(move || {
            if !open() { return; }
            let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
            let root_id = root_id.clone();
            let handler = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                let inside = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                    .and_then(|n| {
                        web_sys::window()?.document()?.get_element_by_id(&root_id).map(|r| r.contains(Some(&n)))
                    })
                    .unwrap_or(false);
                if !inside {
                    open.set(false);
                    cursor.set(-1);
                }
            }) as Box<dyn Fn(web_sys::MouseEvent)>);
            let _ = document.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref());
            // Leaked per open cycle: the panel is short-lived and the listener
            // is idempotent (it only ever closes).
            handler.forget();
        });
    }

    let n = options.len() as i32;
    let mut choose = move |k: K| {
        value.set(k);
        open.set(false);
        cursor.set(-1);
    };

    let sel_idx_for_toggle = selected_index.clone();
    let on_toggle = move |_| {
        let now_open = !*open.peek();
        open.set(now_open);
        cursor.set(if now_open { sel_idx_for_toggle() } else { -1 });
    };

    let key_opts = options.clone();
    let sel_idx_for_key = selected_index.clone();
    let on_key = move |e: KeyboardEvent| {
        match e.key() {
            Key::Enter => {
                e.prevent_default();
                if *open.peek() {
                    let i = (*cursor.peek()).max(0) as usize;
                    if let Some(o) = key_opts.get(i) { choose(o.key); }
                } else {
                    open.set(true);
                    cursor.set(sel_idx_for_key());
                }
            }
            Key::Character(c) if c == " " => {
                e.prevent_default();
                if *open.peek() {
                    let i = (*cursor.peek()).max(0) as usize;
                    if let Some(o) = key_opts.get(i) { choose(o.key); }
                } else {
                    open.set(true);
                    cursor.set(sel_idx_for_key());
                }
            }
            Key::ArrowDown => {
                e.prevent_default();
                if !*open.peek() { open.set(true); cursor.set(sel_idx_for_key()); }
                else { let c = *cursor.peek(); cursor.set((c + 1).rem_euclid(n)); }
            }
            Key::ArrowUp => {
                e.prevent_default();
                if !*open.peek() { open.set(true); cursor.set(sel_idx_for_key()); }
                else { let c = *cursor.peek(); cursor.set((c - 1).rem_euclid(n)); }
            }
            Key::Escape => { open.set(false); cursor.set(-1); }
            _ => {}
        }
    };

    let is_open = open();
    let cur = cursor();
    let selected_key = value();

    rsx! {
        div { id: "{root_id}", class: "tsel {tone.class()}",
            span { class: "tsel-label", "{label}" }
            button {
                r#type: "button",
                class: "tsel-button",
                aria_haspopup: "listbox",
                aria_expanded: "{is_open}",
                onclick: on_toggle,
                onkeydown: on_key,
                span { class: "tsel-value", "{current_label}" }
                span { class: "tsel-caret", aria_hidden: "true",
                    svg { view_box: "0 0 10 6", width: "10", height: "6",
                        path { d: "M1 1 L5 5 L9 1", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round" }
                    }
                }
            }
            if is_open {
                ul { class: "tsel-panel", role: "listbox",
                    for (i, o) in options.iter().copied().enumerate() {
                        li {
                            key: "{i}",
                            class: if o.key == selected_key && cur == i as i32 { "tsel-option selected cursor" }
                                   else if o.key == selected_key { "tsel-option selected" }
                                   else if cur == i as i32 { "tsel-option cursor" }
                                   else { "tsel-option" },
                            role: "option",
                            aria_selected: "{o.key == selected_key}",
                            onmouseenter: move |_| cursor.set(i as i32),
                            onclick: move |_| choose(o.key),
                            span { class: "tsel-tick", aria_hidden: "true" }
                            "{o.label}"
                        }
                    }
                }
            }
        }
    }
}
