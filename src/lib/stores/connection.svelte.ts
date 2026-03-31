import { DaemonClient } from "$lib/api/client";
import { DaemonWsClient } from "$lib/api/ws";

type ConnectionStatus = "disconnected" | "connecting" | "connected";

class ConnectionStore {
  status = $state<ConnectionStatus>("disconnected");
  daemonVersion = $state<string>("");
  error = $state<string>("");

  client: DaemonClient | null = null;
  wsClient: DaemonWsClient | null = null;

  async connect(baseUrl: string, token: string): Promise<void> {
    this.status = "connecting";
    this.error = "";

    try {
      const client = new DaemonClient(baseUrl, token);
      const wsClient = new DaemonWsClient(baseUrl, token);

      // Verify connectivity via health check
      const health = await client.health();

      // Set up handler for the "connected" WS event before connecting
      wsClient.onEvent((event) => {
        if (event.type === "connected") {
          this.daemonVersion = event.daemonVersion;
        }
      });

      wsClient.connect();

      this.client = client;
      this.wsClient = wsClient;
      this.daemonVersion = health.version;
      this.status = "connected";
    } catch (err) {
      this.status = "disconnected";
      this.error = err instanceof Error ? err.message : String(err);
    }
  }

  disconnect(): void {
    this.wsClient?.disconnect();
    this.wsClient = null;
    this.client = null;
    this.status = "disconnected";
    this.daemonVersion = "";
  }
}

export const connectionStore = new ConnectionStore();
