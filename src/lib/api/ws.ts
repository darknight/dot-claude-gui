import type { WsEvent } from "./types.js";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const RECONNECT_INITIAL_DELAY_MS = 1_000;
const RECONNECT_MAX_DELAY_MS = 30_000;
const RECONNECT_BACKOFF_FACTOR = 2;

// ---------------------------------------------------------------------------
// DaemonWsClient
// ---------------------------------------------------------------------------

export class DaemonWsClient {
  private readonly wsUrl: string;

  private ws: WebSocket | null = null;
  private shouldReconnect = false;
  private reconnectDelay = RECONNECT_INITIAL_DELAY_MS;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  private handlers = new Set<(event: WsEvent) => void>();

  constructor(baseUrl: string, token: string) {
    // Convert http(s) → ws(s) and append the auth token as a query param.
    const wsBase = baseUrl
      .replace(/\/$/, "")
      .replace(/^http:/, "ws:")
      .replace(/^https:/, "wss:");

    this.wsUrl = `${wsBase}/api/v1/ws?token=${encodeURIComponent(token)}`;
  }

  // -------------------------------------------------------------------------
  // Public API
  // -------------------------------------------------------------------------

  get connected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  connect(): void {
    this.shouldReconnect = true;
    this.openSocket();
  }

  disconnect(): void {
    this.shouldReconnect = false;
    this.clearReconnectTimer();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Register a handler for incoming events.
   * Returns an unsubscribe function.
   */
  onEvent(handler: (event: WsEvent) => void): () => void {
    this.handlers.add(handler);
    return () => {
      this.handlers.delete(handler);
    };
  }

  // -------------------------------------------------------------------------
  // Internal helpers
  // -------------------------------------------------------------------------

  private openSocket(): void {
    // Don't open a second socket while one is already connecting or open.
    if (
      this.ws !== null &&
      (this.ws.readyState === WebSocket.CONNECTING ||
        this.ws.readyState === WebSocket.OPEN)
    ) {
      return;
    }

    const ws = new WebSocket(this.wsUrl);
    this.ws = ws;

    ws.onopen = () => {
      // Reset backoff on successful connection.
      this.reconnectDelay = RECONNECT_INITIAL_DELAY_MS;
    };

    ws.onmessage = (event: MessageEvent) => {
      let parsed: WsEvent;
      try {
        parsed = JSON.parse(event.data as string) as WsEvent;
      } catch {
        // Ignore malformed frames.
        return;
      }
      for (const handler of this.handlers) {
        try {
          handler(parsed);
        } catch {
          // Prevent a bad handler from breaking other handlers.
        }
      }
    };

    ws.onclose = () => {
      this.ws = null;
      if (this.shouldReconnect) {
        this.scheduleReconnect();
      }
    };

    ws.onerror = () => {
      // onerror is always followed by onclose, so reconnect logic lives there.
    };
  }

  private scheduleReconnect(): void {
    this.clearReconnectTimer();

    const delay = this.reconnectDelay;

    // Advance backoff for next attempt, capped at max.
    this.reconnectDelay = Math.min(
      this.reconnectDelay * RECONNECT_BACKOFF_FACTOR,
      RECONNECT_MAX_DELAY_MS
    );

    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      if (this.shouldReconnect) {
        this.openSocket();
      }
    }, delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }
}
