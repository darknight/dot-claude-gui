import { connectionStore } from "./connection.svelte";
import type { McpServerInfo, AddMcpServerRequest } from "$lib/api/types";

class McpStore {
  servers = $state<McpServerInfo[]>([]);
  loading = $state(false);
  error = $state<string>("");

  async loadServers() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.servers = await client.listMcpServers();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    } finally {
      this.loading = false;
    }
  }

  async addServer(req: AddMcpServerRequest) {
    const client = connectionStore.client;
    if (!client) return;
    try {
      return await client.addMcpServer(req);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async removeServer(name: string, scope?: string) {
    const client = connectionStore.client;
    if (!client) return;
    try {
      return await client.removeMcpServer(name, scope);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  reset(): void {
    this.servers = [];
    this.loading = false;
    this.error = "";
  }
}

export const mcpStore = new McpStore();
