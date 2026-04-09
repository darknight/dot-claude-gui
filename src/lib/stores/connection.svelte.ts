import { DaemonClient } from "$lib/api/client.js";
import { DaemonWsClient } from "$lib/api/ws.js";
import { configStore } from "./config.svelte.js";
import { toastStore } from "./toast.svelte.js";
import { projectsStore } from "./projects.svelte.js";
import { pluginsStore } from "./plugins.svelte.js";
import { skillsStore } from "./skills.svelte.js";
import { memoryStore } from "./memory.svelte.js";
import { mcpStore } from "./mcp.svelte.js";
import { connectionsStore } from "./connections.svelte.js";
import type { ConnectionEntry } from "$lib/api/types.js";

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

      const health = await client.health();

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
      toastStore.info("Connected to daemon");
    } catch (err) {
      this.status = "disconnected";
      this.error = err instanceof Error ? err.message : String(err);
      toastStore.error("Connection failed: " + this.error);
    }
  }

  disconnect(): void {
    this.wsClient?.disconnect();
    this.wsClient = null;
    this.client = null;
    this.status = "disconnected";
    this.daemonVersion = "";
  }

  resetAllStores(): void {
    configStore.reset();
    projectsStore.reset();
    pluginsStore.reset();
    skillsStore.reset();
    memoryStore.reset();
    mcpStore.reset();
  }

  async switchConnection(entry: ConnectionEntry): Promise<void> {
    this.disconnect();
    this.resetAllStores();
    await connectionsStore.setActive(entry.id);
    await this.connect(entry.url, entry.token);
  }
}

export const connectionStore = new ConnectionStore();
