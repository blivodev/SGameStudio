import * as Dialog from "@radix-ui/react-dialog";
import { Button } from "../ui/primitives";
import { DEFAULT_SETTINGS } from "../../services/settings";

export default function SettingsDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/60" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[min(680px,92vw)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-border bg-surface p-5 shadow-2xl">
          <Dialog.Title className="text-lg font-bold">Studio Settings</Dialog.Title>
          <Dialog.Description className="mt-1 text-xs text-muted">
            Mirrors config/default_settings.json. API keys live in the OS keyring in the desktop build.
          </Dialog.Description>

          <h3 className="mt-4 mb-2 text-sm font-semibold">AI Routing</h3>
          <table className="w-full text-xs">
            <thead>
              <tr>
                <th className="py-1 text-left font-medium text-muted">Task</th>
                <th className="py-1 text-left font-medium text-muted">Provider</th>
                <th className="py-1 text-left font-medium text-muted">Model</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(DEFAULT_SETTINGS.aiRouting).map(([task, { provider, model }]) => (
                <tr key={task}>
                  <td className="py-1 pr-4">{task}</td>
                  <td className="py-1 pr-4">{provider}</td>
                  <td className="py-1 font-mono">{model || "default"}</td>
                </tr>
              ))}
            </tbody>
          </table>

          <h3 className="mt-6 mb-2 text-sm font-semibold">Providers</h3>
          <div className="flex flex-wrap gap-2">
            {["openrouter", "deepinfra", "gemini", "openai", "anthropic", "ollama_local", "comfyui_local"].map((p) => (
              <span key={p} className="rounded border border-border px-2 py-1 text-xs">
                {p}
              </span>
            ))}
          </div>

          <h3 className="mt-6 mb-2 text-sm font-semibold">Export Defaults</h3>
          <div className="flex flex-wrap gap-2">
            {Object.values(DEFAULT_SETTINGS.exportDefaults).map((v) => (
              <span key={v} className="rounded border border-border px-2 py-1 text-xs">
                {v}
              </span>
            ))}
          </div>

          <footer className="mt-6 flex justify-end">
            <Dialog.Close asChild>
              <Button size="sm">Close</Button>
            </Dialog.Close>
          </footer>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}