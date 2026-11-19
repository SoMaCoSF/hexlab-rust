//! AnyKIT catalog — the USDCraft-10K industrial harvest + facility BOMs.
//!
//! In the web repo this file is a stub: Grok's full catalog never landed
//! (MCP payload limits; the local copy is lost). This is the rebuild:
//! assets keyed to NVIDIA USDCraft-10K `rigid/industrial_warehouse_
//! construction_energy` (Apache-2.0), with the spoken nouns the bridge
//! resolves, footprints for the AnyScene plan bridge, and the seven
//! facility BOMs from the thread (woodshop first, per Sam's ask).

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct CraftAsset {
    pub id: &'static str,
    pub name: &'static str,
    /// Spoken / search nouns, longest first.
    pub nouns: &'static [&'static str],
    /// Footprint + height in metres (w, d, h).
    pub footprint: (f64, f64, f64),
    /// Path under the USDCraft industrial category, if harvested there.
    pub usdcraft_path: Option<&'static str>,
    pub category: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct BomLine {
    pub asset_id: &'static str,
    pub qty: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct FacilityBom {
    pub id: &'static str,
    pub label: &'static str,
    pub lines: &'static [BomLine],
}

const fn line(asset_id: &'static str, qty: u32) -> BomLine {
    BomLine { asset_id, qty }
}

pub const INDUSTRIAL_ASSETS: &[CraftAsset] = &[
    // Woodshop
    CraftAsset { id: "band_saw", name: "Band saw", nouns: &["band saw", "bandsaw"], footprint: (0.9, 0.6, 1.8), usdcraft_path: Some("band_saw"), category: "woodshop" },
    CraftAsset { id: "drill_press", name: "Drill press", nouns: &["drill press"], footprint: (0.6, 0.5, 1.7), usdcraft_path: Some("drill_press"), category: "woodshop" },
    CraftAsset { id: "table_saw", name: "Table saw", nouns: &["table saw"], footprint: (1.2, 0.9, 1.1), usdcraft_path: Some("table_saw"), category: "woodshop" },
    CraftAsset { id: "workbench", name: "Workbench", nouns: &["workbench", "bench"], footprint: (1.8, 0.8, 0.95), usdcraft_path: Some("workbench"), category: "woodshop" },
    CraftAsset { id: "laser_cutter", name: "Laser cutter", nouns: &["laser cutter", "laser"], footprint: (1.4, 1.0, 1.2), usdcraft_path: Some("laser_cutter"), category: "woodshop" },
    CraftAsset { id: "tool_cart", name: "Tool cart", nouns: &["tool cart", "tool chest", "cart"], footprint: (0.8, 0.5, 1.0), usdcraft_path: Some("tool_cart"), category: "woodshop" },
    CraftAsset { id: "dust_collector", name: "Dust collector", nouns: &["dust collector"], footprint: (0.8, 0.8, 2.0), usdcraft_path: Some("dust_collector"), category: "woodshop" },
    CraftAsset { id: "lumber_rack", name: "Lumber rack", nouns: &["lumber rack", "timber rack"], footprint: (2.4, 0.6, 2.2), usdcraft_path: Some("lumber_rack"), category: "woodshop" },
    // Machine shop
    CraftAsset { id: "cnc_mill", name: "CNC mill", nouns: &["cnc mill", "cnc", "mill"], footprint: (2.2, 1.8, 2.4), usdcraft_path: Some("cnc_mill"), category: "machineshop" },
    CraftAsset { id: "lathe", name: "Lathe", nouns: &["lathe"], footprint: (2.0, 0.8, 1.4), usdcraft_path: Some("lathe"), category: "machineshop" },
    CraftAsset { id: "surface_grinder", name: "Surface grinder", nouns: &["surface grinder", "grinder"], footprint: (1.2, 0.9, 1.8), usdcraft_path: Some("surface_grinder"), category: "machineshop" },
    CraftAsset { id: "inspection_table", name: "Inspection table", nouns: &["inspection table", "granite table"], footprint: (1.5, 1.0, 0.9), usdcraft_path: Some("inspection_table"), category: "machineshop" },
    // Fabrication
    CraftAsset { id: "hydraulic_press", name: "Hydraulic press", nouns: &["hydraulic press", "press"], footprint: (1.4, 1.0, 2.6), usdcraft_path: Some("hydraulic_press"), category: "fabrication" },
    CraftAsset { id: "welding_station", name: "Welding station", nouns: &["welding station", "welder"], footprint: (1.6, 1.0, 1.6), usdcraft_path: Some("welding_station"), category: "fabrication" },
    CraftAsset { id: "plasma_table", name: "Plasma cutting table", nouns: &["plasma table", "plasma cutter"], footprint: (2.6, 1.6, 1.2), usdcraft_path: Some("plasma_table"), category: "fabrication" },
    CraftAsset { id: "gantry_crane", name: "Gantry crane", nouns: &["gantry crane", "crane"], footprint: (4.0, 1.2, 3.5), usdcraft_path: Some("gantry_crane"), category: "fabrication" },
    // Warehouse
    CraftAsset { id: "pallet_rack", name: "Pallet rack", nouns: &["pallet rack", "racking"], footprint: (2.7, 1.1, 3.0), usdcraft_path: Some("pallet_rack"), category: "warehouse" },
    CraftAsset { id: "forklift", name: "Forklift", nouns: &["forklift"], footprint: (2.3, 1.1, 2.1), usdcraft_path: Some("forklift"), category: "warehouse" },
    CraftAsset { id: "conveyor", name: "Conveyor section", nouns: &["conveyor"], footprint: (3.0, 0.8, 0.9), usdcraft_path: Some("conveyor"), category: "warehouse" },
    CraftAsset { id: "packing_station", name: "Packing station", nouns: &["packing station", "pack bench"], footprint: (1.8, 0.9, 1.0), usdcraft_path: Some("packing_station"), category: "warehouse" },
    // Process
    CraftAsset { id: "pump_skid", name: "Pump skid", nouns: &["pump skid", "pump"], footprint: (1.8, 0.9, 1.2), usdcraft_path: Some("pump_skid"), category: "process" },
    CraftAsset { id: "pressure_vessel", name: "Pressure vessel", nouns: &["pressure vessel", "vessel", "tank"], footprint: (1.6, 1.6, 3.2), usdcraft_path: Some("pressure_vessel"), category: "process" },
    CraftAsset { id: "heat_exchanger", name: "Heat exchanger", nouns: &["heat exchanger", "exchanger"], footprint: (2.4, 0.8, 1.4), usdcraft_path: Some("heat_exchanger"), category: "process" },
    CraftAsset { id: "valve_manifold", name: "Valve manifold", nouns: &["valve manifold", "manifold"], footprint: (1.2, 0.6, 1.4), usdcraft_path: Some("valve_manifold"), category: "process" },
    // Power
    CraftAsset { id: "generator", name: "Generator set", nouns: &["generator", "genset"], footprint: (3.0, 1.2, 1.8), usdcraft_path: Some("generator"), category: "power" },
    CraftAsset { id: "battery_rack", name: "Battery rack", nouns: &["battery rack", "batteries"], footprint: (2.0, 0.8, 2.0), usdcraft_path: Some("battery_rack"), category: "power" },
    CraftAsset { id: "switchgear", name: "Switchgear cabinet", nouns: &["switchgear", "switchboard"], footprint: (1.6, 0.6, 2.0), usdcraft_path: Some("switchgear"), category: "power" },
    CraftAsset { id: "transformer", name: "Transformer", nouns: &["transformer"], footprint: (1.8, 1.4, 2.0), usdcraft_path: Some("transformer"), category: "power" },
    // Robotics
    CraftAsset { id: "robot_arm", name: "Robot arm cell", nouns: &["robot arm", "robot"], footprint: (1.6, 1.6, 2.2), usdcraft_path: Some("robot_arm"), category: "robotics" },
    CraftAsset { id: "agv", name: "AGV cart", nouns: &["agv", "automated cart"], footprint: (1.4, 0.8, 0.6), usdcraft_path: Some("agv"), category: "robotics" },
    CraftAsset { id: "vision_station", name: "Vision inspection station", nouns: &["vision station", "vision inspection"], footprint: (1.2, 0.8, 2.0), usdcraft_path: Some("vision_station"), category: "robotics" },
];

pub const WOODSHOP_LINES: &[BomLine] = &[
    line("band_saw", 1), line("drill_press", 1), line("table_saw", 1),
    line("workbench", 4), line("laser_cutter", 1), line("tool_cart", 2),
    line("dust_collector", 1), line("lumber_rack", 2),
];
pub const MACHINESHOP_LINES: &[BomLine] = &[
    line("cnc_mill", 2), line("lathe", 2), line("surface_grinder", 1),
    line("drill_press", 1), line("inspection_table", 1), line("tool_cart", 3),
];
pub const FABRICATION_LINES: &[BomLine] = &[
    line("hydraulic_press", 1), line("welding_station", 3), line("plasma_table", 1),
    line("gantry_crane", 1), line("workbench", 2),
];
pub const WAREHOUSE_LINES: &[BomLine] = &[
    line("pallet_rack", 6), line("forklift", 1), line("conveyor", 3),
    line("packing_station", 2),
];
pub const PROCESS_LINES: &[BomLine] = &[
    line("pump_skid", 3), line("pressure_vessel", 2), line("heat_exchanger", 2),
    line("valve_manifold", 4),
];
pub const POWER_LINES: &[BomLine] = &[
    line("generator", 2), line("battery_rack", 4), line("switchgear", 2),
    line("transformer", 1),
];
pub const ROBOTICS_LINES: &[BomLine] = &[
    line("robot_arm", 3), line("agv", 4), line("vision_station", 2),
];

pub const FACILITY_BOMS: &[FacilityBom] = &[
    FacilityBom { id: "woodshop", label: "Woodshop", lines: WOODSHOP_LINES },
    FacilityBom { id: "machineshop", label: "Machine shop", lines: MACHINESHOP_LINES },
    FacilityBom { id: "fabrication", label: "Fabrication", lines: FABRICATION_LINES },
    FacilityBom { id: "warehouse", label: "Warehouse", lines: WAREHOUSE_LINES },
    FacilityBom { id: "process", label: "Process", lines: PROCESS_LINES },
    FacilityBom { id: "power", label: "Power", lines: POWER_LINES },
    FacilityBom { id: "robotics", label: "Robotics", lines: ROBOTICS_LINES },
];

pub const HF_INDUSTRIAL_BASE: &str =
    "https://huggingface.co/datasets/nvidia/USDCraft-10K/tree/main/rigid/industrial_warehouse_construction_energy";

pub fn asset_hf_url(a: &CraftAsset) -> Option<String> {
    a.usdcraft_path.map(|p| format!("{HF_INDUSTRIAL_BASE}/{p}"))
}

pub fn asset_by_id(id: &str) -> Option<&'static CraftAsset> {
    INDUSTRIAL_ASSETS.iter().find(|a| a.id == id)
}

pub fn get_facility_bom(id: &str) -> Option<&'static FacilityBom> {
    let norm = id.to_ascii_lowercase().replace([' ', '-'], "");
    FACILITY_BOMS.iter().find(|f| f.id.replace([' ', '-'], "") == norm || f.id == norm)
}

/// Resolve a spoken noun to an asset: exact noun, then noun-contained,
/// then asset-name match — the bridge's `resolveAssetByNoun`.
pub fn resolve_asset_by_noun(text: &str) -> Option<&'static CraftAsset> {
    let t = text.to_ascii_lowercase();
    let t = t.trim().trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ");
    for a in INDUSTRIAL_ASSETS {
        if a.nouns.iter().any(|n| *n == t) || a.name.to_ascii_lowercase() == t {
            return Some(a);
        }
    }
    // Longest contained noun wins, so "band saw" beats "saw"-ish partials.
    let mut best: Option<(&CraftAsset, usize)> = None;
    for a in INDUSTRIAL_ASSETS {
        for n in a.nouns {
            if t.contains(n) || n.contains(t) {
                if best.map(|(_, l)| n.len() > l).unwrap_or(true) {
                    best = Some((a, n.len()));
                }
            }
        }
    }
    best.map(|(a, _)| a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_facilities_and_woodshop_first() {
        assert_eq!(FACILITY_BOMS.len(), 7);
        let w = get_facility_bom("woodshop").unwrap();
        assert_eq!(w.lines.len(), 8);
        assert!(get_facility_bom("machine shop").is_some());
    }

    #[test]
    fn noun_resolution() {
        assert_eq!(resolve_asset_by_noun("band saw").unwrap().id, "band_saw");
        assert_eq!(resolve_asset_by_noun("a drill press").unwrap().id, "drill_press");
        assert_eq!(resolve_asset_by_noun("hydraulic press").unwrap().id, "hydraulic_press");
        assert!(resolve_asset_by_noun("flux capacitor").is_none());
    }

    #[test]
    fn all_bom_lines_resolve() {
        for f in FACILITY_BOMS {
            for l in f.lines {
                assert!(asset_by_id(l.asset_id).is_some(), "{} -> {}", f.id, l.asset_id);
            }
        }
    }
}
