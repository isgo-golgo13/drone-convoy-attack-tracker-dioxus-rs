//! # Drone Convoy Tracker Frontend (Dioxus)
//!
//! Tactical HUD for military drone convoy tracking and leaderboard display.
//!
//! Every panel is live. There is no seed data in this crate: the leaderboard,
//! drone cards, convoy status, engagement feed, telemetry chart and the map's
//! airframes are all driven by the 2-second poll below against the GraphQL
//! API, which reads ScyllaDB. An empty panel means an empty table.
//!
//! This is the Dioxus 0.7 port of the Leptos frontend: same DOM, same
//! `main.css`, same assets, same GraphQL contract. If a pixel differs from
//! the Leptos build at the same viewport, that is a bug.

#![forbid(unsafe_code)]
#![warn(clippy::all)]

pub mod components;
pub mod services;
pub mod state;

use std::collections::HashMap;

use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;

use components::*;
use state::*;

/// Poll interval for the live feed.
const POLL_INTERVAL_MS: u64 = 2_000;

/// The well-known demo convoy. The simulator pins its writes to this id
/// (overridable via DRONE_CONVOY_ID), so the dashboard tracks it IMMEDIATELY
/// on load instead of waiting for a convoy-list round trip. `activeConvoys`
/// is still queried right after and overrides this if a different convoy is
/// live.
const DEMO_CONVOY_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

/// Points kept in the rolling telemetry chart (~1 minute at 2s per point).
const TELEMETRY_SERIES_CAP: usize = 30;

/// Stylesheet and favicon bundled by dx via `asset!()`. This is the Dioxus
/// way to ship static files: dx copies them into the build output under a
/// content-hashed path and the macro yields the URL. `main.css` is the SAME
/// byte-for-byte file as the Leptos build.
const MAIN_CSS: Asset = asset!("/style/main.css");
const FAVICON: Asset = asset!("/assets/drone-favicon.svg");

#[component]
pub fn App() -> Element {
    let state = provide_app_state();
    start_live_feed(state);

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        document::Link { rel: "icon", r#type: "image/svg+xml", href: FAVICON }
        div { class: "scanlines" }
        div { class: "hud-container",
            Header {}
            div { class: "hud-left-panel",
                LeaderboardPanel {}
                DroneListPanel {}
            }
            div { class: "hud-main",
                MapPanel {}
            }
            div { class: "hud-right-panel",
                ConvoyStatsPanel {}
                TelemetryChartPanel {}
                EngagementFeedPanel {}
            }
            Footer {}
        }
        ToastContainer {}
    }
}

#[component]
fn ToastContainer() -> Element {
    let mut state = use_app_state();

    rsx! {
        div { class: "toast-container",
            for alert in state.alerts.read().iter().cloned() {
                div { key: "{alert.id}", class: "toast",
                    div { class: "flex justify-between items-center gap-md",
                        div { class: "flex items-center gap-sm",
                            span { class: "status-dot {alert.severity.class()}" }
                            span { "{alert.message}" }
                        }
                        button {
                            class: "btn btn-sm",
                            onclick: move |_| {
                                let id = alert.id;
                                state.alerts.write().retain(|a| a.id != id);
                            },
                            "×"
                        }
                    }
                }
            }
        }
    }
}

/// Start the live feed for every panel.
///
/// The demo convoy id is selected synchronously so the first poll fires on
/// the first tick — ONLINE within one interval of page load. The convoy list
/// is then fetched in the background purely to override the selection if a
/// different convoy is actually live.
fn start_live_feed(mut state: AppState) {
    // Dioxus rule: never write signals during render (it re-triggers the
    // render that owns these hooks and can restart the futures below). All
    // initial state lands inside the once-only future instead. The demo
    // convoy is selected on the first tick of that future — still before the
    // first poll, so ONLINE within one interval of page load holds.
    use_future(move || async move {
        state.mission_start.set(Some(Utc::now()));
        if let Ok(id) = Uuid::parse_str(DEMO_CONVOY_ID) {
            state.selected_convoy.set(Some(id));
        }
        match services::fetch_active_convoys().await {
            Ok(convoys) => {
                if let Some(first) = convoys.first() {
                    if let Ok(id) = Uuid::parse_str(&first.convoy_id) {
                        state.selected_convoy.set(Some(id));
                        log::info!("tracking convoy {} ({})", first.callsign, id);
                    }
                } else {
                    log::info!("no active convoys yet; tracking demo convoy until the service bootstraps");
                }
            }
            Err(err) => log::warn!("could not list convoys: {err}"),
        }
    });

    // The 2-second tick behind every panel. Three queries per tick —
    // leaderboard, drones, engagements. `ws_connected` reflects whether the
    // leaderboard poll (the cheapest, always-valid one) succeeded, so the
    // ONLINE pill is an honest link indicator.
    use_future(move || async move {
        loop {
            if let Some(convoy_id) = state.selected_convoy.peek().clone() {
                match services::fetch_leaderboard(convoy_id, 10).await {
                    Ok(entries) => {
                        state.ws_connected.set(true);
                        state.leaderboard.set(entries);
                    }
                    Err(err) => {
                        state.ws_connected.set(false);
                        log::warn!("leaderboard poll failed: {err}");
                    }
                }

                match services::fetch_drones(convoy_id).await {
                    Ok(drones) => {
                        push_telemetry_point(state.telemetry_series, &drones);
                        state.drones.set(drones.into_iter().map(|d| (d.drone_id, d)).collect());
                    }
                    Err(err) => log::warn!("drones poll failed: {err}"),
                }

                match services::fetch_engagements(convoy_id, 20).await {
                    Ok(mut engagements) => {
                        // The engagement record carries no post-shot accuracy;
                        // stamp it from the freshest leaderboard so the feed
                        // reads like the subscription event would.
                        let accuracy: HashMap<Uuid, f32> = state
                            .leaderboard
                            .peek()
                            .iter()
                            .map(|e| (e.drone_id, e.accuracy_pct))
                            .collect();
                        for e in &mut engagements {
                            if let Some(acc) = accuracy.get(&e.drone_id) {
                                e.new_accuracy_pct = *acc;
                            }
                        }
                        state.engagements.set(engagements);
                    }
                    Err(err) => log::warn!("engagements poll failed: {err}"),
                }
            }
            gloo_timers::future::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
        }
    });
}

/// Append one convoy-average sample to the rolling telemetry series.
///
/// Rounded to one decimal at the source: raw f64 averages leak float dust
/// into the chart tooltip. Averages across airborne assets keep the chart
/// meaningful as drones join and leave; the cap keeps it a sliding window.
fn push_telemetry_point(mut series_signal: Signal<Vec<TelemetryPoint>>, drones: &[DroneState]) {
    if drones.is_empty() {
        return;
    }
    let n = drones.len() as f64;
    let avg_altitude_m = ((drones.iter().map(|d| d.position.altitude_m).sum::<f64>() / n) * 10.0).round() / 10.0;
    let avg_fuel_pct = ((drones.iter().map(|d| f64::from(d.fuel_pct)).sum::<f64>() / n) * 10.0).round() / 10.0;
    let label = Utc::now().format("%H:%M:%S").to_string();

    let mut series = series_signal.write();
    series.push(TelemetryPoint { label, avg_altitude_m, avg_fuel_pct });
    if series.len() > TELEMETRY_SERIES_CAP {
        let excess = series.len() - TELEMETRY_SERIES_CAP;
        series.drain(0..excess);
    }
}

pub fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
    log::info!("Drone Convoy Tracker v{} (Dioxus)", env!("CARGO_PKG_VERSION"));
    dioxus::launch(App);
}
