//! Export briefs — the configurator's X-thread reply + spec sheet,
//! generated from live telemetry (same text shapes as the web export).

use crate::city::City;

fn fmt_int(n: f64) -> String {
    let v = n.round() as i64;
    let s = v.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if v < 0 {
        format!("-{out}")
    } else {
        out
    }
}

pub fn x_thread(city: &City) -> String {
    let t = city.telemetry();
    format!(
        "@PalmerLuckey Why deselect Hexagons? Rectangular hulls concentrate 4-corner shear in cross-seas, whereas 120° hexagonal tessellation distributes wave loads tri-axially and unlocks discrete inframodules:\n\
         \n\
         1/ Each hex unit is an independent functional bay:\n\
         • {}x Flight-Harbor bays with articulating swing/raise/lower face gates for UAV/VTOL ops\n\
         • {}x Ballast-controlled drainable dry-docks for USV maintenance\n\
         • {}x Protected central moonpool sub-launching harbor for UUVs\n\
         \n\
         2/ Structural & Station-Keeping Advantage:\n\
         • {} solid floating corner interlocks + vertex mooring ties\n\
         • {} retractable tuck swivel propellers at the 120° 3-way intersections for omni-directional dynamic positioning\n\
         • Stackable Tier-2 upper decks ({} active hexes / {} m² total deck footprint)\n\
         \n\
         Interactive CAD configurator & proposal model live at hexlab.somacosf.com (powered by SoMaCoSF anymachine).",
        t.flight,
        t.drydock,
        t.subhub,
        t.interlock_nodes,
        t.thrusters,
        t.total,
        fmt_int(t.deck_area_m2),
    )
}

pub fn spec_sheet(city: &City) -> String {
    let t = city.telemetry();
    format!(
        "====================================================================\n\
         HEXLAB.SOMACOSF.COM // MODULAR HEXAGONAL MARINE ARCHITECTURE BRIEF\n\
         ENGINE: SoMaCoSF Anymachine Tri-Axial Configurator\n\
         ====================================================================\n\
         \n\
         1. ARRAY CONFIGURATION TELEMETRY\n\
         \x20  - Total Modular Hex Units : {} \n\
         \x20  - Aggregate Deck Footprint: {} m²\n\
         \x20  - Corner Interlock Nodes  : {} (120-deg Y-Junctions)\n\
         \x20  - Intersection Thrusters  : {} Retractable Azimuth Swivel Pods\n\
         \x20  - Vertex Towers           : {} (degree ≥ 2, vertical hex-tube risers)\n\
         \x20  - Face Interconnects      : {} shared hex-tube faces\n\
         \x20  - Estimated Displacement  : {} t\n\
         \x20  - Stocked Kit Items       : {}\n\
         \n\
         2. DISCRETE INFRAMODULE MANIFEST\n\
         \x20  - [H1] Core Structural Infra-Modules : {}\n\
         \x20  - [FH] Articulating Flight-Harbors   : {} (Swing/Raise/Lower Face Gates)\n\
         \x20  - [DD] Drainable Ballast Dry-Docks   : {}\n\
         \x20  - [SH] Central Sub-Launch Moonpools  : {}\n\
         \n\
         3. KEY ARCHITECTURAL ADVANTAGES OVER RECTANGULAR BARGES\n\
         \x20  - Tri-axial wave bending moment dissipation across 120° intersections.\n\
         \x20  - Zero-tug station keeping via 3-way intersection tuck swivel propellers.\n\
         \x20  - Isolated ballast dry-docking without compromising adjacent flight ops.\n\
         \x20  - Multi-tier vertical scalability via corner floating interlock columns.\n\
         ====================================================================",
        t.total,
        fmt_int(t.deck_area_m2),
        t.interlock_nodes,
        t.thrusters,
        t.towers,
        t.edges,
        fmt_int(t.displacement_t),
        t.stocked,
        t.core,
        t.flight,
        t.drydock,
        t.subhub,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_carry_live_numbers() {
        let c = City::preset_carrier();
        let x = x_thread(&c);
        assert!(x.contains("6x Flight-Harbor"));
        assert!(x.contains("19 active hexes"));
        let spec = spec_sheet(&c);
        assert!(spec.contains("Total Modular Hex Units : 19"));
    }
}
