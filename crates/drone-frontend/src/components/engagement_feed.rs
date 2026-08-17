//! # Engagement Feed Component
//!
//! Real-time engagement event stream.

use dioxus::prelude::*;

use crate::state::{use_app_state, EngagementEvent};

/// Engagement feed panel
#[component]
pub fn EngagementFeedPanel() -> Element {
    let state = use_app_state();
    let events = state.engagements.read().clone();
    let hit_count = events.iter().filter(|e| e.hit).count();
    let total_count = events.len();

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                span { class: "panel-title", "ENGAGEMENT FEED" }
                span { class: "panel-badge", "{hit_count}/{total_count}" }
            }
            // overflow-y:hidden — .panel-body scrolls by default, and
            // .engagement-feed below has its own max-height scroller. Both
            // painting thumbs is the "double vertical slider". The feed list
            // is the ONLY scroller in this panel.
            div { class: "panel-body no-padding", style: "overflow-y: hidden;",
                div { class: "engagement-feed",
                    for event in events.iter().cloned() {
                        EngagementItem { key: "{event.id}", event: event }
                    }
                    if events.is_empty() {
                        div { style: "padding: 24px; text-align: center; color: var(--text-muted);",
                            "Awaiting engagement data..."
                        }
                    }
                }
            }
        }
    }
}

/// Single engagement event row
#[component]
fn EngagementItem(event: EngagementEvent) -> Element {
    let hit_class = if event.hit { "hit" } else { "miss" };
    let dot_class = if event.hit { "nominal" } else { "critical" };
    let result_text = if event.hit { "HIT" } else { "MISS" };
    let result_color = if event.hit { "var(--status-nominal)" } else { "var(--status-critical)" };
    let weapon_short: String = match event.weapon_type.as_str() {
        "AGM114_HELLFIRE" => "AGM-114".into(),
        "GBU12_PAVEWAY" => "GBU-12".into(),
        "AIM9X_SIDEWINDER" => "AIM-9X".into(),
        "GBU38_JDAM" => "GBU-38".into(),
        "AGM176_GRIFFIN" => "AGM-176".into(),
        other => other.to_string(),
    };
    let time_str = event.timestamp.format("%H:%M:%S").to_string();
    let acc = format!("{:.1}%", event.new_accuracy_pct);

    rsx! {
        div { class: "engagement-item {hit_class}",
            span { class: "status-dot {dot_class}" }
            div { class: "engagement-info",
                div { class: "engagement-callsign",
                    "{event.callsign} "
                    span { style: "color: {result_color};", "{result_text}" }
                }
                div { class: "engagement-weapon", "{weapon_short} → {acc}" }
            }
            div { class: "engagement-time", "{time_str}Z" }
        }
    }
}
