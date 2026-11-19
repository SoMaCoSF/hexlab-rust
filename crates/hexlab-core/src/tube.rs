//! Hex-tube kit — a 1:1 Rust port of `src/infra/hexTubeKit.ts`.
//!
//! Horizontal cell = hexagonal tube module (shipping-container modularity,
//! hex section). Vertex tower = same kit, vertical axis, larger diameter —
//! shared risers at 120° junctions. Cross-section of a cell: centre corridor,
//! six radial utility wedges, face couplings mid-edge, vertex nodes feeding
//! the vertical tower stack.

use crate::types::SizeClass;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Power,
    Freshwater,
    Ballast,
    Seawater,
    Hvac,
    Data,
    Waste,
    Fire,
    Fuel,
    Life,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelSpec {
    pub id: Channel,
    pub label: &'static str,
    pub color: &'static str,
    /// Wedge index 0–5 around the hex (flat-centred).
    pub wedge: u8,
    pub description: &'static str,
}

/// Six wedges — one primary service per angle of the hex tube.
pub const CHANNELS: [ChannelSpec; 6] = [
    ChannelSpec { id: Channel::Power, label: "Power", color: "#fbbf24", wedge: 0, description: "MV/LV bus, switchgear risers" },
    ChannelSpec { id: Channel::Freshwater, label: "Fresh water", color: "#22d3ee", wedge: 1, description: "Potable + process supply" },
    ChannelSpec { id: Channel::Ballast, label: "Ballast", color: "#3b82f6", wedge: 2, description: "Trim tanks, flood/drain" },
    ChannelSpec { id: Channel::Hvac, label: "HVAC / air", color: "#94a3b8", wedge: 3, description: "Supply / exhaust / purge" },
    ChannelSpec { id: Channel::Data, label: "Data / fiber", color: "#a855f7", wedge: 4, description: "Backbone + local loops" },
    ChannelSpec { id: Channel::Waste, label: "Waste / grey", color: "#78716c", wedge: 5, description: "Collection, treatment feed" },
];

pub const EXTRA_CHANNELS: [ChannelSpec; 4] = [
    ChannelSpec { id: Channel::Seawater, label: "Seawater", color: "#0ea5e9", wedge: 2, description: "Intake / cooling loop" },
    ChannelSpec { id: Channel::Fire, label: "Fire main", color: "#ef4444", wedge: 0, description: "Pressurized fire ring" },
    ChannelSpec { id: Channel::Fuel, label: "Fuel", color: "#f97316", wedge: 1, description: "Bunker / process feedstock" },
    ChannelSpec { id: Channel::Life, label: "Life support", color: "#10b981", wedge: 3, description: "O₂, medical gas, alarms" },
];

pub fn channel_spec(id: Channel) -> ChannelSpec {
    CHANNELS
        .iter()
        .chain(EXTRA_CHANNELS.iter())
        .find(|c| c.id == id)
        .copied()
        .expect("all channels have specs")
}

impl Channel {
    pub const ALL: [Self; 10] = [
        Self::Power, Self::Freshwater, Self::Ballast, Self::Seawater, Self::Hvac,
        Self::Data, Self::Waste, Self::Fire, Self::Fuel, Self::Life,
    ];
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Power => "power",
            Self::Freshwater => "freshwater",
            Self::Ballast => "ballast",
            Self::Seawater => "seawater",
            Self::Hvac => "hvac",
            Self::Data => "data",
            Self::Waste => "waste",
            Self::Fire => "fire",
            Self::Fuel => "fuel",
            Self::Life => "life",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Channel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .find(|c| c.as_str() == s.to_ascii_lowercase())
            .copied()
            .ok_or_else(|| format!("unknown channel '{s}'"))
    }
}

/// Standard cell lengths — analogous to ISO container lengths, hex envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellLengthClass {
    L20,
    L40,
    L45,
}

impl CellLengthClass {
    pub fn length_m(&self) -> f64 {
        match self {
            Self::L20 => 6.1,
            Self::L40 => 12.2,
            Self::L45 => 13.7,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::L20 => "20",
            Self::L40 => "40",
            Self::L45 => "45",
        }
    }
}

impl FromStr for CellLengthClass {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "20" => Ok(Self::L20),
            "40" => Ok(Self::L40),
            "45" => Ok(Self::L45),
            other => Err(format!("unknown cell length '{other}' (20|40|45)")),
        }
    }
}

/// Vertex towers = same module family, vertical, larger diameter.
pub const TOWER_FF_FACTOR: f64 = 1.35;
pub const STORY_HEIGHT_M: f64 = 4.2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TubeSectionLayout {
    pub flat_to_flat_m: f64,
    pub corridor_ff: f64,
    pub wedge_depth_m: f64,
    pub wall_m: f64,
    pub length_m: f64,
    #[serde(skip)]
    pub channels: Vec<ChannelSpec>,
}

impl TubeSectionLayout {
    pub fn channel_ids(&self) -> Vec<Channel> {
        self.channels.iter().map(|c| c.id).collect()
    }
}

/// Cross-section layout for a horizontal cell. Interior corridor is ~0.28 of
/// flat-to-flat so people + carts pass the spine.
pub fn layout_for_cell(size: SizeClass, length: CellLengthClass) -> TubeSectionLayout {
    let ff = size.ff_m();
    let corridor_ff = ff * 0.28;
    let wall_m = (ff * 0.02).max(0.25);
    TubeSectionLayout {
        flat_to_flat_m: ff,
        corridor_ff,
        wedge_depth_m: (ff - corridor_ff) / 2.0 - wall_m,
        wall_m,
        length_m: length.length_m(),
        channels: CHANNELS.to_vec(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TowerLayout {
    #[serde(flatten)]
    pub section: TubeSectionLayout,
    pub height_m: f64,
    pub stories: u32,
}

/// Cross-section layout for a vertical vertex tower: same kit, upscaled by
/// [`TOWER_FF_FACTOR`], corridor ratio 0.32, story height 4.2 m.
pub fn layout_for_tower(size: SizeClass, stories: u32) -> TowerLayout {
    let base = layout_for_cell(size, CellLengthClass::L40);
    let ff = base.flat_to_flat_m * TOWER_FF_FACTOR;
    let corridor_ff = ff * 0.32;
    let wall_m = (ff * 0.025).max(0.35);
    TowerLayout {
        section: TubeSectionLayout {
            flat_to_flat_m: ff,
            corridor_ff,
            wedge_depth_m: (ff - corridor_ff) / 2.0 - wall_m,
            wall_m,
            length_m: base.length_m,
            channels: CHANNELS.to_vec(),
        },
        height_m: stories as f64 * STORY_HEIGHT_M,
        stories,
    }
}

/// Face coupling: which channels pass cell-to-cell across a shared edge.
pub const DEFAULT_FACE_COUPLING: [Channel; 6] = [
    Channel::Power,
    Channel::Freshwater,
    Channel::Ballast,
    Channel::Hvac,
    Channel::Data,
    Channel::Waste,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_layout_matches_ts() {
        let l = layout_for_cell(SizeClass::Standard, CellLengthClass::L40);
        assert!((l.flat_to_flat_m - 32.0).abs() < 1e-9);
        assert!((l.corridor_ff - 8.96).abs() < 1e-9);
        assert!((l.wall_m - 0.64).abs() < 1e-9);
        assert!((l.length_m - 12.2).abs() < 1e-9);
        assert_eq!(l.channels.len(), 6);
    }

    #[test]
    fn tower_layout_matches_ts() {
        let t = layout_for_tower(SizeClass::Industrial, 6);
        assert!((t.section.flat_to_flat_m - 86.4).abs() < 1e-9);
        assert!((t.height_m - 25.2).abs() < 1e-9);
        assert_eq!(t.stories, 6);
    }
}
