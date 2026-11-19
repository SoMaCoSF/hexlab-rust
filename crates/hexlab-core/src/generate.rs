//! Local generation — the reason this app is a local Rust app.
//!
//! anyMACHINE / anyKIT creation runs on the machine that has Blender.
//! [`run_job`] takes a queued hex-module job, drives Blender headless
//! with `scripts/hex_module.py`, lands `module.glb` in the job folder,
//! marks the manifest Ready, and registers the artifact in the HEX
//! module library. When Blender is absent the job stays Queued and the
//! outcome says so — no fake success, ever.

use crate::kit::{JobStatus, KitJob};
use crate::library::Library;
use crate::tube::{layout_for_cell, CellLengthClass};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    /// Artifact generated and registered in the library.
    Ready { artifact: PathBuf },
    /// Blender not found on this machine; job left Queued.
    BlenderNotFound,
    /// Job is not a hex-module generation job.
    NotGeneratable(String),
}

/// Locate a Blender executable: PATH first, then the standard install
/// locations per OS. Returns None when Blender is not installed —
/// callers report that honestly instead of pretending to generate.
pub fn find_blender() -> Option<PathBuf> {
    if let Ok(out) = Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg("blender")
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = text.lines().next() {
                let p = PathBuf::from(line.trim());
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }
    #[cfg(windows)]
    {
        for base in [r"C:\Program Files\Blender Foundation", r"C:\Program Files (x86)\Blender Foundation"] {
            if let Ok(rd) = std::fs::read_dir(base) {
                for e in rd.flatten() {
                    let p = e.path().join("blender.exe");
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let p = PathBuf::from("/Applications/Blender.app/Contents/MacOS/Blender");
        if p.exists() {
            return Some(p);
        }
    }
    #[cfg(unix)]
    {
        for p in ["/usr/bin/blender", "/usr/local/bin/blender", "/snap/bin/blender"] {
            let p = PathBuf::from(p);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

/// Run one queued job. `script` is `scripts/hex_module.py` from this
/// repo. On success the manifest is rewritten Ready and the library at
/// `<project>/library.json` gains the generated entry.
pub fn run_job(
    project_dir: &Path,
    job_id: &str,
    blender: Option<&Path>,
    script: &Path,
) -> Result<RunOutcome, String> {
    let mut job = KitJob::read_manifest(project_dir, job_id)?;
    let (Some(module), Some(size)) = (job.module, job.size) else {
        return Ok(RunOutcome::NotGeneratable(format!(
            "job '{job_id}' has no module/size — only hex-module jobs generate locally in v1"
        )));
    };
    let Some(blender) = blender.map(|p| p.to_path_buf()).or_else(find_blender) else {
        return Ok(RunOutcome::BlenderNotFound);
    };
    if !script.exists() {
        return Err(format!("generator script not found: {}", script.display()));
    }
    let layout = layout_for_cell(size, CellLengthClass::L40);
    let job_dir = project_dir.join(&job.out_dir);
    std::fs::create_dir_all(&job_dir).map_err(|e| e.to_string())?;
    let out_path = job_dir.join("module.glb");

    job.status = JobStatus::Submitted;
    job.write_manifest(project_dir).map_err(|e| e.to_string())?;

    let status = Command::new(&blender)
        .args(["--background", "--factory-startup", "--python"])
        .arg(script)
        .arg("--")
        .args([
            "--module",
            module.as_str(),
            "--size",
            size.as_str(),
            "--ff",
            &layout.flat_to_flat_m.to_string(),
            "--length",
            &layout.length_m.to_string(),
            "--corridor-ff",
            &layout.corridor_ff.to_string(),
            "--out",
        ])
        .arg(&out_path)
        .status()
        .map_err(|e| format!("failed to launch Blender ({}): {e}", blender.display()))?;

    if !status.success() || !out_path.exists() {
        job.status = JobStatus::Failed;
        job.write_manifest(project_dir).map_err(|e| e.to_string())?;
        return Err(format!(
            "Blender exited {status} without producing {}",
            out_path.display()
        ));
    }

    job.status = JobStatus::Ready;
    job.artifact = Some("module.glb".into());
    job.write_manifest(project_dir).map_err(|e| e.to_string())?;

    let mut lib = Library::load(project_dir)?;
    lib.register_job(&job);
    lib.save(project_dir)?;

    Ok(RunOutcome::Ready { artifact: out_path })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModuleType, SizeClass};

    #[test]
    fn non_module_job_is_not_generatable() {
        let dir = std::env::temp_dir().join("hexlab-generate-test-a");
        std::fs::create_dir_all(&dir).unwrap();
        let job = KitJob::new(crate::kit::KitKind::AnyKit, "brass compass kit", vec![], "steampunk");
        job.write_manifest(&dir).unwrap();
        let out = run_job(&dir, &job.id, None, Path::new("scripts/hex_module.py")).unwrap();
        assert!(matches!(out, RunOutcome::NotGeneratable(_)));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// End-to-end with a stub "blender" shell script that honours the
    /// --out contract: proves the runner, manifest transitions and
    /// library registration without a real Blender install.
    #[cfg(unix)]
    #[test]
    fn stub_blender_lands_artifact_and_registers() {
        let dir = std::env::temp_dir().join("hexlab-generate-test-b");
        std::fs::create_dir_all(&dir).unwrap();
        let stub = dir.join("fake-blender.sh");
        std::fs::write(
            &stub,
            "#!/bin/sh\nwhile [ $# -gt 0 ]; do if [ \"$1\" = \"--out\" ]; then echo GLB > \"$2\"; fi; shift; done\nexit 0\n",
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
        let script = dir.join("hex_module.py");
        std::fs::write(&script, "# stub\n").unwrap();

        let job = KitJob::module_job(ModuleType::Flight, SizeClass::Standard);
        job.write_manifest(&dir).unwrap();
        let out = run_job(&dir, &job.id, Some(&stub), &script).unwrap();
        let RunOutcome::Ready { artifact } = out else { panic!("expected Ready, got {out:?}") };
        assert!(artifact.exists());
        let back = KitJob::read_manifest(&dir, &job.id).unwrap();
        assert_eq!(back.status, JobStatus::Ready);
        assert_eq!(back.artifact.as_deref(), Some("module.glb"));
        let lib = Library::load(&dir).unwrap();
        assert_eq!(lib.entries.len(), 9); // 8 stock + 1 generated
        std::fs::remove_dir_all(&dir).ok();
    }
}
