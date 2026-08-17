//! # Drone Card Component
//!
//! Individual drone status cards for the convoy assets panel.

use dioxus::prelude::*;

use crate::components::leaderboard::inline_svg;
use crate::state::{use_app_state, DroneState};

/// Same airframe SVG the map flies, compiled in — one asset, every surface.
const DRONE_SVG: &str = include_str!("../../../../assets/images/drone.svg");

/// Drone list panel
#[component]
pub fn DroneListPanel() -> Element {
    let state = use_app_state();
    let mut drones: Vec<DroneState> = state.drones.read().values().cloned().collect();
    drones.sort_by(|a, b| a.callsign.cmp(&b.callsign));
    let total = drones.len();
    let airborne = drones.iter().filter(|d| d.status.is_airborne()).count();

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                span { class: "panel-title", "CONVOY ASSETS" }
                span { class: "panel-badge", "{airborne}/{total} AIRBORNE" }
            }
            div { class: "panel-body", style: "display: flex; flex-direction: column; gap: 8px;",
                // Key includes updated_at so a changed row re-renders (the
                // frozen-card lesson): the server bumps it on every upsert.
                for drone in drones.into_iter() {
                    DroneCard { key: "{drone.drone_id}-{drone.updated_at}", drone: drone }
                }
            }
        }
    }
}

/// Individual drone card
#[component]
pub fn DroneCard(drone: DroneState) -> Element {
    let mut state = use_app_state();
    let drone_id = drone.drone_id;

    let is_selected = (state.selected_drone)() == Some(drone_id);
    let fuel_class = if drone.fuel_pct < 20.0 { "critical" } else if drone.fuel_pct < 40.0 { "warning" } else { "" };
    let progress_pct = (drone.current_waypoint as f32 / drone.total_waypoints.max(1) as f32) * 100.0;

    // One airframe silhouette for every platform, themed HUD-green via the
    // SVG's CSS custom properties (same mechanism as the map markers, where
    // the accent is red). Sized by the `.drone-icon svg` rule in main.css.
    let icon_html = format!(
        "<div style=\"--drone-accent: var(--accent-primary); --drone-edge: var(--accent-dim);\">{}</div>",
        inline_svg(DRONE_SVG)
    );

    // GPS: reactive on the smoothed live fix, so the digits glide between
    // server polls instead of stepping every 2 s. Falls back to the poll's
    // own position until the first fix lands.
    let live = state.live_positions.read().get(&drone_id).copied();
    let (lat, lon) = live.map(|p| (p.latitude, p.longitude)).unwrap_or((drone.position.latitude, drone.position.longitude));
    let gps_text = format!(
        "{:.4}°{} {:.4}°{}",
        lat.abs(), if lat >= 0.0 { "N" } else { "S" },
        lon.abs(), if lon >= 0.0 { "E" } else { "W" }
    );
    let gps_title = match live {
        Some(p) => format!("LIVE  ALT {:.0} m  HDG {:03.0}°", p.altitude_m, p.heading_deg),
        None => "awaiting fix".to_string(),
    };

    let card_class = if is_selected { "drone-card selected" } else { "drone-card" };
    let status_class = drone.status.status_class();
    let status_label = drone.status.as_str();
    let fuel_text = format!("{:.0}%", drone.fuel_pct);
    let acc_text = format!("{:.1}%", drone.accuracy_pct);

    rsx! {
        div {
            class: "{card_class}",
            onclick: move |_| {
                let cur = *state.selected_drone.peek();
                state.selected_drone.set(if cur == Some(drone_id) { None } else { Some(drone_id) });
            },
            div { class: "drone-icon", dangerous_inner_html: "{icon_html}" }
            div { class: "drone-details",
                div { class: "drone-callsign", "{drone.callsign}" }
                div { class: "drone-tail", "{drone.tail_number}" }
                div { class: "progress-bar", style: "margin-top: 4px;",
                    div { class: "progress-fill", style: "width: {progress_pct}%;" }
                }
                div { class: "text-xs text-muted", style: "margin-top: 2px;",
                    "WP {drone.current_waypoint}/{drone.total_waypoints}"
                }
                div { class: "drone-gps", title: "{gps_title}",
                    span { class: "gps-tag", "GPS" }
                    "{gps_text}"
                }
            }
            div { class: "drone-metrics",
                div { class: "status-badge {status_class}", "{status_label}" }
                div { class: "metric",
                    span { class: "metric-label", "FUEL" }
                    span { class: "metric-value {fuel_class}", "{fuel_text}" }
                }
                div { class: "metric",
                    span { class: "metric-label", "ACC" }
                    span { class: "metric-value text-accent", "{acc_text}" }
                }
            }
        }
    }
}
