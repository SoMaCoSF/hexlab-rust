//! The HEX module library — the creations store.
//!
//! Sam's architecture (2026-11-18): the local Rust app exists so the
//! anyMACHINE / anyKIT creations have somewhere to live — a library of
//! HEX-based modules. Built-in inframodules are the stock shelf; every
//! generation job that lands (Blender shell, AnyKit prop kit, AnyScene
//! fit-out) registers here, and the city builder, the spoken UX and
//! the BOM flow all draw from the same library. Persisted as
//! `<project>/library.json`; spoken-world speaks to this list, it does
//! not keep its own.

use crate::kit::{JobStatus, KitJob};
use crate::types::{ModuleType, SizeClass};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    /// One of the eight built-in inframodules (the stock shelf).
    Module,
    /// A USDCraft catalog asset (anyKIT shelf).
    Asset,
    /// A landed generation artifact (GLB/USD in a job folder).
    Generated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntrySource {
    Builtin,
    AnyKit,
    AnyMachine,
    AnyScene,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id: String,
    pub label: String,
    pub kind: EntryKind,
    pub source: EntrySource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<ModuleType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<SizeClass>,
    /// Artifact path relative to the project root, when one has landed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Library {
    pub entries: Vec<LibraryEntry>,
}

impl Library {
    /// The stock shelf: the eight inframodules, before any generation.
    pub fn stock() -> Self {
        let entries = ModuleType::ALL
            .iter()
            .map(|m| LibraryEntry {
                id: format!("module-{}", m.as_str()),
                label: format!("{} {}", m.code(), m.as_str()),
                kind: EntryKind::Module,
                source: EntrySource::Builtin,
                module: Some(*m),
                size: None,
                artifact: None,
            })
            .collect();
        Self { entries }
    }

    pub fn path(project_dir: &Path) -> PathBuf {
        project_dir.join("library.json")
    }

    /// Load the project library, or the stock shelf when none exists yet.
    pub fn load(project_dir: &Path) -> Result<Self, String> {
        let path = Self::path(project_dir);
        if !path.exists() {
            return Ok(Self::stock());
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("bad library {}: {e}", path.display()))
    }

    pub fn save(&self, project_dir: &Path) -> Result<PathBuf, String> {
        let path = Self::path(project_dir);
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        Ok(path)
    }

    /// Register a landed generation job. Idempotent per job id: a
    /// re-run updates the entry instead of duplicating it. Jobs that
    /// have not landed (not Ready, no artifact) do not register.
    pub fn register_job(&mut self, job: &KitJob) -> bool {
        if job.status != JobStatus::Ready {
            return false;
        }
        let Some(artifact) = &job.artifact else { return false };
        let id = format!("gen-{}", job.id);
        let entry = LibraryEntry {
            label: job.prompt.clone(),
            kind: EntryKind::Generated,
            source: match job.kind {
                crate::kit::KitKind::AnyKit => EntrySource::AnyKit,
                crate::kit::KitKind::AnyMachine => EntrySource::AnyMachine,
            },
            module: job.module,
            size: job.size,
            artifact: Some(format!("{}/{}", job.out_dir, artifact)),
            id: id.clone(),
        };
        if let Some(existing) = self.entries.iter_mut().find(|e| e.id == id) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
        true
    }

    /// Fuzzy find by id / label / module name — the lookup the spoken
    /// UX uses ("stock the machine shop", "place a flight hex").
    pub fn find(&self, query: &str) -> Vec<&LibraryEntry> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.id.to_lowercase().contains(&q)
                    || e.label.to_lowercase().contains(&q)
                    || e.module.map(|m| m.as_str().contains(&q)).unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::KitKind;

    #[test]
    fn stock_shelf_has_eight_modules() {
        let lib = Library::stock();
        assert_eq!(lib.entries.len(), 8);
        assert_eq!(lib.find("flight").len(), 1);
        assert_eq!(lib.find("flight")[0].module, Some(ModuleType::Flight));
    }

    #[test]
    fn only_ready_jobs_register() {
        let mut lib = Library::stock();
        let mut job = KitJob::module_job(ModuleType::Core, SizeClass::Standard);
        assert!(!lib.register_job(&job)); // still queued
        job.status = JobStatus::Ready;
        job.artifact = Some("module.glb".into());
        assert!(lib.register_job(&job));
        assert_eq!(lib.entries.len(), 9);
        // Idempotent re-register.
        assert!(lib.register_job(&job));
        assert_eq!(lib.entries.len(), 9);
        let gen = lib.find("gen-").pop().unwrap();
        assert_eq!(gen.source, EntrySource::AnyMachine);
        assert_eq!(gen.kind, EntryKind::Generated);
        assert_eq!(gen.artifact.as_deref(), Some("kits/machine-hex-module-core-standard/module.glb"));
        let _ = KitKind::AnyKit; // kind coverage
    }

    #[test]
    fn library_round_trip() {
        let dir = std::env::temp_dir().join("hexlab-library-test");
        std::fs::create_dir_all(&dir).unwrap();
        let lib = Library::stock();
        lib.save(&dir).unwrap();
        let back = Library::load(&dir).unwrap();
        assert_eq!(back.entries.len(), 8);
        std::fs::remove_dir_all(&dir).ok();
    }
}
