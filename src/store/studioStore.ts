import { create } from 'zustand';

type StudioModule = "2d" | "3d" | "audio" | "assembler";
export type Theme = "obsidian" | "cyberpunk" | "paper" | "high-contrast";

interface EntityInfo {
  id: string;
  kind: string;
  name: string;
  label: string;
}

interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  text: string;
}

interface StudioStore {
  activeModule: StudioModule;
  setActiveModule: (module: StudioModule) => void;
  theme: Theme;
  setTheme: (theme: Theme) => void;
  initTheme: () => void;
  entities: EntityInfo[];
  addEntity: (entity: EntityInfo) => void;
  clearEntities: () => void;
  connection: "disconnected" | "connecting" | "connected";
  setConnection: (status: "disconnected" | "connecting" | "connected") => void;
  lastError: string | null;
  setLastError: (error: string | null) => void;
  chat: ChatMessage[];
  addChatMessage: (message: ChatMessage) => void;
  projectName: string;
  targetEngine: string;
}

export const useStudioStore = create<StudioStore>((set) => ({
  activeModule: "2d",
  setActiveModule: (module) => set({ activeModule: module }),
  theme: "obsidian",
  setTheme: (theme) => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("sgame-theme", theme);
    set({ theme });
  },
  initTheme: () => {
    const storedTheme = localStorage.getItem("sgame-theme") as Theme | null;
    if (storedTheme && ["obsidian", "cyberpunk", "paper", "high-contrast"].includes(storedTheme)) {
      document.documentElement.dataset.theme = storedTheme;
      set({ theme: storedTheme });
    }
  },
  entities: [],
  addEntity: (entity) => set((state) => ({ entities: [...state.entities, entity] })),
  clearEntities: () => set({ entities: [] }),
  connection: "disconnected",
  setConnection: (status) => set({ connection: status }),
  lastError: null,
  setLastError: (error) => set({ lastError: error }),
  chat: [],
  addChatMessage: (message) => set((state) => ({ chat: [...state.chat, message] })),
  projectName: "Untitled",
  targetEngine: "godot4",
}));