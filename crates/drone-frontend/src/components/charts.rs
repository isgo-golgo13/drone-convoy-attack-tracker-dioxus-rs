//! # Telemetry Chart Component
//!
//! Real-time charts using Charming (ECharts wrapper).

use charming::{
    component::{Axis, Grid, Legend},
    element::{AreaStyle, AxisType, LineStyle, Tooltip, Trigger},
    series::Line,
    Chart, WasmRenderer,
};
use dioxus::prelude::*;

use crate::state::use_app_state;

/// Telemetry chart panel
///
/// Renders the rolling convoy-average series from `state.telemetry_series`.
/// Reading the signal inside the effect makes the render reactive: every
/// poll tick that appends a point re-renders the chart. Charming's WASM
/// renderer redraws in place on the same element id.
#[component]
pub fn TelemetryChartPanel() -> Element {
    let state = use_app_state();
    let chart_id = "telemetry-chart";

    // The Echarts handle from the first render. Subsequent series changes go
    // through WasmRenderer::update — calling render() again would echarts-init
    // the same DOM node repeatedly and stack instances on every poll tick.
    // Held in a hook-scoped Rc so it survives re-renders.
    let echarts = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(None::<charming::Echarts>)));

    // Rebuild chart whenever the series changes. use_effect auto-tracks the
    // read of telemetry_series. The map div is in the DOM after first paint,
    // and effects run after render, so render() finds its element.
    {
        let echarts = echarts.clone();
        use_effect(move || {
            let series = state.telemetry_series.read().clone();
            if series.len() < 2 {
                return;
            }
            let labels: Vec<String> = series.iter().map(|p| p.label.clone()).collect();
            let altitude_data: Vec<f64> = series.iter().map(|p| p.avg_altitude_m).collect();
            let fuel_data: Vec<f64> = series.iter().map(|p| p.avg_fuel_pct).collect();

        let chart = Chart::new()
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .legend(
                Legend::new()
                    .data(vec!["Altitude (m)", "Fuel (%)"])
                    .text_style(charming::element::TextStyle::new().color("#99cc99"))
                    .bottom(0),
            )
            .grid(
                Grid::new()
                    .left("10%")
                    .right("10%")
                    .top("8%")       // title strip gone -> reclaim it for the plot
                    .bottom("20%"),
            )
            .x_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(labels)
                    .axis_line(charming::element::AxisLine::new().line_style((1.0, "#557755")))
                    .axis_label(charming::element::AxisLabel::new().color("#557755")),
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .name("Altitude (m)")
                    .axis_line(charming::element::AxisLine::new().line_style((1.0, "#557755")))
                    .axis_label(charming::element::AxisLabel::new().color("#557755"))
                    .split_line(charming::element::SplitLine::new().line_style(LineStyle::new().color("#1a2a1a"))),
            )
            .series(
                Line::new()
                    .name("Altitude (m)")
                    .data(altitude_data)
                    .smooth(true)
                    .line_style(LineStyle::new().color("#00ff41").width(2))
                    .area_style(AreaStyle::new().color("rgba(0, 255, 65, 0.1)")),
            )
            .series(
                Line::new()
                    .name("Fuel (%)")
                    .data(fuel_data)
                    .smooth(true)
                    .line_style(LineStyle::new().color("#ffaa00").width(2))
                    .area_style(AreaStyle::new().color("rgba(255, 170, 0, 0.1)")),
            );


            let mut handle = echarts.borrow_mut();
            if let Some(instance) = handle.as_ref() {
                WasmRenderer::update(instance, &chart);
            } else {
                match WasmRenderer::new(400, 200).render(chart_id, &chart) {
                    Ok(instance) => *handle = Some(instance),
                    Err(e) => log::error!("Chart render error: {:?}", e),
                }
            }
        });
    }

    let live = state.telemetry_series.read().len() >= 2;

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                span { class: "panel-title", "FLIGHT TELEMETRY" }
                if live { span { class: "panel-badge", "LIVE" } }
            }
            div { class: "panel-body no-padding",
                div { id: "{chart_id}", class: "chart-container" }
            }
        }
    }
}

/// Stats summary panel
#[component]
pub fn ConvoyStatsPanel() -> Element {
    let state = use_app_state();
    let drones = state.drones.read();
    let leaderboard = state.leaderboard.read();

    let total = drones.len();
    let airborne = drones.values().filter(|d| d.status.is_airborne()).count();
    let avg_fuel: f32 = if total > 0 { drones.values().map(|d| d.fuel_pct).sum::<f32>() / total as f32 } else { 0.0 };
    let avg_accuracy: f32 = if !leaderboard.is_empty() {
        leaderboard.iter().map(|e| e.accuracy_pct).sum::<f32>() / leaderboard.len() as f32
    } else { 0.0 };
    let total_engagements: u32 = leaderboard.iter().map(|e| e.total_engagements).sum();
    let total_hits: u32 = leaderboard.iter().map(|e| e.successful_hits).sum();
    let fuel_class = if avg_fuel < 40.0 { "text-xl font-bold text-warning" } else { "text-xl font-bold" };
    let fuel_text = format!("{:.0}%", avg_fuel);
    let acc_text = format!("{:.1}%", avg_accuracy);

    rsx! {
        div { class: "panel",
            div { class: "panel-header", span { class: "panel-title", "CONVOY STATUS" } }
            div { class: "panel-body",
                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                    div {
                        div { class: "text-xs text-muted uppercase tracking-wide", "ASSETS" }
                        div { class: "text-xl font-bold text-accent", "{airborne}/{total}" }
                        div { class: "text-xs text-muted", "airborne" }
                    }
                    div {
                        div { class: "text-xs text-muted uppercase tracking-wide", "AVG FUEL" }
                        div { class: "{fuel_class}", "{fuel_text}" }
                        div { class: "text-xs text-muted", "remaining" }
                    }
                    div {
                        div { class: "text-xs text-muted uppercase tracking-wide", "ACCURACY" }
                        div { class: "text-xl font-bold text-accent", "{acc_text}" }
                        div { class: "text-xs text-muted", "convoy avg" }
                    }
                    div {
                        div { class: "text-xs text-muted uppercase tracking-wide", "ENGAGEMENTS" }
                        div { class: "text-xl font-bold", "{total_hits}/{total_engagements}" }
                        div { class: "text-xs text-muted", "hits/total" }
                    }
                }
            }
        }
    }
}
