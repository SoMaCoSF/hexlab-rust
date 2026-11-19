//! Size classes and module types — from `platformStore.ts` (canonical).

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Physical scale class — flat-to-flat metres (concept scale).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SizeClass {
    Scout,
    Standard,
    Industrial,
    Capital,
}

impl SizeClass {
    pub const ALL: [Self; 4] = [Self::Scout, Self::Standard, Self::Industrial, Self::Capital];
    pub fn ff_m(&self) -> f64 {
        match self {
            Self::Scout => 16.0,
            Self::Standard => 32.0,
            Self::Industrial => 64.0,
            Self::Capital => 128.0,
        }
    }
    /// Hex area in m²: (√3/2)·FF² — 886 m² at standard, as in the configurator.
    pub fn area_m2(&self) -> f64 {
        (3f64.sqrt() / 2.0) * self.ff_m() * self.ff_m()
    }
    /// Displacement scale relative to a standard hex.
    pub fn scale(&self) -> f64 {
        self.area_m2() / SizeClass::Standard.area_m2()
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scout => "scout",
            Self::Standard => "standard",
            Self::Industrial => "industrial",
            Self::Capital => "capital",
        }
    }
}

impl fmt::Display for SizeClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SizeClass {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "scout" | "16" => Ok(Self::Scout),
            "standard" | "32" => Ok(Self::Standard),
            "industrial" | "64" => Ok(Self::Industrial),
            "capital" | "128" => Ok(Self::Capital),
            other => Err(format!("unknown size class '{other}' (scout|standard|industrial|capital)")),
        }
    }
}

/// The inframodule set. First four are the marine configurator's; the
/// remaining four are the city build-out from `platformStore.ts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleType {
    Core,
    Flight,
    DryDock,
    SubHub,
    Refinery,
    Desal,
    Lab,
    Power,
}

impl ModuleType {
    pub const ALL: [Self; 8] = [
        Self::Core, Self::Flight, Self::DryDock, Self::SubHub,
        Self::Refinery, Self::Desal, Self::Lab, Self::Power,
    ];
    pub fn code(&self) -> &'static str {
        match self {
            Self::Core => "H1",
            Self::Flight => "FH",
            Self::DryDock => "DD",
            Self::SubHub => "SH",
            Self::Refinery => "RF",
            Self::Desal => "DS",
            Self::Lab => "LB",
            Self::Power => "PW",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Core => "Core Infra-Module",
            Self::Flight => "Flight-Harbor",
            Self::DryDock => "Drainable Dry-Dock",
            Self::SubHub => "Sub-Launching Harbor",
            Self::Refinery => "Refinery",
            Self::Desal => "Desalination",
            Self::Lab => "Lab",
            Self::Power => "Power",
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Flight => "flight",
            Self::DryDock => "drydock",
            Self::SubHub => "subhub",
            Self::Refinery => "refinery",
            Self::Desal => "desal",
            Self::Lab => "lab",
            Self::Power => "power",
        }
    }
    /// Is this one of the autonomous-operations bays counted in telemetry?
    pub fn is_bay(&self) -> bool {
        matches!(self, Self::Flight | Self::DryDock | Self::SubHub)
    }
    /// Module kinds a facility BOM prefers to stock onto (sensible defaults).
    pub fn stocking_priority(&self) -> u8 {
        match self {
            Self::Lab => 0,
            Self::Core => 1,
            Self::Power => 2,
            Self::Refinery => 3,
            _ => 4,
        }
    }
}

impl fmt::Display for ModuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ModuleType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "core" | "h1" => Ok(Self::Core),
            "flight" | "fh" | "flight-harbor" | "flightharbor" => Ok(Self::Flight),
            "drydock" | "dd" | "dry-dock" => Ok(Self::DryDock),
            "subhub" | "sh" | "sub" | "sub-harbor" => Ok(Self::SubHub),
            "refinery" | "rf" => Ok(Self::Refinery),
            "desal" | "ds" => Ok(Self::Desal),
            "lab" | "lb" => Ok(Self::Lab),
            "power" | "pw" => Ok(Self::Power),
            other => Err(format!(
                "unknown module '{other}' (core|flight|drydock|subhub|refinery|desal|lab|power)"
            )),
        }
    }
}
