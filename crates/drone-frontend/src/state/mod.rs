//! # Application State
//!
//! Reactive state management for the drone convoy HUD.

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Global application state
#[derive(Clone, Copy, Debug)]
pub struct AppState {
    pub selected_convoy: Signal<Option<Uuid>>,
    pub selected_drone: Signal<Option<Uuid>>,
    pub leaderboard: Signal<Vec<LeaderboardEntry>>,
    pub drones: Signal<HashMap<Uuid, DroneState>>,
    pub engagements: Signal<Vec<EngagementEvent>>,
    /// Rolling convoy-average telemetry, one point per poll tick. The chart
    /// reads this reactively; `lib.rs` appends and caps it.
    pub telemetry_series: Signal<Vec<TelemetryPoint>>,
    /// Which tactical theater the map shows. Written by the header's mission
    /// selector, read by `map.rs`. Default Afghanistan (Kandahar).
    pub selected_theater: Signal<crate::components::regions::TheaterId>,
    /// Smoothed live position per drone for the GPS readout: the map's flight
    /// loop interpolates between the last two SERVER fixes (from the 2 s
    /// poll) at animation rate and writes here. Truth is re-anchored on every
    /// poll; the display glides between fixes — exactly what a real GPS/INS
    /// display does. Never invented: with a single fix it holds that fix.
    pub live_positions: Signal<HashMap<Uuid, LivePosition>>,
    /// A tasking order is in flight: the selector chose this theater and the
    /// convoy record has been (or is being) updated, but the airframes have
    /// not yet reported from there. Cleared by the map when the server's
    /// positions arrive in the new theater. Drives the RETASKING indicator.
    pub retasking: Signal<Option<crate::components::regions::TheaterId>>,
    /// Last tasking order the API rejected (message), shown on the HUD card
    /// so a failure is never silent. Cleared on the next order.
    pub retask_error: Signal<Option<String>>,
    pub ws_connected: Signal<bool>,
    pub mission_start: Signal<Option<DateTime<Utc>>>,
    pub alerts: Signal<Vec<Alert>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            selected_convoy: Signal::new(None),
            selected_drone: Signal::new(None),
            leaderboard: Signal::new(Vec::new()),
            drones: Signal::new(HashMap::new()),
            engagements: Signal::new(Vec::new()),
            telemetry_series: Signal::new(Vec::new()),
            selected_theater: Signal::new(crate::components::regions::TheaterId::default()),
            live_positions: Signal::new(HashMap::new()),
            retasking: Signal::new(None),
            retask_error: Signal::new(None),
            ws_connected: Signal::new(false),
            mission_start: Signal::new(None),
            alerts: Signal::new(Vec::new()),
        }
    }
}

/// A drone's smoothed live fix for the GPS readout (see `AppState::live_positions`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LivePosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f64,
    pub heading_deg: f32,
}

/// One convoy-average telemetry sample for the flight telemetry chart.
#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryPoint {
    /// X-axis label (HH:MM:SS of the poll).
    pub label: String,
    pub avg_altitude_m: f64,
    pub avg_fuel_pct: f64,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LeaderboardEntry {
    pub drone_id: Uuid,
    pub callsign: String,
    pub platform_type: String,
    pub rank: u32,
    pub accuracy_pct: f32,
    pub total_engagements: u32,
    pub successful_hits: u32,
    pub current_streak: i32,
    pub best_streak: i32,
    pub rank_change: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DroneState {
    pub drone_id: Uuid,
    pub convoy_id: Uuid,
    pub callsign: String,
    pub tail_number: String,
    pub platform_type: String,
    pub status: DroneStatus,
    pub position: Coordinates,
    pub fuel_pct: f32,
    pub accuracy_pct: f32,
    pub current_waypoint: u32,
    pub total_waypoints: u32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DroneStatus {
    Preflight,
    Airborne,
    Loiter,
    Ingress,
    Egress,
    Rtb,
    Landed,
    Maintenance,
}

impl DroneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Preflight => "PREFLIGHT",
            Self::Airborne => "AIRBORNE",
            Self::Loiter => "LOITER",
            Self::Ingress => "INGRESS",
            Self::Egress => "EGRESS",
            Self::Rtb => "RTB",
            Self::Landed => "LANDED",
            Self::Maintenance => "MAINT",
        }
    }

    pub fn status_class(&self) -> &'static str {
        match self {
            Self::Airborne | Self::Loiter | Self::Ingress | Self::Egress => "nominal",
            Self::Rtb | Self::Preflight => "warning",
            Self::Landed | Self::Maintenance => "offline",
        }
    }

    /// Whether the airframe is in the air. Distinct from `status_class`:
    /// RTB is warning-TIER for display, but a drone flying home is still
    /// airborne — counting only "nominal" statuses made the stats panel
    /// read "0/4 airborne" during the RTB phase while four drones flew.
    pub fn is_airborne(&self) -> bool {
        matches!(
            self,
            Self::Airborne | Self::Loiter | Self::Ingress | Self::Egress | Self::Rtb
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f64,
    pub heading_deg: f32,
    pub speed_mps: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EngagementEvent {
    pub id: Uuid,
    pub drone_id: Uuid,
    pub callsign: String,
    pub hit: bool,
    pub weapon_type: String,
    pub new_accuracy_pct: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: Uuid,
    pub sequence: u32,
    pub name: String,
    pub coordinates: Coordinates,
    pub status: WaypointStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WaypointStatus {
    Pending,
    Active,
    Complete,
    Skipped,
}

pub fn provide_app_state() -> AppState {
    // Dioxus: signals are created inside a component scope; the App component
    // calls this once via use_context_provider.
    use_context_provider(AppState::new)
}

pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}
