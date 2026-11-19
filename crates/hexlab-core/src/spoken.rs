//! Spoken bridge — a 1:1 Rust port of `src/spoken/spokenBridge.ts`.
//!
//! Spoken-world style, decoupled from the estate. Tier-0 local grammar:
//! instant ops for install + facility BOM + stock. The same grammar is
//! what a voice front-end (OS speech APIs locally) feeds into.
//!
//! Examples:
//!   "install a band saw"
//!   "install drill press on selected"
//!   "create a bom for a woodshop and stock it"
//!   "stock the machine shop"
//!   "show bom for woodshop"

use crate::catalog::{get_facility_bom, resolve_asset_by_noun, FACILITY_BOMS};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SpokenOp {
    Install { asset_id: String, target: InstallTarget },
    Bom { facility_id: String, stock: bool },
    ListFacilities,
    Help,
    Unknown { raw: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallTarget {
    Selected,
    Next,
}

fn normalize(input: &str) -> String {
    let lowered = input.to_lowercase();
    let cleaned: String = lowered
        .chars()
        .map(|c| if matches!(c, '.' | ',' | '!' | '?') { ' ' } else { c })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_article(s: &str) -> &str {
    s.strip_prefix("a ").or_else(|| s.strip_prefix("the ")).unwrap_or(s)
}

fn fuzzy_facility(name: &str) -> Option<&'static crate::catalog::FacilityBom> {
    let n = name.to_lowercase();
    let has = |pats: &[&str]| pats.iter().any(|p| n.contains(p));
    if has(&["wood", "maker", "carpentry", "joinery"]) {
        return get_facility_bom("woodshop");
    }
    if has(&["machine shop", "machining", "cnc"]) {
        return get_facility_bom("machineshop");
    }
    if has(&["fab", "weld", "metal"]) {
        return get_facility_bom("fabrication");
    }
    if has(&["ware", "logistics", "storage"]) {
        return get_facility_bom("warehouse");
    }
    if has(&["process", "chemical", "desal", "refin"]) {
        return get_facility_bom("process");
    }
    if has(&["power", "energy", "solar", "gen"]) {
        return get_facility_bom("power");
    }
    if has(&["robot", "automation", "cell"]) {
        return get_facility_bom("robotics");
    }
    None
}

fn facility_from(name: &str) -> Option<&'static crate::catalog::FacilityBom> {
    let name = strip_article(name.trim());
    get_facility_bom(name).or_else(|| fuzzy_facility(name))
}

pub fn parse_spoken(input: &str) -> SpokenOp {
    let raw = input.trim().to_string();
    let t = normalize(input);

    if t.is_empty() || matches!(t.as_str(), "help" | "what can i say" | "?") {
        return SpokenOp::Help;
    }
    if t.contains("list facilities") || t.contains("list boms") || t.contains("list shops") || t == "facilities" {
        return SpokenOp::ListFacilities;
    }

    // BOM forms: "create a bom for a woodshop and stock it",
    // "stock the machine shop", "show bom for fabrication".
    let stock_hint = t.contains("stock");
    let bom_candidate: Option<String> = if let Some(rest) = strip_bom_prefix(&t) {
        Some(rest)
    } else if let Some(rest) = t.strip_prefix("stock ") {
        Some(rest.trim_start_matches("the ").to_string())
    } else {
        None
    };
    if let Some(rest) = bom_candidate {
        let name = rest
            .replace(" and stock it", "")
            .replace(" and stock", "")
            .replace(" stock", "");
        let name = name
            .trim_end_matches(" bay")
            .trim_end_matches(" shop")
            .trim_end_matches(" facility");
        // "stock the machine shop" trims the word shop — restore facility check on full rest first.
        if let Some(f) = facility_from(&rest.replace(" and stock it", "").replace(" and stock", ""))
            .or_else(|| facility_from(name)) {
            return SpokenOp::Bom { facility_id: f.id.to_string(), stock: stock_hint || t.starts_with("stock") };
        }
    }

    // Install forms: "install a band saw", "place drill press on selected".
    for verb in ["install", "place", "add", "put", "drop", "mount"] {
        if let Some(rest) = t.strip_prefix(&format!("{verb} ")) {
            let (noun, target) = split_target(rest);
            if let Some(asset) = resolve_asset_by_noun(noun) {
                return SpokenOp::Install { asset_id: asset.id.to_string(), target };
            }
        }
    }

    // Bare asset name as install (≤ 4 words, as in the TS bridge).
    if t.split(' ').count() <= 4 {
        if let Some(asset) = resolve_asset_by_noun(&t) {
            return SpokenOp::Install { asset_id: asset.id.to_string(), target: InstallTarget::Next };
        }
    }

    SpokenOp::Unknown { raw }
}

/// "create a bom for a woodshop ..." / "show bom for fabrication" /
/// "bom woodshop stock" → the facility-name remainder.
fn strip_bom_prefix(t: &str) -> Option<String> {
    let mut s = t;
    for verb in ["create ", "make ", "generate ", "build "] {
        if let Some(rest) = s.strip_prefix(verb) {
            s = rest;
            break;
        }
    }
    for verb in ["show ", "open "] {
        if let Some(rest) = s.strip_prefix(verb) {
            s = rest;
            break;
        }
    }
    let s = s.strip_prefix("a ").unwrap_or(s);
    let s = s.strip_prefix("the ").unwrap_or(s);
    if let Some(rest) = s.strip_prefix("bom for ") {
        return Some(strip_article(rest.trim()).to_string());
    }
    if let Some(rest) = s.strip_prefix("bill of materials for ") {
        return Some(strip_article(rest.trim()).to_string());
    }
    if let Some(rest) = s.strip_prefix("bom ") {
        return Some(strip_article(rest.trim()).to_string());
    }
    None
}

fn split_target(rest: &str) -> (&str, InstallTarget) {
    for (suffix, target) in [
        (" on selected", InstallTarget::Selected),
        (" on this", InstallTarget::Selected),
        (" on here", InstallTarget::Selected),
        (" on next", InstallTarget::Next),
    ] {
        if let Some(noun) = rest.strip_suffix(suffix) {
            return (noun.trim(), target);
        }
    }
    (rest.trim(), InstallTarget::Next)
}

pub fn help_text() -> String {
    let facilities = FACILITY_BOMS
        .iter()
        .map(|f| f.id)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "SPOKEN // HEXLAB (decoupled from estate)\n\
         \n\
         Install a piece:\n\
         \x20 \"install a band saw\"\n\
         \x20 \"place drill press on selected\"\n\
         \x20 \"add a workbench\"\n\
         \n\
         Facility BOM + stock:\n\
         \x20 \"create a bom for a woodshop and stock it\"\n\
         \x20 \"stock the machine shop\"\n\
         \x20 \"show bom for fabrication\"\n\
         \n\
         Facilities: {facilities}\n\
         \n\
         Assets map to NVIDIA USDCraft-10K industrial (Apache-2.0)."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bom_and_stock() {
        assert_eq!(
            parse_spoken("create a bom for a woodshop and stock it"),
            SpokenOp::Bom { facility_id: "woodshop".into(), stock: true }
        );
        assert_eq!(
            parse_spoken("stock the machine shop"),
            SpokenOp::Bom { facility_id: "machineshop".into(), stock: true }
        );
        assert_eq!(
            parse_spoken("show bom for fabrication"),
            SpokenOp::Bom { facility_id: "fabrication".into(), stock: false }
        );
    }

    #[test]
    fn install_forms() {
        assert_eq!(
            parse_spoken("install a band saw"),
            SpokenOp::Install { asset_id: "band_saw".into(), target: InstallTarget::Next }
        );
        assert_eq!(
            parse_spoken("place drill press on selected"),
            SpokenOp::Install { asset_id: "drill_press".into(), target: InstallTarget::Selected }
        );
        assert_eq!(
            parse_spoken("band saw"),
            SpokenOp::Install { asset_id: "band_saw".into(), target: InstallTarget::Next }
        );
    }

    #[test]
    fn help_and_unknown() {
        assert_eq!(parse_spoken("help"), SpokenOp::Help);
        assert_eq!(parse_spoken("facilities"), SpokenOp::ListFacilities);
        assert!(matches!(parse_spoken("dance a little jig"), SpokenOp::Unknown { .. }));
    }
}
