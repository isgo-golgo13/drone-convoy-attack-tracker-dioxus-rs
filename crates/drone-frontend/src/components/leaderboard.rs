//! # Leaderboard Component
//!
//! Real-time accuracy rankings display.

use dioxus::prelude::*;

use crate::state::{use_app_state, LeaderboardEntry};

/// Streak marker: target roundel with crosshair ticks, stroke-only in
/// currentColor so it themes with the row (accent-green via the wrapping
/// span). Lives in assets/images/ beside drone.svg — one directory holds
/// every piece of artwork, compiled in at build time.
const TARGET_SVG: &str = include_str!("../../../../assets/images/target-streak.svg");

/// Strip the XML prolog: valid in a standalone file, invalid inside innerHTML.
pub(crate) fn inline_svg(svg: &str) -> &str {
    svg.find("<svg").map_or(svg, |i| &svg[i..])
}

/// Leaderboard panel component
#[component]
pub fn LeaderboardPanel() -> Element {
    let state = use_app_state();
    let entries = state.leaderboard.read().clone();
    let total = entries.len();

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                span { class: "panel-title", "ACCURACY LEADERBOARD" }
                span { class: "panel-badge", "{total}" }
            }
            div { class: "panel-body no-padding",
                div { class: "leaderboard",
                    // An empty result renders an explicit empty state, so a
                    // fresh database is distinguishable from a broken query.
                    if entries.is_empty() {
                        div { class: "leaderboard-entry", style: "justify-content: center;",
                            span { class: "text-xs text-muted uppercase tracking-wide", "NO ENGAGEMENTS RECORDED" }
                        }
                    }
                    // Composite key: keyed by drone_id alone, each row's view
                    // froze at first render (the "UI leaderboard doesn't match
                    // the simulator's final tally" report). Any change to the
                    // shot record, streak or rank mints a new key.
                    for entry in entries.into_iter() {
                        LeaderboardRow {
                            key: "{entry.drone_id}-{entry.total_engagements}-{entry.successful_hits}-{entry.current_streak}-{entry.rank}",
                            entry: entry,
                        }
                    }
                }
            }
        }
    }
}

/// Single leaderboard row
#[component]
fn LeaderboardRow(entry: LeaderboardEntry) -> Element {
    let rank_class = match entry.rank {
        1 => "rank-1",
        2 => "rank-2",
        3 => "rank-3",
        _ => "",
    };
    let platform_short: String = match entry.platform_type.as_str() {
        "MQ9_REAPER" => "MQ-9".to_string(),
        "MQ1C_GRAY_EAGLE" => "MQ-1C".to_string(),
        "RQ4_GLOBAL_HAWK" => "RQ-4".to_string(),
        "MQ25_STINGRAY" => "MQ-25".to_string(),
        other => other.to_string(),
    };
    let target = inline_svg(TARGET_SVG).to_string();

    rsx! {
        div { class: "leaderboard-entry {rank_class}",
            div { class: "leaderboard-rank", "{entry.rank}" }
            div { class: "leaderboard-info",
                div { class: "leaderboard-callsign",
                    "{entry.callsign}"
                    if entry.rank_change > 0 {
                        span { class: "rank-change up", "▲{entry.rank_change}" }
                    } else if entry.rank_change < 0 {
                        span { class: "rank-change down", "▼{entry.rank_change.abs()}" }
                    }
                }
                div { class: "leaderboard-platform", "{platform_short}" }
            }
            div { class: "leaderboard-stats",
                div { class: "leaderboard-accuracy", {format!("{:.1}%", entry.accuracy_pct)} }
                div { class: "leaderboard-record",
                    "{entry.successful_hits}/{entry.total_engagements}"
                    " • "
                    span {
                        style: "display:inline-block; vertical-align:-1px; color: var(--accent-primary);",
                        dangerous_inner_html: "{target}",
                    }
                    "{entry.current_streak}"
                }
            }
        }
    }
}
