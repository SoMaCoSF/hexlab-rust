//! anyMACHINE + anyKIT job model, and the AnyScene plan bridge.
//!
//! Honest split (from the thread): the pipelines themselves — AnyKit
//! (reference → prop kit, fal/Tripo/Meshy + Blender) and AnyMachine
//! (description → assembled machine rig, Blender/USD) — run *outside*
//! any UI. What runs fully local is the job: a folder under
//! `project/kits/<job-id>/` with a manifest, the BOM/plan that feeds the
//! pipeline, and a place for the returned GLB/USD assets to land.
//! Local agents (CLI/TUI/Tauri) create, inspect and stock from jobs
//! without a GUI or a network; generation itself is a separate step
//! that needs Blender installed and/or fal keys.

use crate::catalog::{asset_by_id, FacilityBom};
use crate::types::SizeClass;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KitKind {
    /// AnyKit: reference image → cohesive prop kit.
    AnyKit,
    /// AnyMachine: description → assembled machine with drive + pose.
    AnyMachine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Manifest written locally; nothing generated yet.
    Queued,
    /// Handed to a local/remote pipeline (Blender, fal).
    Submitted,
    /// Assets landed in the job folder.
    Ready,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KitJob {
    pub id: String,
    pub kind: KitKind,
    /// AnyKit: reference description/image note. AnyMachine: machine description.
    pub prompt: String,
    pub items: Vec<String>,
    pub style: String,
    pub status: JobStatus,
    /// Folder the job lives in, relative to the project root.
    pub out_dir: String,
    /// Hex-module generation jobs only: the inframodule to generate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<crate::types::ModuleType>,
    /// Hex-module generation jobs only: the size class to generate at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<SizeClass>,
    /// Landed artifact (e.g. `module.glb`), relative to the job folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
}

impl KitJob {
    pub fn new(kind: KitKind, prompt: &str, items: Vec<String>, style: &str) -> Self {
        let slug: String = prompt
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .take(4)
            .collect::<Vec<_>>()
            .join("-");
        let id = format!(
            "{}-{}",
            match kind {
                KitKind::AnyKit => "kit",
                KitKind::AnyMachine => "machine",
            },
            if slug.is_empty() { "job".to_string() } else { slug }
        );
        Self {
            out_dir: format!("kits/{id}"),
            id,
            kind,
            prompt: prompt.to_string(),
            items,
            style: style.to_string(),
            status: JobStatus::Queued,
            module: None,
            size: None,
            artifact: None,
        }
    }

    /// A hex-module generation job: AnyMachine builds the module shell
    /// locally in Blender (`generate::run_job`), and the result joins
    /// the HEX module library. This is the "creations" flow — the app
    /// is local precisely because Blender is local.
    pub fn module_job(module: crate::types::ModuleType, size: SizeClass) -> Self {
        let mut job = Self::new(
            KitKind::AnyMachine,
            &format!("hex module {} {}", module.as_str(), size.as_str()),
            vec![],
            "marine-industrial",
        );
        job.module = Some(module);
        job.size = Some(size);
        job
    }

    /// Read a job manifest back from `<project>/kits/<id>/job.json`.
    pub fn read_manifest(project_dir: &std::path::Path, job_id: &str) -> Result<Self, String> {
        let path = project_dir.join("kits").join(job_id).join("job.json");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("bad manifest {}: {e}", path.display()))
    }

    /// Write the job manifest into `<project>/kits/<id>/job.json`.
    /// Returns the manifest path. Creates no other files.
    pub fn write_manifest(&self, project_dir: &std::path::Path) -> std::io::Result<PathBuf> {
        let dir = project_dir.join(&self.out_dir);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("job.json");
        let json = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into());
        std::fs::write(&path, json)?;
        Ok(path)
    }
}

/// `facilityBomToAnyScenePlan` from the thread: turn a HexLab BOM into an
/// AnyScene-style plan fragment — room sized to the hex class, each BOM
/// line a scene object with footprint, `usdcraft_path` when the harvest
/// has the mesh (`generate: false`), otherwise `generate: true` for
/// AnyScene/AnyKit generation.
pub fn facility_bom_to_anyscene_plan(bom: &FacilityBom, size: SizeClass) -> serde_json::Value {
    let objects: Vec<serde_json::Value> = bom
        .lines
        .iter()
        .filter_map(|l| {
            let a = asset_by_id(l.asset_id)?;
            Some(serde_json::json!({
                "asset_id": a.id,
                "name": a.name,
                "qty": l.qty,
                "w": a.footprint.0,
                "d": a.footprint.1,
                "h": a.footprint.2,
                "usdcraft_path": a.usdcraft_path,
                "generate": a.usdcraft_path.is_none(),
            }))
        })
        .collect();
    serde_json::json!({
        "facility": bom.id,
        "label": bom.label,
        "room": {
            "shape": "hex",
            "size_class": size.as_str(),
            "flat_to_flat_m": size.ff_m(),
            "area_m2": size.area_m2(),
            "up": "Z",
        },
        "objects": objects,
    })
}

/// AnyScene plan for a single HEX module: the room IS the hex, and the
/// objects are the kit's six primary channel wedges plus the corridor
/// spine — the BOM a module is kitted from before fit-out. This is how
/// anyscene "builds a BOM and kits it out" for any module, not just
/// whole facilities.
pub fn module_to_anyscene_plan(
    module: crate::types::ModuleType,
    size: SizeClass,
) -> serde_json::Value {
    let layout = crate::tube::layout_for_cell(size, crate::tube::CellLengthClass::L40);
    let wedges: Vec<serde_json::Value> = crate::tube::CHANNELS
        .iter()
        .take(6)
        .enumerate()
        .map(|(i, c)| {
            serde_json::json!({
                "asset_id": format!("channel_wedge_{}", c.id.as_str()),
                "name": format!("{} wedge", c.id.as_str()),
                "wedge_index": i,
                "channel": c.id.as_str(),
                "w": layout.wedge_depth_m,
                "d": layout.length_m,
                "generate": true,
            })
        })
        .collect();
    serde_json::json!({
        "module": module.as_str(),
        "code": module.code(),
        "room": {
            "shape": "hex",
            "size_class": size.as_str(),
            "flat_to_flat_m": size.ff_m(),
            "area_m2": size.area_m2(),
            "corridor_ff_m": layout.corridor_ff,
            "length_m": layout.length_m,
            "up": "Z",
        },
        "shell": {
            "generator": "scripts/hex_module.py (Blender, local)",
            "artifact": "module.glb",
        },
        "objects": wedges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::get_facility_bom;

    #[test]
    fn woodshop_plan() {
        let plan = facility_bom_to_anyscene_plan(get_facility_bom("woodshop").unwrap(), SizeClass::Standard);
        assert_eq!(plan["facility"], "woodshop");
        assert_eq!(plan["room"]["flat_to_flat_m"], 32.0);
        let objs = plan["objects"].as_array().unwrap();
        assert_eq!(objs.len(), 8);
        assert_eq!(objs[0]["asset_id"], "band_saw");
        assert_eq!(objs[0]["generate"], false);
    }

    #[test]
    fn job_manifest_round_trip() {
        let job = KitJob::new(KitKind::AnyMachine, "hydraulic press line", vec!["hydraulic_press".into()], "marine-industrial");
        assert_eq!(job.id, "machine-hydraulic-press-line");
        assert_eq!(job.status, JobStatus::Queued);
        let dir = std::env::temp_dir().join("hexlab-kit-test");
        let path = job.write_manifest(&dir).unwrap();
        assert!(path.exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
