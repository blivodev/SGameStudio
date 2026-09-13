import { useStudioStore } from '../store/studioStore';

type WsMessageHandler = (msg: unknown) => void;

export class StudioSocket {
  private ws: WebSocket | null = null;
  private handlers: Set<WsMessageHandler> = new Set();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private manuallyClosed = false;

  connect(): Promise<void> {
    return new Promise((resolve) => {
      useStudioStore.getState().setConnection("connecting");
      const apiBase = import.meta.env.VITE_SGAME_API_BASE ?? "http://127.0.0.1:8080";
      const wsUrl = apiBase.replace(/^http/, 'ws') + '/ws';

      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        useStudioStore.getState().setConnection("connected");
        resolve();
      };

      this.ws.onclose = () => {
        useStudioStore.getState().setConnection("disconnected");
        if (!this.manuallyClosed && this.reconnectAttempts < this.maxReconnectAttempts) {
          setTimeout(() => {
            this.reconnectAttempts++;
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, 10000);
            this.connect();
          }, this.reconnectDelay);
        }
      };

      this.ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          if (msg.event === "scene_mutated" || msg.event === "entity_created" || msg.event === "entity_deleted") {
            console.debug("WS event:", msg);
          }
          this.handlers.forEach((handler) => handler(msg));
        } catch (err) {
          console.error("Failed to parse WS message:", err);
        }
      };

      this.ws.onerror = (err) => {
        console.error("WebSocket error:", err);
      };
    });
  }

  subscribe(handler: WsMessageHandler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  close() {
    this.manuallyClosed = true;
    if (this.ws) this.ws.close();
  }
}

export const studioSocket = new StudioSocket();