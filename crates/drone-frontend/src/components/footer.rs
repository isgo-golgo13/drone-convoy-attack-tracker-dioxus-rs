//! # Footer Component
//!
//! Status bar with system information.

use dioxus::prelude::*;

use crate::state::use_app_state;

/// Dissemination marking shown in the footer. FOUO ("For Official Use Only")
/// is the legacy pre-2020 DoD marking; the current equivalent is CUI
/// ("Controlled Unclassified Information"). Kept as one constant so the
/// demo can wear whichever reads right for the room.
pub const CLASSIFICATION_MARKING: &str = "UNCLASSIFIED // CUI";

/// Footer status bar
#[component]
pub fn Footer() -> Element {
    let state = use_app_state();

    let (dot_class, conn_label) = if (state.ws_connected)() {
        ("nominal", "CONNECTED")
    } else {
        ("critical", "DISCONNECTED")
    };
    let drone_count = state.drones.read().len();
    let alert_count = state.alerts.read().len();

    rsx! {
        footer { class: "hud-footer",
            div { class: "flex items-center gap-lg",
                span { class: "text-muted", "DRONE OPS v0.1.0" }
                span { class: "text-muted", "|" }
                span {
                    span { class: "text-muted", "ASSETS: " }
                    span { class: "text-accent", "{drone_count}" }
                }
            }

            div { class: "flex items-center gap-lg",
                if alert_count > 0 {
                    span { class: "status-badge warning", "{alert_count} ALERTS" }
                }

                span { class: "flex items-center gap-xs",
                    span { class: "status-dot {dot_class}" }
                    span { class: "text-sm", "{conn_label}" }
                }

                span { class: "text-muted", "CLASSIFICATION: {CLASSIFICATION_MARKING}" }
            }
        }
    }
}
