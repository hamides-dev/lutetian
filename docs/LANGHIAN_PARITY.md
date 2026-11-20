# Langhian Parity & Lutetian Architecture Blueprint

> **Phase 0 Discovery Record** — Extracted directly from `E:\langhian` prior to writing any Lutetian application code.

---

## 1. UI Framework & Runtime Footprint Decision

| Attribute | `E:\langhian` (Reference) | `E:\lutetian` (Target Parity) |
| :--- | :--- | :--- |
| **GUI Framework** | `iced = "0.14"` (`features = ["tokio", "image", "advanced", "canvas", "svg", "tiny-skia"]`) | Exact match: `iced = "0.14"` with identical features |
| **WebView / Chromium** | **Zero WebView** (100% native Rust `wgpu` + `tiny-skia` software fallback) | **Zero WebView** (strictly pure Rust) |
| **Rust Edition** | `edition = "2024"` (verified `rustc 1.99.1` / `cargo 1.99.1`) | `edition = "2024"` |
| **Release Profile** | `opt-level = "s"`, `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"` | Exact match (minimal binary size, fast cold start, low idle RAM) |
| **Windows Subsystem** | `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` + `winresource` icon embed (`build.rs`) | Exact match |
| **Single-Instance & Window Chrome** | Frameless transparent window (`decorations: false`, `transparent: true`, `DoNotRound`), 12px outer radius, 1px subtle border, 8-edge resize overlay | Exact match (`1060x720` default or `1220x780` with `min_size = 720x560`) |

**Conclusion:** `E:\langhian` already uses a pure-Rust non-WebView GPU-accelerated UI stack (`iced 0.14`). `Lutetian` reuses `iced 0.14` directly and replicates every color token, spacing rule, font family, icon system, and container style natively.

---

## 2. Extracted Visual Design System ("Velvet Plum")

### 2.1 Color Tokens (`ThemePalette`)

#### Midnight Dark (`UiThemeMode::Dark`)
| Token | RGB / RGBA | Hex / Approx | Usage |
| :--- | :--- | :--- | :--- |
| `bg_app` | `rgb(0.059, 0.055, 0.078)` | `#0F0E14` | Main workspace background |
| `bg_sidebar` | `rgb(0.043, 0.039, 0.059)` | `#0B0A0F` | Titlebar & left navigation sidebar |
| `bg_surface` | `rgb(0.086, 0.078, 0.114)` | `#16141D` | Cards, panels, preview frames |
| `bg_surface_hover` | `rgb(0.118, 0.106, 0.153)` | `#1E1B27` | Card / nav item hover state |
| `bg_elevated` | `rgb(0.110, 0.098, 0.141)` | `#1C1924` | Active nav item, pill buttons, keycaps, code blocks |
| `bg_input` | `rgb(0.051, 0.047, 0.071)` | `#0D0C12` | Text inputs, pick lists, metric fields |
| `bg_modal` | `rgba(0.075, 0.067, 0.102, 0.98)` | `#13111AFA` | Modal & floating dialog surfaces |
| `border_subtle` | `rgba(0.84, 0.78, 0.90, 0.095)` | — | 1px card, input, and shell borders |
| `border_focus` | `rgba(0.651, 0.545, 0.745, 0.65)` | — | Focused/active borders |
| `divider` | `rgba(0.84, 0.78, 0.90, 0.065)` | — | 1px hairline dividers |
| `text_primary` | `rgb(0.945, 0.937, 0.961)` | `#F1EFF5` | Primary headings & values |
| `text_secondary` | `rgb(0.678, 0.651, 0.722)` | `#ADA6B8` | Subtitles, descriptions, inactive icons |
| `text_muted` | `rgb(0.478, 0.447, 0.529)` | `#7A7287` | Placeholders, separators, secondary metadata |
| `accent` | `rgb(0.467, 0.388, 0.549)` | `#77638C` | Velvet Plum primary action & toggle active |
| `accent_hover` | `rgb(0.545, 0.463, 0.631)` | `#8B76A1` | Primary button hover |
| `accent_soft` | `rgba(0.467, 0.388, 0.549, 0.22)` | — | Selected state tint, soft badges |
| `accent_text` | `rgb(0.788, 0.722, 0.831)` | `#C9B8D4` | Soft Lavender active labels & icons |
| `success` / `success_soft` | `rgb(0.243, 0.804, 0.588)` / `alpha 0.15` | `#3ECD96` | `COMPLIANT` / approved / connected badges |
| `warning` | `rgb(0.965, 0.714, 0.282)` | `#F6B648` | `PENDING` / in-progress / needs review |
| `danger` / `danger_soft` | `rgb(0.949, 0.435, 0.467)` / `alpha 0.15` | `#F26F77` | `VIOLATED` / close hover / errors |
| `scrollbar_thumb` | `rgba(0.82, 0.76, 0.88, 0.18)` | — | Slim 4px scrollbar thumb (`0.34` on hover) |

#### Daylight Light (`UiThemeMode::Light`)
| Token | RGB / RGBA | Hex / Approx |
| :--- | :--- | :--- |
| `bg_app` | `rgb(0.969, 0.961, 0.980)` | `#F7F5FA` |
| `bg_sidebar` | `rgb(0.937, 0.922, 0.957)` | `#EFEBF4` |
| `bg_surface` | `rgb(1.0, 1.0, 1.0)` | `#FFFFFF` |
| `bg_surface_hover` | `rgb(0.953, 0.937, 0.973)` | `#F3EFF8` |
| `bg_elevated` | `rgb(0.937, 0.918, 0.961)` | `#EFEAF5` |
| `bg_input` | `rgb(0.980, 0.976, 0.988)` | `#FAF9FC` |
| `border_subtle` | `rgba(0.220, 0.180, 0.270, 0.115)` | — |
| `border_focus` | `rgba(0.408, 0.361, 0.455, 0.58)` | — |
| `divider` | `rgba(0.220, 0.180, 0.270, 0.08)` | — |
| `text_primary` | `rgb(0.114, 0.098, 0.145)` | `#1D1925` |
| `text_secondary` | `rgb(0.369, 0.333, 0.424)` | `#5E556C` |
| `text_muted` | `rgb(0.525, 0.490, 0.580)` | `#867D94` |
| `accent` | `rgb(0.408, 0.361, 0.455)` | `#685C74` |
| `accent_hover` | `rgb(0.337, 0.294, 0.384)` | `#564B62` |
| `accent_soft` | `rgba(0.408, 0.361, 0.455, 0.13)` | — |
| `accent_text` | `rgb(0.353, 0.302, 0.408)` | `#5A4D68` |
| `success` / `warning` / `danger` | `#11976A` / `#C8780E` / `#CF3038` | Semantic parity |

### 2.2 Typography, Iconography & Spacing Rules
- **Bundled Fonts (`include_bytes!`):** `Inter` (`Regular`, `Medium`, `SemiBold`, `Bold`) for LTR/UI; `Vazirmatn` (`Regular`, `Medium`, `Bold`) for RTL fallback; `Font::MONOSPACE` for metric IDs, JSON snippets, and coordinates. (`Weight::Semibold` normalizes to `Weight::Medium` to match bundled TTF weights.)
- **Vector Icons:** Google **Material Symbols Rounded** (`24px` grid, `viewBox="0 -960 960 960"`) embedded as `&[u8]` slices in `ui/icons.rs` and tinted via `iced::widget::svg::Style`. Zero Unicode emojis in UI controls/headers.
- **App Icon:** Transparent squircle icon (`assets/icons/lutetian.png` + `.ico`) in Velvet Plum (`#685C74`) and Soft Lavender (`#C9B8D4`) matching Langhian's silhouette and visual finish.
- **Layout Geometry:**
  - **Custom Titlebar:** Height `38px`, padding `[2, 12]`, `bg_sidebar`, top-left/top-right radius `11px` (0 when maximized). Contains `app_logo(22.0)`, `Lutetian · Environment Architecture Studio`, live Agent Bridge connection pill, and right-aligned `28x28` square icon controls (sidebar toggle, minimize, maximize/restore, close with `danger` hover).
  - **Sidebar:** `220px` expanded (`padding([16, 14])`), `64px` collapsed (`padding([16, 10])`), `bg_sidebar`, bottom-left corner radius `11px`, `5px` nav button spacing, bottom update banner slot + hairline divider + `Settings` pinned at bottom.
  - **Workspace:** Separated by `1px` vertical `border_subtle` divider, `padding([22, 26])`, `20pt` bold page header + `12.5pt` subtitle, `10px` radius cards (`card_style`), `8px` radius buttons/inputs, `999px` radius status pills (`badge_style`), `sleek_row` (`[14, 18]` padding) separated by `hairline_divider`, slim `4px` scrollbar (`sleek_scrollable`).

---

## 3. Installer, Auto-Updater & CI/CD Parity

1. **Versioning & Manifest (`release-please` + `version.json`):**
   - `release-please-config.json` (`release-type: "simple"`, `bump-minor-pre-major: true`, updates `Cargo.toml`, `packaging/macos/Info.plist`, `packaging/windows/lutetian.iss`, `version.json`).
   - `.release-please-manifest.json` starting at `"0.1.0"`.
   - `version.json` tracks `version`, `tag`, `published_at`, `release_url`, `notes`, and platform `assets` (`windows_x86_64_installer`, `windows_x86_64_portable`, `macos_arm64_dmg`, `macos_x86_64_dmg`, `linux_x86_64_deb`, `linux_x86_64_tar`).
2. **In-App Auto-Updater (`lutetian-core/src/updater.rs` + `platform/updater_exec.rs`):**
   - TCP pre-flight connectivity check (`has_internet_connectivity` against `api.github.com` / `raw.githubusercontent.com`).
   - Checks `https://api.github.com/repos/hamides-dev/lutetian/releases/latest` (or `/releases?per_page=10` when `include_prereleases` is enabled, supporting `nightly` commit SHA comparison via `LUTETIAN_BUILD_SHA`), falling back to `https://raw.githubusercontent.com/hamides-dev/lutetian/main/version.json`.
   - Streams installer download with `DownloadProgress` percentage bar + optional SHA-256 verification.
   - Windows installer launch: spawns Inno Setup `.exe` with `["/SILENT", "/CLOSEAPPLICATIONS", "/RESTARTAPPLICATIONS", "/RELAUNCH"]` and returns `Ok(true)` so `lutetian-app` exits cleanly while the installer updates and relaunches.
3. **Windows Packaging (`packaging/windows/lutetian.iss`):**
   - Inno Setup 6 (`lzma2/ultra64`, `SolidCompression=yes`, `WizardStyle=modern`, `PrivilegesRequired=lowest`, `CloseApplications=yes`, `RestartApplications=yes`, `AppMutex=LutetianStudioSingleInstanceMutex`).
   - Installs `lutetian.exe`, `lutetian-mcp.exe`, and `antigravity-plugin\` under `{localappdata}\Programs\Lutetian`.
   - `[Run]` entry handles `/RELAUNCH` for seamless auto-update restart.
4. **GitHub Actions (`.github/workflows/ci.yml` & `release.yml`):**
   - `ci.yml`: Matrix over `[windows-latest, macos-latest, ubuntu-latest]` running `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (under `xvfb-run` on Linux).
   - `release.yml`: `release-please-action@v4`, `prepare` metadata job (stable tags, `v*`, `workflow_dispatch`, or `nightly` pre-release on `main` push), parallel release builds for Windows (`x86_64` portable `.zip` + Inno Setup `.exe`), macOS (`arm64` & `x86_64` `.dmg` + `.tar.gz`), and Linux (`x86_64` `.tar.gz` + `.deb`), generating `checksums.txt` and publishing via `softprops/action-gh-release@v2`.

---

## 4. Lutetian Cargo Workspace & System Architecture

```text
e:\lutetian\
├── Cargo.toml                         # Workspace root (edition 2024, release profile matching Langhian)
├── version.json / CHANGELOG.md / LICENSE / README.md / THIRD_PARTY_LICENSES.md
├── release-please-config.json / .release-please-manifest.json
├── assets/
│   ├── fonts/                         # Inter (4 weights) + Vazirmatn (3 weights)
│   └── icons/                         # lutetian.png, lutetian.ico, ui/*.svg (Material Symbols Rounded)
├── crates/
│   ├── lutetian-core/                 # Domain models, Stage State Machine, Master Blueprint JSON,
│   │                                  # Deterministic Scale Validator, Telemetry/Compliance models,
│   │                                  # Engine Sync Export, Storage, UpdaterClient
│   ├── lutetian-bridge/               # Embedded Axum HTTP JSON-RPC + WebSocket server inside lutetian-app
│   │                                  # for bidirectional real-time sync with MCP agents
│   ├── lutetian-mcp/                  # Standalone stdio MCP server binary bridging Google Antigravity
│   │                                  # to lutetian-bridge (with active port discovery & offline fallback)
│   └── lutetian-app/                  # Pure-Rust iced 0.14 GUI: Velvet Plum shell, 6-Stage Views,
│                                      # Interactive Multi-Layer 2D CAD Blueprint Canvas,
│                                      # Always-Accessible Live Agent Mission Control & Compliance Matrix
├── antigravity-plugin/                # Ready-to-install Google Antigravity plugin pack
│   ├── README.md / mcp_config.json / install.ps1 / install.sh
│   └── skills/
│       ├── lutetian-live-protocol/SKILL.md
│       ├── lutetian-stage1-analysis/SKILL.md
│       ├── lutetian-stage2-multiview/SKILL.md
│       ├── lutetian-stage3-blueprint/SKILL.md
│       └── lutetian-stage4-assets/SKILL.md
├── packaging/                         # Windows (.iss), macOS (Info.plist), Linux (.desktop)
└── .github/workflows/                 # ci.yml + release.yml
```

### Key Lutetian Functional Contracts
1. **Stage-Gated Pipeline (Stages 1–6):** Enforced in `lutetian-core` state machine. Cannot advance or lock downstream artifacts without upstream approval (`ProjectBible` scale anchors → `MultiView` 5-angle approvals → `MasterBlueprint` JSON → `AssetCatalog` deduplicated BOM + isolated slots → `Deterministic Scale Validation` zero-error gate → `Engine Sync Export`).
2. **Interactive Multi-Layer 2D CAD Blueprint Canvas (`iced::widget::canvas`):**
   - Pan (drag) & Zoom (wheel / toolbar buttons / Fit-to-View) on a true metric coordinate grid (`1 unit = 1.0 meter`).
   - Dynamic metric rulers (top & left axes in meters) + scale bar.
   - 5 independently toggleable layers:
     1. **Footprint & Metrics Layer** (wall footprints, exact X/Y/Z dimensions in meters, door openings)
     2. **Elevation & Height Layer** (terrain contour bands, Z-elevation tags, height shading)
     3. **Navigation & Scale Layer** (walkable paths, minimum clearance widths, interactive draggable `1.8m` human silhouette & `4.5m × 1.9m` vehicle silhouette for instant human-eye proportion verification)
     4. **Zoning & Surface Layer** (road, grass, dirt, water, residential, commercial, plaza surfaces)
     5. **Asset ID Map Layer** (clickable element callouts e.g. `HOUSE_A_01`, `TREE_PINE_01` linked directly to the Asset Catalog)
   - Clicking a violated rule in the Compliance Matrix selects and focuses/highlights the offending entity directly on the 2D canvas.
3. **Live Agent Mission Control & Compliance Dashboard:**
   - Persistent status strip in the header + full dedicated Mission Control tab.
   - **Live Activity Stream:** Active agent name, heartbeat age, active stage & skill, current tool, live status message, and progress bar.
   - **3-State Execution Pipeline Board:** `Completed` (timestamps & linked artifacts), `In Progress` (progress % & sub-steps), `Upcoming` (planned queue).
   - **Compliance & Rules Matrix:** Every stage gate, reporting requirement, and deterministic math rule with `COMPLIANT`, `PENDING`, `VIOLATED` badges and click-to-inspect/highlight entity behavior.
