import { Theme } from '../store/studioStore';

export const THEMES: { id: Theme; label: string }[] = [
  { id: "obsidian", label: "Obsidian Dark" },
  { id: "cyberpunk", label: "Cyberpunk Neon" },
  { id: "paper", label: "Paper Light" },
  { id: "high-contrast", label: "High-Contrast Pro" },
];

export function applyTheme(t: Theme): void {
  document.documentElement.dataset.theme = t;
  localStorage.setItem("sgame-theme", t);
}

export function storedTheme(): Theme {
  const theme = localStorage.getItem("sgame-theme");
  return theme && ["obsidian", "cyberpunk", "paper", "high-contrast"].includes(theme) ? theme as Theme : "obsidian";
}