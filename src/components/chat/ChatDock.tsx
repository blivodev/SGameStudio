import { useState } from "react";
import { Send } from "lucide-react";
import { Button } from "../ui/primitives";
import { useStudioStore } from "../../store/studioStore";

export default function ChatDock() {
  const { chat, addChatMessage } = useStudioStore();
  const [input, setInput] = useState("");

  const send = () => {
    const text = input.trim();
    if (!text) return;
    addChatMessage({ id: crypto.randomUUID(), role: "user", text });
    setInput("");
    window.setTimeout(() => {
      addChatMessage({ id: crypto.randomUUID(), role: "assistant", text: "AI providers are configured in Settings → AI Routing. Connect a provider (OpenRouter, Gemini, Ollama, ComfyUI…) to enable the studio assistant." });
    }, 350);
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-9 shrink-0 items-center border-b border-border px-3 text-sm font-semibold">AI Assistant</div>
      <div className="flex-1 space-y-2 overflow-y-auto p-3">
        {chat.length === 0 ? (
          <p className="text-xs text-muted">Connect a chat provider in Settings → AI Routing to enable the studio assistant.</p>
        ) : (
          chat.map((m) => (
            <div
              key={m.id}
              className={`max-w-[85%] whitespace-pre-wrap rounded-lg border px-3 py-2 text-sm ${m.role === "user" ? "self-end border-accent/40 bg-accent/20" : "self-start border-border bg-surfaceAlt"}`}
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
        <Button size="sm" variant="ghost" onClick={send} aria-label="Send">
          <Send className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}