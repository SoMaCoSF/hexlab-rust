//! The axial city array: cells, face couplings, vertex towers, telemetry.
//!
//! Presets are the `platformStore.ts` versions (the later city build —
//! carrier ring 2 carries refinery/desal/lab; recon is scout-sized with a
//! power hex). Telemetry formulas are the marine configurator's, generalised
//! per size class by area scale.

use crate::axial::Axial;
use crate::tube::{Channel, DEFAULT_FACE_COUPLING};
use crate::types::{ModuleType, SizeClass};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub module: ModuleType,
    pub size: SizeClass,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StockedItem {
    pub asset_id: String,
    /// Cell key ("q,r") the item was stocked onto, if any.
    pub cell: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CitySettings {
    /// Flight-harbor face gate, 0.0 sealed → 1.0 launch ramp (90°).
    pub gate_open: f64,
    /// Dry-dock ballast, 0.0 drained → 1.0 flooded.
    pub ballast: f64,
    pub show_thrusters: bool,
    pub show_interlocks: bool,
    pub show_stack: bool,
    pub show_waves: bool,
}

impl Default for CitySettings {
    fn default() -> Self {
        Self {
            gate_open: 0.75,
            ballast: 0.0,
            show_thrusters: true,
            show_interlocks: true,
            show_stack: true,
            show_waves: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct City {
    pub cells: BTreeMap<String, Cell>,
    #[serde(default)]
    pub stocked: Vec<StockedItem>,
    #[serde(default)]
    pub settings: CitySettings,
}

impl City {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, at: Axial) -> Option<&Cell> {
        self.cells.get(&at.key())
    }

    pub fn place(&mut self, at: Axial, module: ModuleType, size: SizeClass) {
        self.cells.insert(at.key(), Cell { module, size });
    }

    pub fn erase(&mut self, at: Axial) -> bool {
        self.cells.remove(&at.key()).is_some()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Axial, &Cell)> {
        self.cells.iter().map(|(k, c)| {
            let mut it = k.split(',');
            let q: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
            let r: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
            (Axial::new(q, r), c)
        })
    }

    pub fn count(&self, module: ModuleType) -> usize {
        self.cells.values().filter(|c| c.module == module).count()
    }

    pub fn counts(&self) -> BTreeMap<ModuleType, usize> {
        let mut out = BTreeMap::new();
        for m in ModuleType::ALL {
            out.insert(m, self.count(m));
        }
        out
    }

    /// 7-hex recon outpost (scout): centre sub-hub, ring per platformStore.
    pub fn preset_recon() -> Self {
        let s = SizeClass::Scout;
        let mut city = Self::new();
        city.place(Axial::new(0, 0), ModuleType::SubHub, s);
        city.place(Axial::new(1, 0), ModuleType::Flight, s);
        city.place(Axial::new(0, 1), ModuleType::Core, s);
        city.place(Axial::new(-1, 1), ModuleType::DryDock, s);
        city.place(Axial::new(-1, 0), ModuleType::Flight, s);
        city.place(Axial::new(0, -1), ModuleType::Power, s);
        city.place(Axial::new(1, -1), ModuleType::Core, s);
        city
    }

    /// 19-hex carrier hub (standard): centre sub-hub, core ring 1,
    /// mixed ring 2 per platformStore.
    pub fn preset_carrier() -> Self {
        let s = SizeClass::Standard;
        let mut city = Self::new();
        city.place(Axial::new(0, 0), ModuleType::SubHub, s);
        for (q, r) in [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)] {
            city.place(Axial::new(q, r), ModuleType::Core, s);
        }
        let ring2: [(i32, i32, ModuleType); 12] = [
            (2, 0, ModuleType::Flight),
            (1, 1, ModuleType::Core),
            (0, 2, ModuleType::Flight),
            (-1, 2, ModuleType::DryDock),
            (-2, 2, ModuleType::Flight),
            (-2, 1, ModuleType::Refinery),
            (-2, 0, ModuleType::Flight),
            (-1, -1, ModuleType::Desal),
            (0, -2, ModuleType::Flight),
            (1, -2, ModuleType::DryDock),
            (2, -2, ModuleType::Flight),
            (2, -1, ModuleType::Lab),
        ];
        for (q, r, m) in ring2 {
            city.place(Axial::new(q, r), m, s);
        }
        city
    }

    pub fn preset(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "recon" => Some(Self::preset_recon()),
            "carrier" => Some(Self::preset_carrier()),
            "sandbox" => Some(Self::new()),
            _ => None,
        }
    }

    /// Shared faces between adjacent cells — the horizontal tube
    /// interconnects. Each carries the default face-coupling channels.
    pub fn edges(&self) -> Vec<(Axial, Axial)> {
        let mut out = Vec::new();
        for (at, _) in self.iter() {
            for dir in 0..3 {
                // Only 3 dirs so each shared face is counted once.
                let n = at.neighbor(dir);
                if self.get(n).is_some() {
                    out.push((at, n));
                }
            }
        }
        out
    }

    /// Vertex interconnect matrix — when 2+ cells share a vertex, the
    /// vertical tower inherits the union of channels and upsizes.
    /// Port of the configurator's vertex map + hexTubeKit's VertexNode.
    pub fn vertices(&self) -> Vec<VertexNode> {
        let mut map: BTreeMap<String, (f64, f64, usize, SizeClass)> = BTreeMap::new();
        for (at, cell) in self.iter() {
            for (vx, vz) in at.vertices_world(cell.size.ff_m()) {
                let key = format!("{:.2},{:.2}", vx, vz);
                let e = map.entry(key).or_insert((vx, vz, 0, cell.size));
                e.2 += 1;
                if cell.size > e.3 {
                    e.3 = cell.size;
                }
            }
        }
        map.into_iter()
            .map(|(key, (x, z, degree, size))| VertexNode {
                key,
                x,
                z,
                degree,
                channels: DEFAULT_FACE_COUPLING.to_vec(),
                tower_ff: size.ff_m() * crate::tube::TOWER_FF_FACTOR,
                size,
                has_thruster: degree >= 2,
            })
            .collect()
    }

    /// Vertex towers: the vertices where 2+ cells meet (degree ≥ 2).
    pub fn towers(&self) -> Vec<VertexNode> {
        self.vertices().into_iter().filter(|v| v.degree >= 2).collect()
    }

    /// First cell (stocking-priority order, then axial) a facility prefers.
    pub fn preferred_stock_cell(&self) -> Option<String> {
        let mut cells: Vec<(Axial, &Cell)> = self.iter().collect();
        cells.sort_by_key(|(_, c)| c.module.stocking_priority());
        cells.first().map(|(at, _)| at.key())
    }

    pub fn telemetry(&self) -> Telemetry {
        let vertices = self.vertices();
        let towers = vertices.iter().filter(|v| v.degree >= 2).count();
        let total = self.len();
        let core = self.count(ModuleType::Core);

        let mut area = 0.0f64;
        let mut displacement = 0.0f64;
        for (_, cell) in self.iter() {
            area += cell.size.area_m2();
            displacement += 2200.0 * cell.size.scale();
        }
        let (stack_area, stack_disp) = if self.settings.show_stack {
            let mut a = 0.0;
            let mut d = 0.0;
            for (_, cell) in self.iter() {
                if cell.module == ModuleType::Core {
                    a += 680.0 * cell.size.scale();
                    d += 380.0 * cell.size.scale();
                }
            }
            (a, d)
        } else {
            (0.0, 0.0)
        };
        area += stack_area;
        displacement += stack_disp;

        let thrusters = if self.settings.show_thrusters { towers } else { 0 };
        let stability = if total == 0 {
            0.0
        } else {
            (84.0 + (towers as f64 / total as f64) * 8.5).min(99.4)
        };
        let bays = ModuleType::ALL
            .iter()
            .filter(|m| m.is_bay())
            .map(|m| self.count(*m))
            .sum();

        Telemetry {
            total,
            deck_area_m2: area,
            interlock_nodes: vertices.len(),
            towers,
            thrusters,
            displacement_t: displacement,
            bays,
            stability_pct: stability,
            core,
            flight: self.count(ModuleType::Flight),
            drydock: self.count(ModuleType::DryDock),
            subhub: self.count(ModuleType::SubHub),
            edges: self.edges().len(),
            stocked: self.stocked.len(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// A vertex where cell corners meet. Degree ≥ 2 stands a tower here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexNode {
    pub key: String,
    pub x: f64,
    pub z: f64,
    pub degree: usize,
    pub channels: Vec<Channel>,
    /// Tower flat-to-flat: largest meeting cell × TOWER_FF_FACTOR.
    pub tower_ff: f64,
    pub size: SizeClass,
    pub has_thruster: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Telemetry {
    pub total: usize,
    pub deck_area_m2: f64,
    pub interlock_nodes: usize,
    pub towers: usize,
    pub thrusters: usize,
    pub displacement_t: f64,
    pub bays: usize,
    pub stability_pct: f64,
    pub core: usize,
    pub flight: usize,
    pub drydock: usize,
    pub subhub: usize,
    pub edges: usize,
    pub stocked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_right_counts() {
        assert_eq!(City::preset_recon().len(), 7);
        let carrier = City::preset_carrier();
        assert_eq!(carrier.len(), 19);
        assert_eq!(carrier.count(ModuleType::Core), 7);
        assert_eq!(carrier.count(ModuleType::Flight), 6);
        assert_eq!(carrier.count(ModuleType::DryDock), 2);
        assert_eq!(carrier.count(ModuleType::SubHub), 1);
        assert_eq!(carrier.count(ModuleType::Refinery), 1);
        assert_eq!(carrier.count(ModuleType::Desal), 1);
        assert_eq!(carrier.count(ModuleType::Lab), 1);
    }

    #[test]
    fn carrier_telemetry_invariants() {
        let t = City::preset_carrier().telemetry();
        assert_eq!(t.total, 19);
        assert_eq!(t.bays, 9);
        assert!(t.towers > 0, "carrier must stand vertex towers");
        assert_eq!(t.thrusters, t.towers);
        assert!(t.stability_pct > 84.0 && t.stability_pct <= 99.4);
        // 19 standard hexes: 19*886.8 base area + 7 core stacks * 680.
        assert!((t.deck_area_m2 - (19.0 * 886.81 + 7.0 * 680.0)).abs() < 5.0);
    }

    #[test]
    fn json_round_trip() {
        let c = City::preset_recon();
        let back = City::from_json(&c.to_json()).unwrap();
        assert_eq!(back.len(), 7);
        assert_eq!(back.count(ModuleType::Power), 1);
    }
}
