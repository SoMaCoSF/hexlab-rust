//! HexLab core — local Rust model of the modular floating Hex City.
//!
//! Ported faithfully from the live web build (SoMaCoSF/hexlab-platform):
//! `src/infra/hexTubeKit.ts` (hex-tube kit, channels, towers),
//! `src/state/platformStore.ts` (modules, sizes, presets),
//! `src/spoken/spokenBridge.ts` (tier-0 spoken grammar), and the Gemini
//! marine configurator's telemetry formulas. The USDCraft catalog that
//! Grok could not push (stub in the web repo) is rebuilt here in `catalog`.

pub mod axial;
pub mod catalog;
pub mod city;
pub mod export;
pub mod kit;
pub mod spoken;
pub mod tube;
pub mod types;

pub use axial::Axial;
pub use city::{City, Telemetry, VertexNode};
pub use types::{ModuleType, SizeClass};
