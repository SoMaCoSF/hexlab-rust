//! HexLab Tauri shell — Rust backend for the desktop app.
//!
//! Tauri ≠ Electron: Rust process + OS webview (WebKitGTK on
//! Omarchy/Linux, WebView2 on Windows, WKWebView on macOS), no bundled
//! Chromium, no Node. The UI in `../ui` is the first frontend — the
//! existing React Three Fiber Build Bay from SoMaCoSF/hexlab-platform
//! can replace it without touching these commands, because every
//! command is a thin invoke over `hexlab-core`, the same crate the
//! CLI and TUI use.
//!
//! Build (on the target machine — needs the OS webview dev packages):
//!   cargo install tauri-cli --version "^2"
//!   cd apps/hexlab-tauri && cargo tauri build
//! On this build VM the webview system libraries are absent, so this
//! crate is scaffold-verified only; core/CLI/TUI carry the test proof.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use hexlab_core::catalog::{asset_by_id, get_facility_bom};
use hexlab_core::city::City;
use hexlab_core::kit::facility_bom_to_anyscene_plan;
use hexlab_core::spoken::{help_text, parse_spoken, SpokenOp};
use hexlab_core::types::SizeClass;
use std::str::FromStr;
use std::sync::Mutex;

struct State {
    city: Mutex<City>,
}

#[tauri::command]
fn city_preset(state: tauri::State<State>, name: String) -> Result<serde_json::Value, String> {
    let city = City::preset(&name).ok_or_else(|| format!("unknown preset '{name}'"))?;
    *state.city.lock().unwrap() = city.clone();
    Ok(serde_json::json!({
        "telemetry": city.telemetry(),
        "city": city,
    }))
}

#[tauri::command]
fn city_place(
    state: tauri::State<State>,
    q: i32,
    r: i32,
    module: String,
    size: String,
) -> Result<serde_json::Value, String> {
    let module = hexlab_core::types::ModuleType::from_str(&module)?;
    let size = SizeClass::from_str(&size)?;
    let mut city = state.city.lock().unwrap();
    city.place(hexlab_core::Axial::new(q, r), module, size);
    Ok(serde_json::json!({ "telemetry": city.telemetry() }))
}

#[tauri::command]
fn city_towers(state: tauri::State<State>) -> serde_json::Value {
    let city = state.city.lock().unwrap();
    serde_json::json!({ "towers": city.towers(), "edges": city.edges().len() })
}

#[tauri::command]
fn facility_bom(id: String) -> Result<serde_json::Value, String> {
    let bom = get_facility_bom(&id).ok_or_else(|| format!("unknown facility '{id}'"))?;
    let lines: Vec<_> = bom
        .lines
        .iter()
        .map(|l| {
            let a = asset_by_id(l.asset_id);
            serde_json::json!({
                "asset_id": l.asset_id,
                "name": a.map(|a| a.name),
                "qty": l.qty,
                "footprint_m": a.map(|a| a.footprint),
            })
        })
        .collect();
    Ok(serde_json::json!({ "facility": bom.id, "label": bom.label, "lines": lines }))
}

#[tauri::command]
fn spoken(text: String) -> serde_json::Value {
    match parse_spoken(&text) {
        SpokenOp::Help => serde_json::json!({ "op": "help", "text": help_text() }),
        other => serde_json::to_value(other).unwrap_or_default(),
    }
}

#[tauri::command]
fn kit_plan(facility: String, size: String) -> Result<serde_json::Value, String> {
    let bom = get_facility_bom(&facility).ok_or_else(|| format!("unknown facility '{facility}'"))?;
    let size = SizeClass::from_str(&size)?;
    Ok(facility_bom_to_anyscene_plan(bom, size))
}

#[tauri::command]
fn export_brief(state: tauri::State<State>, format: String) -> String {
    let city = state.city.lock().unwrap();
    match format.as_str() {
        "xthread" => hexlab_core::export::x_thread(&city),
        _ => hexlab_core::export::spec_sheet(&city),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(State {
            city: Mutex::new(City::preset_carrier()),
        })
        .invoke_handler(tauri::generate_handler![
            city_preset,
            city_place,
            city_towers,
            facility_bom,
            spoken,
            kit_plan,
            export_brief
        ])
        .run(tauri::generate_context!())
        .expect("error while running HexLab");
}
