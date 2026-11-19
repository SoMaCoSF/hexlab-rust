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
        }
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
