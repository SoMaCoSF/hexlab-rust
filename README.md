# HexLab local (Rust)

City CAD core + agent CLI. Tauri shell next.

```bash
git clone https://github.com/SoMaCoSF/hexlab-rust.git && cd hexlab-rust
cargo run -p hexlab-cli -- bom woodshop
cargo run -p hexlab-cli -- city preset recon -o city.json
cargo run -p hexlab-cli -- city towers city.json
cargo run -p hexlab-cli -- layout industrial --stories 6
```

## Commands

| Command | What |
|---------|------|
| `hexlab bom <woodshop\|machineshop\|fabrication> [--json]` | Facility BOM |
| `hexlab layout [size] [--length M] [--stories N]` | Tube + vertex tower layout |
| `hexlab channels` | Utility channel list |
| `hexlab city new -o FILE` | Empty city JSON |
| `hexlab city preset recon -o FILE` | 7-hex recon ring |
| `hexlab city place FILE q r MODULE [--size]` | Place module |
| `hexlab city erase FILE q r` | Remove cell |
| `hexlab city towers FILE` | Vertex towers (degree ≥ 2) |
| `hexlab city show FILE` | Dump JSON |

Sizes: `scout` `standard` `industrial` `capital`  
Modules: `core` `flight` `drydock` `subhub` `refinery` `desal` `lab` `power`

## Layout

```
crates/hexlab-core   topology, channels, BOMs, tube/tower
crates/hexlab-cli    `hexlab` binary
apps/hexlab-tauri    Tauri notes (Rust ≥1.77)
```

---

## Extended (2026-11-18) — the build Grok ran out of time on

Grok's scaffold commit carried only the `Cargo.toml`s and this README —
no source. This tree now contains the full local build, ported from the
live web build (`SoMaCoSF/hexlab-platform` → hexlab.somacosf.com) and
the Gemini marine configurator, with the pieces Grok lost rebuilt.

```
crates/hexlab-core    axial topology · hex-tube kit (10 channels,
                      cell/tower sections) · city + presets · vertex
                      towers · telemetry · anyKIT catalog (USDCraft-10K
                      + 7 facility BOMs) · spoken grammar · kit jobs +
                      AnyScene plans · export briefs
crates/hexlab-cli     `hexlab` binary (below)
crates/hexlab-tui     `hexlab-tui` binary — fullscreen TUI, floating
                      panels: palette / platform / telemetry / log
apps/hexlab-tauri     Tauri 2 shell (Rust + OS webview, not Electron):
                      invoke commands over hexlab-core + first UI
                      (fullscreen platform, floating draggable panels)
docs/ARCHITECTURE.md  SOM-DOC-10857 — model, kit, jobs, provenance
install.ps1           Windows one-shot: clone → build → smoke → run
install.sh            Omarchy/Linux one-shot (same shape)
```

### More commands

| Command | What |
|---------|------|
| `hexlab assets [--json]` | anyKIT catalog (31 USDCraft-10K industrial assets) |
| `hexlab city telemetry FILE [--json]` | area, interlocks, towers, thrusters, displacement, stability |
| `hexlab city export FILE [--format xthread\|spec]` | @PalmerLuckey X reply / engineering spec from live telemetry |
| `hexlab spoken "<text>" [--city FILE]` | spoken-world grammar: "install a band saw", "create a bom for a woodshop and stock it" |
| `hexlab kit plan <facility> [--size] [-o FILE]` | AnyScene plan JSON for a facility BOM |
| `hexlab kit plan-module <module> [--size]` | AnyScene plan for one HEX module (shell + channel-wedge BOM) |
| `hexlab kit job <anykit\|anymachine> "<prompt>" [--item …]` | local job folder + manifest under `kits/<id>/` |
| `hexlab kit module <module> [--size]` | queue a HEX module creation job (AnyMachine) |
| `hexlab kit run <job-id> [--blender PATH]` | generate on the LOCAL Blender → `module.glb` + library entry |
| `hexlab library [--json]` | the HEX module library: 8 stock modules + generated creations |
| `hexlab-tui [--city city.json]` | fullscreen TUI (`--snapshot` renders one frame headlessly) |

### One-shot install

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/SoMaCoSF/hexlab-rust/main/install.ps1 | iex
```

Omarchy / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/SoMaCoSF/hexlab-rust/main/install.sh | bash
```

### What runs fully local

Hex city, Build Bay model, section/cell/tower/matrix math, BOMs,
spoken grammar (OS speech APIs feed it), telemetry, exports — yes,
100% local. AnyKit/AnyMachine *generation* runs outside the app
(Blender locally; fal/Tripo/Meshy need network + keys) — the app owns
the job folders and stocks the results. See `docs/ARCHITECTURE.md`.
