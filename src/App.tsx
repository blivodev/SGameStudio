import { useEffect, useState } from "react";
import { useStudioStore } from "./store/studioStore";
import { checkHealth } from "./services/commandBridge";
import { studioSocket } from "./services/eventSocket";
import * as Tabs from "@radix-ui/react-tabs";
import * as Select from "@radix-ui/react-select";
import { PenTool, Boxes, AudioWaveform, Package, Clapperboard, Aperture, Settings } from "lucide-react";
import { Button } from "./components/ui/primitives";
import Studio2D from "./modules/studio2d/Studio2D";
import Studio3D from "./modules/studio3d/Studio3D";
import AudioStudio from "./modules/audio/AudioStudio";
import Assembler from "./modules/assembler/Assembler";
import AnimationStudio from "./modules/anim/AnimationStudio";
import VideoStudio from "./modules/video/VideoStudio";
import ChatDock from "./components/chat/ChatDock";
import SettingsDialog from "./components/settings/SettingsDialog";
import type { Theme } from "./store/studioStore";

const asModule = (v: string) => v as "2d" | "3d" | "audio" | "assembler" | "anim" | "video";

export default function App() {
  const { activeModule, setActiveModule, theme, setTheme, initTheme, connection, setConnection, projectName, targetEngine, lastError, setLastError } = useStudioStore();
  const [settingsOpen, setSettingsOpen] = useState(false);

  useEffect(() => {
    initTheme();
    studioSocket.connect();
    const t = window.setInterval(() => {
      checkHealth()
        .then((ok) => setConnection(ok ? "connected" : "disconnected"))
        .catch(() => setConnection("disconnected"));
    }, 15000);
    return () => window.clearInterval(t);
  }, [initTheme, setConnection]);

  const modules: { id: ReturnType<typeof asModule>; label: string; icon: React.ReactNode }[] = [
    { id: "2d", label: "2D Studio", icon: <PenTool className="h-4 w-4" /> },
    { id: "3d", label: "3D Studio", icon: <Boxes className="h-4 w-4" /> },
    { id: "audio", label: "Audio Studio", icon: <AudioWaveform className="h-4 w-4" /> },
    { id: "anim", label: "Animation", icon: <Aperture className="h-4 w-4" /> },
    { id: "video", label: "Video / Motion", icon: <Clapperboard className="h-4 w-4" /> },
    { id: "assembler", label: "Assembler", icon: <Package className="h-4 w-4" /> },
  ];

  return (
    <div className="flex h-screen flex-col bg-background text-text">
      <header className="flex h-12 shrink-0 items-center justify-between border-b border-border px-3">
        <div className="flex items-center gap-2">
          <div className="h-7 w-7 rounded bg-gradient-to-br from-accent to-accent-2" />
          <span className="font-bold">SGameStudio</span>
          <span className="text-xs text-muted">v0.1.0</span>
        </div>
        <div className="flex items-center gap-3">
          <Select.Root value={theme} onValueChange={(v) => setTheme(v as Theme)}>
            <Select.Trigger className="inline-flex h-7 items-center rounded border border-border bg-surface px-2 text-xs" aria-label="Theme">
              <Select.Value placeholder="Theme" />
            </Select.Trigger>
            <Select.Portal>
              <Select.Content position="popper" className="z-50 rounded border border-border bg-surface shadow-xl">
                <Select.Viewport className="p-1">
                  <Select.Item value="obsidian" className="flex cursor-pointer items-center rounded px-2 py-1 text-sm outline-none data-[highlighted]:bg-surfaceAlt">
                    <Select.ItemText>Obsidian Dark</Select.ItemText>
                  </Select.Item>
                  <Select.Item value="cyberpunk" className="flex cursor-pointer items-center rounded px-2 py-1 text-sm outline-none data-[highlighted]:bg-surfaceAlt">
                    <Select.ItemText>Cyberpunk Neon</Select.ItemText>
                  </Select.Item>
                  <Select.Item value="paper" className="flex cursor-pointer items-center rounded px-2 py-1 text-sm outline-none data-[highlighted]:bg-surfaceAlt">
                    <Select.ItemText>Paper Light</Select.ItemText>
                  </Select.Item>
                  <Select.Item value="high-contrast" className="flex cursor-pointer items-center rounded px-2 py-1 text-sm outline-none data-[highlighted]:bg-surfaceAlt">
                    <Select.ItemText>High-Contrast Pro</Select.ItemText>
                  </Select.Item>
                </Select.Viewport>
              </Select.Content>
            </Select.Portal>
          </Select.Root>
          <div className="flex items-center gap-1">
            <span className={`h-2 w-2 rounded-full ${connection === "connected" ? "bg-green-500" : "bg-red-500"}`} />
            <span className="text-xs text-muted">{connection}</span>
          </div>
          <Button variant="ghost" size="sm" onClick={() => setSettingsOpen(true)} aria-label="Settings">
            <Settings className="h-4 w-4" />
          </Button>
        </div>
      </header>
      <div className="flex min-h-0 flex-1">
        <nav className="flex w-44 shrink-0 flex-col gap-1 border-r border-border p-2">
          {modules.map((m) => (
            <Button key={m.id} variant={activeModule === m.id ? "accent" : "ghost"} size="sm" className="w-full justify-start gap-2" onClick={() => setActiveModule(m.id)}>
              {m.icon}
              <span>{m.label}</span>
            </Button>
          ))}
        </nav>
        <main className="min-w-0 flex-1">
          <Tabs.Root className="h-full" value={activeModule} onValueChange={(v) => setActiveModule(asModule(v))}>
            <Tabs.Content className="h-full outline-none" value="2d">
              <Studio2D />
            </Tabs.Content>
            <Tabs.Content className="h-full outline-none" value="3d">
              <Studio3D />
            </Tabs.Content>
            <Tabs.Content className="h-full outline-none" value="audio">
              <AudioStudio />
            </Tabs.Content>
            <Tabs.Content className="h-full outline-none" value="anim">
              <AnimationStudio />
            </Tabs.Content>
            <Tabs.Content className="h-full outline-none" value="video">
              <VideoStudio />
            </Tabs.Content>
            <Tabs.Content className="h-full outline-none" value="assembler">
              <Assembler />
            </Tabs.Content>
          </Tabs.Root>
        </main>
        <aside className="w-80 shrink-0 border-l border-border">
          <ChatDock />
        </aside>
      </div>
      <footer className="flex h-6 shrink-0 items-center gap-3 border-t border-border px-3 text-xs text-muted">
        <span>{projectName}</span>
        <span>•</span>
        <span>{targetEngine}</span>
        <span>•</span>
        <span>{connection}</span>
        <span>•</span>
        <span>MCP: stdio + sse</span>
        {lastError && (
          <span className="ml-auto flex items-center gap-1 text-red-500">
            <span className="max-w-[40ch] truncate">{lastError}</span>
            <button type="button" aria-label="Dismiss error" onClick={() => setLastError(null)} className="hover:text-red-300">
              ✕
            </button>
          </span>
        )}
      </footer>
      <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} />
    </div>
  );
}