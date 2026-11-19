//! `hexlab` — agent-facing CLI for the local HexLab.
//!
//! Agents on Omarchy (or anywhere) drive the city without a GUI:
//!   hexlab bom woodshop
//!   hexlab city preset recon -o city.json
//!   hexlab city towers city.json
//!   hexlab spoken "create a bom for a woodshop and stock it" --city city.json
//!   hexlab kit plan woodshop -o plan.json
//!   hexlab kit module flight --size standard   (queue a module creation)
//!   hexlab kit run machine-hex-module-flight-standard   (Blender, local)
//!   hexlab library   (the HEX module library: stock + generated)

use clap::{Parser, Subcommand};
use hexlab_core::axial::Axial;
use hexlab_core::catalog::{
    asset_by_id, asset_hf_url, get_facility_bom, FACILITY_BOMS, INDUSTRIAL_ASSETS,
};
use hexlab_core::city::{City, StockedItem};
use hexlab_core::generate::{run_job, RunOutcome};
use hexlab_core::kit::{facility_bom_to_anyscene_plan, module_to_anyscene_plan, KitJob, KitKind};
use hexlab_core::library::Library;
use hexlab_core::spoken::{help_text, parse_spoken, SpokenOp};
use hexlab_core::tube::{
    channel_spec, layout_for_cell, layout_for_tower, CellLengthClass, Channel, CHANNELS,
    EXTRA_CHANNELS,
};
use hexlab_core::types::{ModuleType, SizeClass};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "hexlab", about = "HexLab local — modular floating Hex City CAD core + agent CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Facility BOM (woodshop|machineshop|fabrication|warehouse|process|power|robotics)
    Bom {
        facility: String,
        #[arg(long)]
        json: bool,
    },
    /// AnyKIT catalog assets (USDCraft-10K industrial harvest)
    Assets {
        #[arg(long)]
        json: bool,
    },
    /// Tube + vertex tower layout for a size class
    Layout {
        #[arg(default_value = "standard")]
        size: SizeClass,
        #[arg(long, default_value = "40")]
        length: CellLengthClass,
        #[arg(long, default_value_t = 4)]
        stories: u32,
    },
    /// Utility channel list (6 primary wedges + 4 extra)
    Channels,
    /// City file operations
    City {
        #[command(subcommand)]
        cmd: CityCmd,
    },
    /// Parse (and optionally apply) a spoken-world style command
    Spoken {
        text: String,
        #[arg(long)]
        city: Option<PathBuf>,
    },
    /// anyKIT / anyMACHINE job utilities
    Kit {
        #[command(subcommand)]
        cmd: KitCmd,
    },
    /// The HEX module library (stock modules + generated creations)
    Library {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum CityCmd {
    /// Empty city JSON
    New {
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Preset city JSON (recon|carrier|sandbox)
    Preset {
        name: String,
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Place a module: hexlab city place FILE q r MODULE [--size SIZE]
    Place {
        file: PathBuf,
        q: i32,
        r: i32,
        module: ModuleType,
        #[arg(long, default_value = "standard")]
        size: SizeClass,
    },
    /// Remove a cell
    Erase { file: PathBuf, q: i32, r: i32 },
    /// Vertex towers (degree ≥ 2)
    Towers { file: PathBuf },
    /// Dump JSON
    Show { file: PathBuf },
    /// Live array telemetry
    Telemetry {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Export brief: --format xthread|spec
    Export {
        file: PathBuf,
        #[arg(long, default_value = "spec")]
        format: String,
    },
}

#[derive(Subcommand)]
enum KitCmd {
    /// Export an AnyScene plan JSON for a facility BOM
    Plan {
        facility: String,
        #[arg(long, default_value = "standard")]
        size: SizeClass,
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Create a local anyKIT / anyMACHINE job folder + manifest
    Job {
        /// anykit | anymachine
        kind: String,
        prompt: String,
        #[arg(long, default_value = "marine-industrial")]
        style: String,
        #[arg(long = "item")]
        items: Vec<String>,
        /// Project root the job folder lands under (kits/<id>/)
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Export an AnyScene plan for a single HEX module (shell + channel-wedge BOM)
    PlanModule {
        module: ModuleType,
        #[arg(long, default_value = "standard")]
        size: SizeClass,
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Queue a HEX module generation job (AnyMachine, local Blender)
    Module {
        module: ModuleType,
        #[arg(long, default_value = "standard")]
        size: SizeClass,
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Run a queued hex-module job on the local Blender; lands module.glb + registers it in the library
    Run {
        job_id: String,
        #[arg(long, default_value = ".")]
        project: PathBuf,
        /// Blender executable (default: find on PATH / standard locations)
        #[arg(long)]
        blender: Option<PathBuf>,
        /// Generator script (scripts/hex_module.py in this repo)
        #[arg(long, default_value = "scripts/hex_module.py")]
        script: PathBuf,
    },
}

fn load_city(path: &Path) -> Result<City, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    City::from_json(&text).map_err(|e| format!("cannot parse {}: {e}", path.display()))
}

fn save_city(city: &City, path: &Path) -> Result<(), String> {
    std::fs::write(path, city.to_json())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

fn print_bom(facility_id: &str, json: bool) -> Result<(), String> {
    let bom = get_facility_bom(facility_id)
        .ok_or_else(|| format!("unknown facility '{facility_id}' (woodshop|machineshop|fabrication|warehouse|process|power|robotics)"))?;
    if json {
        let lines: Vec<_> = bom
            .lines
            .iter()
            .map(|l| {
                let a = asset_by_id(l.asset_id);
                serde_json::json!({
                    "asset_id": l.asset_id,
                    "name": a.map(|a| a.name),
                    "qty": l.qty,
                    "footprint_m": a.map(|a| a.footprint),
                    "usdcraft": a.and_then(asset_hf_url),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "facility": bom.id, "label": bom.label, "lines": lines
            }))
            .unwrap()
        );
        return Ok(());
    }
    println!("BOM // {} ({})", bom.label.to_uppercase(), bom.id);
    let mut total = 0;
    for l in bom.lines {
        let a = asset_by_id(l.asset_id);
        total += l.qty;
        println!(
            "  {:>2} × {:<28} {}",
            l.qty,
            a.map(|a| a.name).unwrap_or(l.asset_id),
            a.map(|a| format!("{:.1}×{:.1}×{:.1} m", a.footprint.0, a.footprint.1, a.footprint.2))
                .unwrap_or_default()
        );
    }
    println!("  — {total} items, {} lines", bom.lines.len());
    Ok(())
}

fn apply_spoken(op: &SpokenOp, city: Option<&mut City>) -> String {
    match op {
        SpokenOp::Help => help_text(),
        SpokenOp::ListFacilities => format!(
            "Facilities: {}",
            FACILITY_BOMS.iter().map(|f| f.id).collect::<Vec<_>>().join(", ")
        ),
        SpokenOp::Install { asset_id, target } => {
            let name = asset_by_id(asset_id).map(|a| a.name).unwrap_or(asset_id);
            match city {
                Some(city) => {
                    let cell = city.preferred_stock_cell();
                    city.stocked.push(StockedItem { asset_id: asset_id.clone(), cell: cell.clone() });
                    format!("Installed {name} → stocked on cell {} ({target:?})", cell.as_deref().unwrap_or("none — empty city"))
                }
                None => format!("Would install {name} ({target:?}) — pass --city FILE to apply"),
            }
        }
        SpokenOp::Bom { facility_id, stock } => {
            let bom = get_facility_bom(facility_id).expect("parser only yields known facilities");
            let mut out = format!("BOM // {} — {} lines", bom.label, bom.lines.len());
            if *stock {
                match city {
                    Some(city) => {
                        let cell = city.preferred_stock_cell();
                        let mut n = 0;
                        for l in bom.lines {
                            for _ in 0..l.qty {
                                city.stocked.push(StockedItem {
                                    asset_id: l.asset_id.to_string(),
                                    cell: cell.clone(),
                                });
                                n += 1;
                            }
                        }
                        out += &format!("\nStocked {n} items on cell {}", cell.as_deref().unwrap_or("none — empty city"));
                    }
                    None => out += "\nNot stocked — pass --city FILE to apply",
                }
            }
            out
        }
        SpokenOp::Unknown { raw } => format!("Unknown command: \"{raw}\" — say \"help\""),
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Bom { facility, json } => print_bom(&facility, json),
        Cmd::Assets { json } => {
            if json {
                let v: Vec<_> = INDUSTRIAL_ASSETS
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "id": a.id, "name": a.name, "category": a.category,
                            "footprint_m": a.footprint, "usdcraft": asset_hf_url(a),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&v).unwrap());
            } else {
                println!("ANYKIT CATALOG // USDCraft-10K industrial (Apache-2.0)");
                for a in INDUSTRIAL_ASSETS {
                    println!("  {:<18} {:<28} [{}]", a.id, a.name, a.category);
                }
                println!("  — {} assets", INDUSTRIAL_ASSETS.len());
            }
            Ok(())
        }
        Cmd::Layout { size, length, stories } => {
            let cell = layout_for_cell(size, length);
            let tower = layout_for_tower(size, stories);
            println!("HEX-TUBE CELL // {size} ({} m flat-to-flat, length class {})", cell.flat_to_flat_m, length.as_str());
            println!("  corridor FF   {:.2} m", cell.corridor_ff);
            println!("  wedge depth   {:.2} m", cell.wedge_depth_m);
            println!("  wall          {:.2} m", cell.wall_m);
            println!("  length        {:.1} m", cell.length_m);
            println!("  channels      {}", cell.channel_ids().iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", "));
            println!("VERTEX TOWER // vertical, larger Ø (×1.35), {stories} stories");
            println!("  tower FF      {:.2} m", tower.section.flat_to_flat_m);
            println!("  corridor FF   {:.2} m", tower.section.corridor_ff);
            println!("  height        {:.1} m", tower.height_m);
            Ok(())
        }
        Cmd::Channels => {
            println!("UTILITY CHANNELS // six radial wedges of the hex tube");
            for c in CHANNELS.iter().chain(EXTRA_CHANNELS.iter()) {
                let kind = if CHANNELS.contains(c) { "wedge" } else { "extra" };
                println!(
                    "  {:<12} wedge {} [{}] {} — {}",
                    c.id, c.wedge, kind, c.label, c.description
                );
                let _ = channel_spec(c.id);
            }
            let _ = Channel::ALL;
            Ok(())
        }
        Cmd::City { cmd } => match cmd {
            CityCmd::New { out } => save_city(&City::new(), &out).map(|_| println!("wrote empty city → {}", out.display())),
            CityCmd::Preset { name, out } => {
                let city = City::preset(&name).ok_or_else(|| format!("unknown preset '{name}' (recon|carrier|sandbox)"))?;
                save_city(&city, &out).map(|_| println!("wrote {name} preset ({} hexes) → {}", city.len(), out.display()))
            }
            CityCmd::Place { file, q, r, module, size } => {
                let mut city = load_city(&file)?;
                city.place(Axial::new(q, r), module, size);
                save_city(&city, &file).map(|_| println!("placed {module} ({size}) at ({q},{r}) — {} hexes", city.len()))
            }
            CityCmd::Erase { file, q, r } => {
                let mut city = load_city(&file)?;
                let gone = city.erase(Axial::new(q, r));
                save_city(&city, &file).map(|_| println!("{} ({q},{r}) — {} hexes", if gone { "erased" } else { "nothing at" }, city.len()))
            }
            CityCmd::Towers { file } => {
                let city = load_city(&file)?;
                let towers = city.towers();
                println!("VERTEX TOWERS // {} (degree ≥ 2)", towers.len());
                for t in &towers {
                    println!(
                        "  ({:>7.1},{:>7.1}) degree {} — tower FF {:.1} m, thruster {}, channels {}",
                        t.x,
                        t.z,
                        t.degree,
                        t.tower_ff,
                        if t.has_thruster { "yes" } else { "no" },
                        t.channels.len()
                    );
                }
                Ok(())
            }
            CityCmd::Show { file } => {
                let city = load_city(&file)?;
                println!("{}", city.to_json());
                Ok(())
            }
            CityCmd::Telemetry { file, json } => {
                let city = load_city(&file)?;
                let t = city.telemetry();
                if json {
                    println!("{}", serde_json::to_string_pretty(&t).unwrap());
                } else {
                    println!("TELEMETRY // {} hexes", t.total);
                    println!("  deck area     {:.0} m²", t.deck_area_m2);
                    println!("  interlocks    {} nodes", t.interlock_nodes);
                    println!("  towers        {}", t.towers);
                    println!("  thrusters     {}", t.thrusters);
                    println!("  displacement  {:.0} t", t.displacement_t);
                    println!("  bays          {} ({} FH / {} DD / {} SH)", t.bays, t.flight, t.drydock, t.subhub);
                    println!("  stability     {:.1}%", t.stability_pct);
                    println!("  interconnects {} shared faces", t.edges);
                    println!("  stocked       {} items", t.stocked);
                }
                Ok(())
            }
            CityCmd::Export { file, format } => {
                let city = load_city(&file)?;
                match format.as_str() {
                    "xthread" | "x" => println!("{}", hexlab_core::export::x_thread(&city)),
                    "spec" => println!("{}", hexlab_core::export::spec_sheet(&city)),
                    other => return Err(format!("unknown export format '{other}' (xthread|spec)")),
                }
                Ok(())
            }
        },
        Cmd::Spoken { text, city } => {
            let op = parse_spoken(&text);
            match city {
                Some(path) => {
                    let mut city = load_city(&path)?;
                    let out = apply_spoken(&op, Some(&mut city));
                    save_city(&city, &path)?;
                    println!("{out}");
                }
                None => println!("{}", apply_spoken(&op, None)),
            }
            Ok(())
        }
        Cmd::Kit { cmd } => match cmd {
            KitCmd::Plan { facility, size, out } => {
                let bom = get_facility_bom(&facility)
                    .ok_or_else(|| format!("unknown facility '{facility}'"))?;
                let plan = facility_bom_to_anyscene_plan(bom, size);
                let text = serde_json::to_string_pretty(&plan).unwrap();
                match out {
                    Some(path) => std::fs::write(&path, text)
                        .map(|_| println!("wrote AnyScene plan for {facility} → {}", path.display()))
                        .map_err(|e| e.to_string()),
                    None => {
                        println!("{text}");
                        Ok(())
                    }
                }
            }
            KitCmd::Job { kind, prompt, style, items, project } => {
                let kind = match kind.to_ascii_lowercase().as_str() {
                    "anykit" | "kit" => KitKind::AnyKit,
                    "anymachine" | "machine" => KitKind::AnyMachine,
                    other => return Err(format!("unknown kit kind '{other}' (anykit|anymachine)")),
                };
                let job = KitJob::new(kind, &prompt, items, &style);
                let path = job
                    .write_manifest(&project)
                    .map_err(|e| format!("cannot write job manifest: {e}"))?;
                println!("queued {:?} job {} → {}", job.kind, job.id, path.display());
                println!("status: queued — hex-module jobs generate locally: hexlab kit run {} (Blender); prop kits land assets in {}", job.id, job.out_dir);
                Ok(())
            }
            KitCmd::PlanModule { module, size, out } => {
                let plan = module_to_anyscene_plan(module, size);
                let text = serde_json::to_string_pretty(&plan).unwrap();
                match out {
                    Some(path) => std::fs::write(&path, text)
                        .map(|_| println!("wrote AnyScene module plan for {} → {}", module.as_str(), path.display()))
                        .map_err(|e| e.to_string()),
                    None => {
                        println!("{text}");
                        Ok(())
                    }
                }
            }
            KitCmd::Module { module, size, project } => {
                let job = KitJob::module_job(module, size);
                let path = job
                    .write_manifest(&project)
                    .map_err(|e| format!("cannot write job manifest: {e}"))?;
                println!("queued AnyMachine hex-module job {} → {}", job.id, path.display());
                println!("generate it on this machine: hexlab kit run {} --project {}", job.id, project.display());
                Ok(())
            }
            KitCmd::Run { job_id, project, blender, script } => {
                match run_job(&project, &job_id, blender.as_deref(), &script)? {
                    RunOutcome::Ready { artifact } => {
                        println!("READY {job_id} → {}", artifact.display());
                        println!("registered in the HEX module library ({})", Library::path(&project).display());
                    }
                    RunOutcome::BlenderNotFound => {
                        println!("Blender not found on this machine — job '{job_id}' stays QUEUED.");
                        println!("HexLab generates locally: install Blender (blender.org) or pass --blender <path>.");
                    }
                    RunOutcome::NotGeneratable(why) => println!("not generatable: {why}"),
                }
                Ok(())
            }
        },
        Cmd::Library { project, json } => {
            let lib = Library::load(&project)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&lib).unwrap());
            } else {
                println!("HEX MODULE LIBRARY // {} entries ({})", lib.entries.len(), Library::path(&project).display());
                for e in &lib.entries {
                    let art = e.artifact.as_deref().unwrap_or("—");
                    println!("  {:28} {:?}/{:?}  {}", e.id, e.kind, e.source, art);
                }
            }
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("hexlab: {e}");
            ExitCode::FAILURE
        }
    }
}
