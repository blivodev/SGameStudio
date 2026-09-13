# SGameStudio

> **The Open-Source, AI-Native, All-in-One Game Asset Studio**  
> Create 2D vector art & animations, intuitive 3D models & rigs, sound effects, vocals, and music—assembled into ready-to-export game assets for Godot, Unity, and Unreal. Fully controllable via **GUI**, **Headless API**, and **Model Context Protocol (MCP)**.

---

## 🌟 Executive Overview

**SGameStudio** is a next-generation creative suite tailored for game developers, technical artists, and autonomous AI agents. Built entirely on free and open-source software (FOSS), SGameStudio eliminates the friction of jumping between 5 different specialized tools (Blender, Illustrator/Inkscape, Spine, Audacity, and DAW trackers) by uniting the four pillars of game asset creation inside one cohesive, easy-to-use desktop application:

1. **2D Studio**: Full vector drafting, path booleans, bone/mesh skeletal rigging, 2D inverse kinematics, dopesheet animation, and cinematic sequencers.
2. **3D Studio**: Dual-mode modeling uniting **Dust3D-style node-based modeling** (draw 3D nodes/spheres $\to$ instant watertight mesh $\to$ inherent auto-rigging & auto-UVs for creatures/characters) and **Wings3D/SketchUp-style box modeling** (extrude/inset/bevel for weapons, props, and armor), plus skeletal animation and cinematic cutscenes.
3. **Audio Studio**: Procedural sound effects synthesizer (sfxr/foley DSP), neural text-to-speech vocals with modulation, and pattern-based music tracker.
4. **Game Asset Assembler**: A unified entity compositor that binds 2D/3D meshes, rigs, animation states, audio event triggers, collision hulls, and sockets into one asset package. Includes an **Automated Code Generator** that produces ready-to-run character controller scripts (GDScript, C#, C++, Rust, TypeScript, C/Raylib) and exports to **Godot 4**, **Unity**, **Unreal Engine 5**, and indie engines in one click.

---

## 🖱️ One-Click Launch & Zero-Terminal Architecture

> **No command-line wrestling. No 10 terminal windows. Just one click to start creating.**

SGameStudio is built for creative flow, not DevOps debugging. Unlike typical open-source AI tools that force you to open multiple command-line prompts, manage Python environments, and manually start servers:

- **Single Click Launch**: Open the app with a single desktop click (or desktop shortcut). No terminal windows will ever pop up.
- **Embedded Service Supervisor**: All required background services (REST API, WebSocket bridge, audio synthesis engine, and local AI workers) are automatically spawned, monitored, and shut down **invisibly inside the app**.
- **Internal Sandboxed Downloads**: Any models, neural voice weights, libraries, or extra tools you download are managed **completely inside the application's isolated sandbox directory**—never polluting your system PATH, system Python, or requiring `pip`/`npm` commands.
- **In-App Model & Tool Manager**: Download or update local AI models (Piper TTS voices, Trellis 3D, ComfyUI workflows) through an intuitive in-app UI with visual progress bars and one-click toggles.
- **Clean Background Teardown**: When you close SGameStudio, the internal supervisor cleanly terminates all child worker processes via OS Job Objects / process groups. Zero orphan zombie processes left in Task Manager.
- *(Advanced users & CI/CD can still optionally use the headless CLI flags like `--server` or `--mcp` when desired!)*

---

## 🤖 AI-Native Architecture & Model Context Protocol (MCP)

SGameStudio is engineered from the ground up to be **AI-first**:
- **Universal AI Integration**: Every tool, action, transformation, and property in the software can be triggered and modified by AI.
- **Pluggable Provider Matrix**: Connect local models (Ollama, ComfyUI, LocalAI, vLLM) or cloud API providers (OpenRouter, DeepInfra, OpenAI, Google Gemini, Anthropic, Groq, Mistral).
- **Granular Task Routing**: Assign individual models and providers to specific jobs (e.g., DeepSeek via OpenRouter for Audio prompts, Gemini via DeepInfra for the interactive Chat assistant, Trellis/Hunyuan3D for 3D generation, Flux/SDXL via ComfyUI for 2D sprites, Piper for offline neural vocals).
- **Interactive Chat Assistant**: Built-in conversational side-panel with complete workspace awareness, tool calling, and viewport screenshot analysis.
- **100% MCP Controllable**: Includes an embedded **Model Context Protocol (MCP)** server. AI agents in Claude Desktop, Cursor, Antigravity, or custom autonomous pipelines can pilot the studio over standard `stdio` or HTTP/SSE streams.
- **Dual Mode (Desktop GUI & Headless Server)**: Run as a rich desktop GUI on Windows and Linux, or as a headless server / microservice (`--server --port 8080`) controllable via REST API, WebSockets, and CLI scripts for CI/CD game asset pipelines.

---

## 🛠️ Technology Stack (100% Free & Open-Source)

| Domain | Selected Framework / Library | License | Justification |
| :--- | :--- | :--- | :--- |
| **Desktop Shell** | **Tauri v2** (Rust + Native Webview) | MIT / Apache 2.0 | Ultra-lightweight (25MB binary vs 180MB Electron), low memory footprint, blazing native Rust backend for heavy I/O, threading, and hardware access. |
| **Frontend UI** | **React 19 + TypeScript + Tailwind CSS + Radix UI + Lucide** | MIT | Fluid, responsive, modern studio aesthetics with high-velocity component development, glassmorphism, micro-animations, and multi-theme support. |
| **2D Vector Engine** | **Paper.js + Skia PathKit (WASM) + PixiJS** | MIT / BSD-3 | Professional Bezier curves, Boolean path operations (Union, Subtract, Intersect, Exclude), SVG parser/serializer, hardware-accelerated 2D mesh deformation shaders. |
| **3D Modeling & Viewport** | **Three.js + WebGPU + libdust3d (Rust/WASM) + Half-Edge B-Rep** | MIT / Apache 2.0 | High-performance 3D viewport, real-time PBR, Dust3D-style instant node-to-mesh generation with inherent auto-rigging, and Wings3D-style face/edge extrusion & beveling. |
| **Audio Engine** | **Web Audio API + Rust Rodio / Symphonia + sfxr DSP + Faust WASM** | MIT / Apache 2.0 | Zero-latency procedural SFX synthesis, parametric audio filters, real-time waveform visualization, and tracker playback. |
| **AI Worker Engine** | **Rust Tokio Async Client + Python FastMCP Sidecar (Optional for Local PyTorch)** | MIT / Apache 2.0 | Native async HTTP/SSE connection to Ollama/OpenRouter/DeepInfra/ComfyUI; optional local microservice for local models like Piper TTS, Trellis, Hunyuan3D. |
| **MCP Server** | **Rust `mcp-core` / embedded stdio & SSE transport** | MIT | Native Model Context Protocol server exposing all studio tools, resources, and prompt templates to AI agents. |
| **Headless Server & API** | **Axum (Rust) + WebSocket Gateway** | MIT | Industrial-grade REST endpoints, real-time duplex WebSocket sync for remote collaboration and headless CI/CD automation. |
| **Data & Packaging** | **SQLite + glTF-Transform + ZIP/Tar** | Public Domain / MIT | Fast embedded project state database, lossless GLTF 2.0 / FBX / SVG / WAV interchange. |

---

## 📂 Documentation Directory

Comprehensive project documentation is organized in the `docs/` folder:

1. [**docs/ARCHITECTURE.md**](docs/ARCHITECTURE.md):  
   Deep-dive technical architecture covering the Core Rust engine, UI layout, 2D Vector & Rigging suite, 3D Modeling & Rigging suite, Audio DSP & Sequencer suite, AI Provider Matrix & Routing, Game Asset Assembler, and Export Pipelines.
2. [**docs/MCP_AND_API_SPECIFICATION.md**](docs/MCP_AND_API_SPECIFICATION.md):  
   Complete specification of all Model Context Protocol (MCP) tools, resources, and prompts, as well as the REST API and WebSocket events for headless automation.
3. [**docs/IMPLEMENTATION_ROADMAP.md**](docs/IMPLEMENTATION_ROADMAP.md):  
   Step-by-step implementation milestones, sprint-by-sprint execution plan, project folder structure, automated testing framework, and starter asset pack specifications.
4. [**docs/USER_MANUAL_ELI5.md**](docs/USER_MANUAL_ELI5.md):  
   A plain-English user manual — how to draw, animate, build, and connect the AI assistant.
5. [**docs/DEVELOPER_GUIDE_ELI5.md**](docs/DEVELOPER_GUIDE_ELI5.md):  
   A plain-English contributor guide — where things live, how to add endpoints/panels, how to test, and how to publish a new `.exe`.

### Quick start with AI

1. Open **Settings** (gear icon, top-right).
2. Under **AI Provider API Keys**, paste a key for any provider (OpenRouter,
   DeepInfra, OpenAI, Anthropic, or Gemini) and click **Save**.
3. Ask the **AI Assistant** (right-hand chat) anything — it now runs through the
   configured provider automatically. Keys are stored obfuscated and are never
   returned unmasked.

---

## 🚀 Quick Start & Launch Modes
 
### 1. Primary Desktop Mode (Just Click & Launch)
Simply double-click `SGameStudio.exe` (Windows) or `SGameStudio.AppImage` (Linux), or click the desktop shortcut icon.  
The app starts immediately with zero console windows. All internal engines, webviews, and local services are orchestrated silently in the background.

### 2. Optional Advanced Headless & CLI Modes (For CI/CD & External Agents)
For automated pipelines, headless servers, or external AI coding agents, SGameStudio can also be invoked via command line:

```bash
# Launch in Headless Server Mode with REST & WebSocket API on port 8080
sgame-studio --server --port 8080 --host 0.0.0.0

# Launch in Headless MCP Mode for external AI Agent control (Stdio)
sgame-studio --mcp

# Launch Headless Server with both Web API and MCP over SSE
sgame-studio --server --port 8080 --mcp-sse

# Run an automated asset creation batch script
sgame-studio run-script ./scripts/build_knight_asset.json --export-to godot4 --out ./dist
```

### CLI Command Options

```text
SGameStudio - All-in-one AI-Native Game Asset Creation Studio

USAGE:
    sgame-studio [OPTIONS] [COMMAND]

OPTIONS:
    -s, --server              Run in headless server mode (no GUI window)
    -p, --port <PORT>         Port for HTTP REST & WebSocket API [default: 8080]
        --host <HOST>         Host interface to bind [default: 127.0.0.1]
    -m, --mcp                 Expose Model Context Protocol (MCP) server on stdio
        --mcp-sse             Expose MCP server over Server-Sent Events (SSE) at /mcp/sse
    -c, --config <PATH>       Path to custom settings.json configuration file
    -t, --theme <THEME>       Override default theme [obsidian, cyberpunk, paper, high-contrast]
        --no-gpu              Force software fallback rendering for headless environments
    -v, --version             Print version information
    -h, --help                Print help information

COMMANDS:
    run-script <SCRIPT>       Execute an automated asset creation workflow script
    export <PROJECT>          Export a project or asset package to engine target
    test                      Run comprehensive self-diagnostics and integration test suite
```

---

## 🎨 Themes & UI Aesthetics

SGameStudio features a bespoke, glassmorphic studio interface built for long creative sessions:
- **Obsidian Dark** *(Default)*: Deep slate and charcoal background with neon indigo/violet accents, optimized for OLED and eye comfort.
- **Cyberpunk Neon**: High-contrast dark violet with electric cyan and magenta highlights.
- **Paper Light**: Clean, studio-grade alabaster and light grey with navy typography, ideal for bright workspaces.
- **High-Contrast Pro**: Maximum contrast monochrome palette adhering to WCAG AAA accessibility standards.

---

## 📄 License & Open-Source Compliance

SGameStudio is licensed under the **GNU General Public License v3.0 (GPL-3.0)** or **MIT License** (dual-licensing for non-GPL modular plugins). All third-party libraries, dependencies, neural model weights, and tools integrated into SGameStudio are strictly verified to be Free and Open-Source Software (FOSS), with permissive or copyleft licenses (MIT, Apache 2.0, BSD-3-Clause, GPL/LGPL, Creative Commons Zero / CC-BY).
