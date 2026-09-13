import { useCallback, useEffect, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Button } from "../ui/primitives";
import { KeyRound, Loader2, Trash2, Check, Wifi, WifiOff } from "lucide-react";
import {
  PROVIDER_NAMES,
  PROVIDER_LABELS,
  DEFAULT_SETTINGS,
  loadSettings,
  saveProviderKey,
  clearProviderKey,
  type StudioSettings,
  type SaveKeyResult,
} from "../../services/settings";
import { checkHealth } from "../../services/commandBridge";

interface ProviderState {
  value: string;
  saving: boolean;
  activeConfig: { configured: boolean; keyMasked?: string | null };
}

function initialProviders(): Record<string, ProviderState> {
  const out: Record<string, ProviderState> = {};
  for (const p of PROVIDER_NAMES) {
    out[p] = { value: "", saving: false, activeConfig: { configured: false, keyMasked: null } };
  }
  return out;
}

export default function SettingsDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const [settings, setSettings] = useState<StudioSettings>(DEFAULT_SETTINGS);
  const [providers, setProviders] = useState<Record<string, ProviderState>>(initialProviders);
  const [loading, setLoading] = useState(false);
  const [savingKey, setSavingKey] = useState<string | null>(null);
  const [clearing, setClearing] = useState<string | null>(null);
  const [health, setHealth] = useState<boolean | null>(null);
  const [notice, setNotice] = useState<{ provider: string; ok: boolean; message: string } | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    const s = await loadSettings();
    setSettings(s);
    const next: Record<string, ProviderState> = {};
    for (const p of PROVIDER_NAMES) {
      const cfg = s.providers?.[p] ?? { configured: false, keyMasked: null };
      next[p] = {
        value: "",
        saving: false,
        activeConfig: { configured: cfg.configured, keyMasked: cfg.keyMasked ?? null },
      };
    }
    setProviders(next);
    setHealth(await checkHealth());
    setLoading(false);
  }, []);

  useEffect(() => {
    if (open) refresh();
  }, [open, refresh]);

  const setField = (provider: string, value: string) => {
    setProviders((prev) => ({
      ...prev,
      [provider]: { ...prev[provider], value },
    }));
  };

  const doSave = async (provider: string) => {
    const st = providers[provider];
    if (!st || !st.value.trim() || savingKey) return;
    setSavingKey(provider);
    try {
      const res: SaveKeyResult = await saveProviderKey(provider, st.value.trim());
      setNotice({ provider, ok: true, message: "Saved" });
      setProviders((prev) => ({
        ...prev,
        [provider]: {
          value: "",
          saving: false,
          activeConfig: { configured: true, keyMasked: res.keyMasked ?? null },
        },
      }));
    } catch (e) {
      setNotice({ provider, ok: false, message: (e as Error).message });
    } finally {
      setSavingKey(null);
    }
  };

  const doClear = async (provider: string) => {
    if (savingKey) return;
    setClearing(provider);
    try {
      await clearProviderKey(provider);
      setNotice({ provider, ok: true, message: "Cleared" });
      setProviders((prev) => ({
        ...prev,
        [provider]: { ...prev[provider], value: "", activeConfig: { configured: false, keyMasked: null } },
      }));
    } catch (e) {
      setNotice({ provider, ok: false, message: (e as Error).message });
    } finally {
      setClearing(null);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/60" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-h-[85vh] w-[min(720px,92vw)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border border-border bg-surface p-5 shadow-2xl">
          <div className="flex items-start justify-between gap-4">
            <div>
              <Dialog.Title className="text-lg font-bold">Studio Settings</Dialog.Title>
              <Dialog.Description className="mt-1 text-xs text-muted">
                AI provider API keys, routing, and export defaults. Keys are stored obfuscated on disk and are never returned unmasked.
              </Dialog.Description>
            </div>
            {health === null ? (
              <Loader2 className="h-4 w-4 shrink-0 animate-spin text-muted" />
            ) : health ? (
              <span className="flex shrink-0 items-center gap-1 text-xs text-green-500">
                <Wifi className="h-4 w-4" /> Backend connected
              </span>
            ) : (
              <span className="flex shrink-0 items-center gap-1 text-xs text-red-500">
                <WifiOff className="h-4 w-4" /> Backend disconnected
              </span>
            )}
          </div>

          <h3 className="mt-5 mb-2 text-sm font-semibold">AI Provider API Keys</h3>
          {!loading && (
            <div className="space-y-2">
              {PROVIDER_NAMES.map((p) => {
                const st = providers[p];
                if (!st) return null;
                const n = notice?.provider === p ? notice : null;
                const busy = savingKey === p || clearing === p;
                return (
                  <div key={p} className="flex items-center gap-2 rounded border border-border bg-background/40 p-2">
                    <div className="flex w-40 shrink-0 items-center gap-2">
                      {st.activeConfig.configured ? (
                        <Check className="h-4 w-4 shrink-0 text-green-500" />
                      ) : (
                        <span className="h-4 w-4 shrink-0 rounded-full border border-border" />
                      )}
                      <span className="truncate text-xs font-medium">{PROVIDER_LABELS[p] ?? p}</span>
                    </div>
                    {st.activeConfig.configured && st.activeConfig.keyMasked ? (
                      <span className="hidden w-32 shrink-0 truncate font-mono text-[10px] text-muted sm:block" title="Current key (masked)">
                        {st.activeConfig.keyMasked}
                      </span>
                    ) : null}
                    <input
                      type="password"
                      value={st.value}
                      onChange={(e) => setField(p, e.target.value)}
                      onKeyDown={(e) => e.key === "Enter" && doSave(p)}
                      placeholder={st.activeConfig.configured ? "•••••••• (type to replace)" : "Paste your API key…"}
                      className="min-w-0 flex-1 rounded border border-border bg-background px-2 py-1 text-xs outline-none focus:border-accent"
                      autoComplete="off"
                      spellCheck={false}
                    />
                    {n && <span className={`w-28 shrink-0 truncate text-[10px] ${n.ok ? "text-green-500" : "text-red-500"}`}>{n.message}</span>}
                    <Button size="sm" disabled={busy || !st.value.trim()} onClick={() => doSave(p)} aria-label={`Save ${p} API key`}>
                      {busy ? <Loader2 className="h-3 w-3 animate-spin" /> : <KeyRound className="h-3 w-3" />}
                      Save
                    </Button>
                    {st.activeConfig.configured && (
                      <Button
                        size="sm"
                        variant="ghost"
                        disabled={busy}
                        onClick={() => doClear(p)}
                        aria-label={`Clear ${p} API key`}
                        className="text-red-400 hover:text-red-300"
                      >
                        <Trash2 className="h-3 w-3" />
                      </Button>
                    )}
                  </div>
                );
              })}
            </div>
          )}

          <h3 className="mt-6 mb-2 text-sm font-semibold">AI Routing</h3>
          <table className="w-full text-xs">
            <thead>
              <tr>
                <th className="py-1 text-left font-medium text-muted">Task</th>
                <th className="py-1 text-left font-medium text-muted">Provider</th>
                <th className="py-1 text-left font-medium text-muted">Model</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(settings.aiRouting).map(([task, { provider, model }]) => (
                <tr key={task}>
                  <td className="py-1 pr-4">{task}</td>
                  <td className="py-1 pr-4">{provider}</td>
                  <td className="py-1 font-mono">{model || "default"}</td>
                </tr>
              ))}
            </tbody>
          </table>

          <h3 className="mt-6 mb-2 text-sm font-semibold">Export Defaults</h3>
          <div className="flex flex-wrap gap-2">
            {Object.values(settings.exportDefaults).map((v) => (
              <span key={v} className="rounded border border-border px-2 py-1 text-xs">
                {v}
              </span>
            ))}
          </div>

          <footer className="mt-6 flex justify-between">
            <Button variant="ghost" size="sm" onClick={() => refresh()} disabled={loading}>
              {loading ? <Loader2 className="h-3 w-3 animate-spin" /> : "Refresh"}
            </Button>
            <Dialog.Close asChild>
              <Button size="sm">Close</Button>
            </Dialog.Close>
          </footer>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
