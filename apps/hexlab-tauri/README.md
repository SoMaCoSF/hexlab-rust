# HexLab Tauri

Requires Rust ≥1.77 + tauri-cli.

```bash
cd apps/hexlab-tauri
npm create tauri-app@latest . -- --template vanilla-ts
# path-depend hexlab-core in src-tauri/Cargo.toml
# invoke layout/bom/city from the web UI
```

---

## As built (2026-11-18)

The scaffold above is now real — no `npm create` step needed:

```
apps/hexlab-tauri/
  src-tauri/
    Cargo.toml          tauri 2, path-depends hexlab-core
    build.rs
    tauri.conf.json     com.somacosf.hexlab, frontendDist ../ui
    src/main.rs         invoke commands over hexlab-core:
                        city_preset, city_place, city_towers,
                        facility_bom, spoken, kit_plan, export_brief
  ui/
    index.html          first frontend: fullscreen SVG platform,
                        floating draggable panels (palette, kinematics,
                        telemetry, export), spoken line. Uses Tauri
                        invoke when packaged; runs the same model in
                        plain JS in a browser for testing.
```

Tauri ≠ Electron: Rust process + OS webview (WebKitGTK on
Omarchy/Linux, WebView2 on Windows), no bundled Chromium.

Build on the target machine (needs the OS webview dev packages —
`libwebkit2gtk-4.1-dev` etc. on Arch/Debian; nothing extra on Windows):

```bash
cargo install tauri-cli --version "^2"
cd apps/hexlab-tauri
cargo tauri build   # or: cargo tauri dev
```

The React Three Fiber Build Bay from `SoMaCoSF/hexlab-platform` can
replace `ui/` later without touching the Rust commands — they are the
contract. On the build VM (no webview libraries) this crate is
scaffold-verified only; `hexlab-core` / CLI / TUI carry the test proof.
