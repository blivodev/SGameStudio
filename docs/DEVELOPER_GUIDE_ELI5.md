# SGameStudio — Developer Guide (Plain English)

This guide is for people who want to change or extend the SGameStudio code. It assumes
you know a bit of code, but explains the layout, conventions, and how to test your
changes so you don't guess.

---

## 1. Big Picture

SGameStudio has **two halves** that talk over HTTP/WebSocket:

- **Backend (Rust)** in `src-tauri/` — the "brain". It holds the scene, creates shapes
  and 3D geometry, makes sound, stores settings, and exports files. It can run as a
  desktop GUI or as a headless server.
- **Frontend (React + TypeScript)** in `src/` — the "face". It is the webview UI
  (2D/3D/audio/assembler panels). It talks to the backend over `http://127.0.0.1:8080`.

The rule that keeps everything consistent: **the frontend never changes scene data
directly.** Every action is a *command* sent to the backend's **Command Bus**. The same
command can be triggered by a button, by the AI, or by an external automation tool.

---

## 2. Where the Important Files Live

### Backend (`src-tauri/src/`)
| File / folder | Purpose |
| :--- | :--- |
| `main.rs` | Entry point. Parses `--server`, `--mcp`, and GUI mode. In GUI mode it starts the embedded backend automatically. |
| `lib.rs` | Library root; wires modules; runs self-diagnostics (`sgame-studio test`). |
| `core/` | Command Bus, commands, state store, project zip save/load. |
| `core/commands.rs` | The `StudioCommand` enum — all commands live here. |
| `core/command_bus.rs` | Dispatches commands to the state store, handles undo/redo. |
| `core/state_store.rs` | The actual scene state (entities, meta). |
| `server/routes.rs` | All REST HTTP endpoints (`/api/v1/...`). |
| `server/mod.rs` | Builds the Axum app; adds CORS and the default port. |
| `mcp/` | Model Context Protocol server (stdio + SSE) + tools + resources. |
| `export/` | 3D/2D/audio exporters (OBJ, STL, PLY, GLB, WAV…). |
| `settings.rs` | Provider API-key storage, masking, deobfuscation. |
| `ai_gateway/` | Multi-provider AI router + providers (OpenRouter, Gemini, …). |
| `audio_dsp/` | sfxr sound synthesis + WAV encoder. |
| `mesh_kernel/` | 3D half-edge B-Rep kernel (bevel/loop_cut are documented stubs). |

### Frontend (`src/`)
| File / folder | Purpose |
| :--- | :--- |
| `App.tsx` | Main layout, tab switching, connection dot. |
| `services/` | API clients: `commandBridge.ts`, `settings.ts`, `ai.ts`, `export.ts`, `eventSocket.ts`. |
| `store/studioStore.ts` | Zustand global state (entities, chat, theme, connection). |
| `lib/anim.ts` | Pure keyframe interpolation helpers (testable). |
| `modules/` | The four panels: `studio2d/`, `studio3d/`, `audio/`, `assembler/`. |
| `components/settings/` | The Settings dialog (edits API keys). |
| `components/chat/` | The AI assistant chat panel. |
| `__tests__/` | Vitest tests (mirror the services/lib). |

---

## 3. How to Add a New REST Endpoint

Say you want a backend endpoint `GET /api/v1/hello`.

1. In `src-tauri/src/server/routes.rs`, add to the router:
   ```rust
   .route("/api/v1/hello", get(hello))
   ```
2. Write the handler near the others:
   ```rust
   async fn hello() -> Json<serde_json::Value> {
       Json(serde_json::json!({ "message": "hi" }))
   }
   ```
3. Add a test in the `#[cfg(test)] mod tests` at the bottom of `routes.rs`.

To call it from the frontend, add a function in a `src/services/*.ts` file.

---

## 4. How to Add a New Frontend Panel/Feature

1. Usually you add a button in one of the `modules/*.tsx` files.
2. To call the backend, use `dispatchCommand(...)` or a service function.
3. To show live data, read from `useStudioStore()`.

Example — export a mesh (already implemented in `modules/studio3d/Studio3D.tsx`):

```ts
import { exportMesh, downloadAssetResult } from '../../services/export';
const res = await exportMesh('mesh3d_1', 'obj');
downloadAssetResult(res); // saves the file
```

---

## 5. Testing (do this before you finish)

Both halves have tests. **Run them after any change.**

Backend (from `src-tauri/`):

```powershell
cargo test
cargo check   # compile only, faster
```

Frontend (from repo root):

```powershell
npm run build   # TypeScript typecheck + Vite build
npm test        # Vitest unit tests
```

Self-diagnostics (headless end-to-end sanity check):

```powershell
cargo run --no-default-features -- test
```

> Windows note: if `cargo test` hits a rare "file-in-use" error (local antivirus),
> re-run with fewer jobs: `cargo test -j 2`.

---

## 6. The Settings / API-Key Flow (important details)

- Keys live in `%APPDATA%\sgamestudio\providers.json`, **obfuscated** (XOR) at rest.
- `GET /api/v1/settings` returns providers with a **masked** key (`sk-a••••mnop`) — never
  the full key.
- `POST /api/v1/settings` expects camelCase: `{ "provider": "openrouter", "apiKey": "…" }`.
  (The backend `SettingsBody` struct uses `#[serde(rename_all = "camelCase")]` — do not
  change this or saving breaks.)
- `POST /api/v1/ai/chat` uses `ai_gateway::run_task(...)`. It returns `503` with a
  friendly message when no key is configured.

If you add a new provider:
1. Add its name to `PROVIDERS` in `src-tauri/src/settings.rs`.
2. Add a `Provider` case in `src-tauri/src/ai_gateway/providers.rs` (and either a local
   or keyed entry in `provider_usable` in `ai_gateway/mod.rs`).
3. Add the env var name and, optionally, a Settings-dialog input in
   `src/components/settings/SettingsDialog.tsx`.

---

## 7. The Desktop "Connected" Indicator

The header dot is green when the backend health check succeeds. In GUI mode the backend
is started in a background thread (see `main.rs`) on `127.0.0.1:8080`. The frontend
checks `GET /api/v1/health`. If you change the port, either set `SGAME_PORT` on the
backend or `VITE_SGAME_API_BASE` on the frontend — they must match.

---

## 8. Publishing a New Windows `.exe`

```powershell
npm run build                 # builds the frontend into dist/
npx tauri build               # compiles Rust + bundles the NSIS installer
```

Output:
- App binary: `src-tauri/target/release/sgame-studio.exe`
- Installer: `src-tauri/target/release/bundle/nsis/SGameStudio_0.1.0_x64-setup.exe`

Before publishing, remember the workflow: **test → commit → push → rebuild the exe**.

---

## 9. Conventions Checklist

- Commands go through the Command Bus — never mutate scene data from the UI.
- Add a unit test for every new backend function and every non-trivial frontend helper.
- Keep API keys masked/obfuscated; never log or return them unmasked.
- Keep the frontend depending only on libraries already in `package.json`
  (adding `npm` deps needs review).
- Update `docs/` when you add an endpoint or a user-facing feature.
