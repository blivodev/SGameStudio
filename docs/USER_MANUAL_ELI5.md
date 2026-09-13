# SGameStudio — User Manual (Plain English)

This guide explains SGameStudio the way you'd explain it to a friend. No jargon, no
assumed knowledge. It is written "like I'm five" (ELI5): short sentences, simple
words, and concrete steps.

> Screenshots will be added here once the UI is final. Each section marks where an
> image will go with `[screenshot: ...]`.

---

## 1. What is SGameStudio?

SGameStudio is a program for making **game art and sound** in one place. You can:

- Draw simple 2D shapes and make a little animation with them.
- Build 3D shapes and save them as common 3D files (OBJ, STL, GLB, PLY).
- Make sound effects and music with buttons and a simple step-tracker.
- Chat with an AI assistant that can answer questions and create things for you.

Everything is free and open source. Anyone can use it — no paid accounts required,
though the AI features need keys from the providers you choose (see Section 5).

It opens as a normal desktop program (an `.exe` on Windows). It can also run as a
"server" for automation, but normal users don't need to worry about that.

---

## 2. The Main Screen

When you open SGameStudio you see four main sections switched by the buttons down the
left side:

| Button | What it does |
| :--- | :--- |
| **2D Studio** | Draw shapes and animate them. |
| **3D Studio** | Build 3D shapes and export them. |
| **Audio Studio** | Make sound effects, voices, and music. |
| **Assembler** | Combine art, animation, and audio into a game "character". |

On the **right side** there is an AI chat panel.
At the **top** there is the Settings button (the gear) and a small dot that shows
whether the app is **connected** (green) or not (red).

`[screenshot: main window]`

---

## 3. 2D Studio — Draw and Animate

The 2D Studio is a simple drawing canvas with a timeline at the bottom.

### Draw a shape
1. Pick a tool on the left: **Select**, **Rectangle**, **Ellipse**, or **Star**.
2. If you picked a shape, **click and drag** on the canvas. A shape appears.
3. The shape is now listed in **Layers** on the right.

### Move a shape
1. Click the **Select** tool.
2. Click a shape to pick it (a dashed box appears).
3. **Drag** the shape to move it.

### Animate a shape (keyframes)
A keyframe is a "frozen moment" you save. The computer draws the in-between steps.

1. Select a shape.
2. Move the timeline **scrubber** to a frame (like `Frame 10`).
3. Drag the shape where you want it at that moment.
4. Click **Record Keyframe @ 10**.
5. Move to another frame, move the shape, record again.
6. Press **Play** ▶ and the shape glides between your keyframes.

`[screenshot: 2d studio with keyframes]`

---

## 4. 3D Studio — Build and Export

The 3D Studio is a 3D view. Use the **Add Cube / Cylinder / Sphere / Torus** buttons to
put shapes in the scene. Click and drag on the view to spin the camera; use the mouse
wheel to zoom in and out.

### Export a 3D shape
1. In the right **Export Mesh** panel, choose which mesh to export.
2. Click **OBJ**, **STL**, **GLB**, or **PLY**.
3. The app saves a file to your computer Downloads folder (or browser download area).

These are standard formats you can use in Blender, Godot, Unity, Unreal, and most 3D
software.

`[screenshot: 3d studio export panel]`

---

## 5. Settings — AI Provider API Keys

Some AI features (like the chat assistant) need an internet provider. You must tell
the app where to send requests and give it a key.

> A provider is a company that runs AI models for you, for example OpenRouter,
> DeepInfra, OpenAI, Anthropic, or Google Gemini. You sign up at their website to get
> a "key" — a long secret password for their service.

1. Click the **gear icon** (top right) to open **Studio Settings**.
2. Under **AI Provider API Keys**, each provider has a space to paste its key.
3. Paste the key and click **Save**. A green dot appears next to the provider.
4. To stop using a key, click the **trash** button.

Your key is kept safe: the app hides it (`sk-a••••mnop`) and never sends the full key
back to your browser.

`[screenshot: settings dialog with api keys]`

### Why does the app say "disconnected"?

"Disconnected" means the app's built-in brain (the backend) is not reachable. In the
normal desktop build it starts automatically, so you should see **connected** (green).
If you see **disconnected**, the backend may still be starting up, or a firewall/port
conflict is blocking it. Restarting the app usually fixes it.

---

## 6. Audio Studio — Sounds and Music

- **Sound Effects (SFX)**: Click a preset button (laser, explosion, coin, jump…) to
  generate a retro game sound.
- **Vocals**: Type some text, pick a voice and an effect (robot, monster…), and click
  **Generate** to make a spoken line.
- **Music Tracker**: Turn on steps for the **Lead** and **Drums** rows, then click
  **Compose** to build a short loop.

`[screenshot: audio studio]`

---

## 7. Helpful Tips

- If something goes wrong, a red message appears at the bottom of the screen. It tells
  you what happened.
- To connect the AI assistant, first add an API key in **Settings** for any provider,
  then ask it anything in the chat panel on the right.
- Your shapes appear in the **Layers** list. Click a layer to select that shape.

---

## 8. Where is my work saved?

Your AI provider keys are saved in a private folder on your computer:

- Windows: `%APPDATA%\sgamestudio\providers.json`

You can also set keys using environment variables (more advanced), for example
`SGAME_OPENROUTER_KEY`. Environment-variable keys are the safest: they are never
written to disk.
