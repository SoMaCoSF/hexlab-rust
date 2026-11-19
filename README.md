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
