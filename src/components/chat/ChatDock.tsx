import { useState } from "react";
import { Send, Loader2 } from "lucide-react";
import { Button } from "../ui/primitives";
import { useStudioStore } from "../../store/studioStore";
import { chatWithAssistant } from "../../services/ai";

export default function ChatDock() {
  const { chat, addChatMessage } = useStudioStore();
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);

  const send = async () => {
    const text = input.trim();
    if (!text || busy) return;
    addChatMessage({ id: crypto.randomUUID(), role: "user", text });
    setInput("");
    setBusy(true);
    try {
      const res = await chatWithAssistant(text);
      addChatMessage({ id: crypto.randomUUID(), role: "assistant", text: res.reply || "(no reply)" });
    } catch (e) {
      addChatMessage({
        id: crypto.randomUUID(),
        role: "assistant",
        text: (e as Error).message,
      });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-9 shrink-0 items-center justify-between border-b border-border px-3 text-sm font-semibold">
        <span>AI Assistant</span>
        {busy && (
          <span className="flex items-center gap-1 text-xs font-normal text-muted">
            <Loader2 className="h-3 w-3 animate-spin" /> Thinking…
          </span>
        )}
      </div>
      <div className="flex-1 space-y-2 overflow-y-auto p-3">
        {chat.length === 0 ? (
          <p className="text-xs text-muted">
            Ask the assistant to create, edit, or export assets. Open Settings → AI Provider API Keys to connect a chat provider.
          </p>
        ) : (
          chat.map((m) => (
            <div
              key={m.id}
              className={`max-w-[92%] whitespace-pre-wrap rounded-lg border px-3 py-2 text-sm ${m.role === "user" ? "self-end border-accent/40 bg-accent/20" : "self-start border-border bg-surfaceAlt"}`}
            >
              {m.text}
            </div>
          ))
        )}
      </div>
      <div className="flex shrink-0 gap-2 border-t border-border p-2">
        <input
          className="min-w-0 flex-1 rounded border border-border bg-surface px-2 py-1 text-sm"
          placeholder="Message…"
          aria-label="Chat message"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") send();
          }}
        />
        <Button size="sm" variant="ghost" onClick={send} disabled={busy} aria-label="Send">
          <Send className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
