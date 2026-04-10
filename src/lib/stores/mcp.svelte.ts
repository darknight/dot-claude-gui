import { ipcClient } from "$lib/ipc/client.js";
import type { McpServerInfo, AddMcpServerRequest } from "$lib/api/types";

class McpStore {
  servers = $state<McpServerInfo[]>([]);
  loading = $state(false);
  error = $state<string>("");

  async loadServers() {
    this.loading = true;
    this.error = "";
    try {
      this.servers = await ipcClient.listMcpServers();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    } finally {
      this.loading = false;
    }
  }

  async addServer(req: AddMcpServerRequest) {
    try {
      return await ipcClient.addMcpServer(req);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async removeServer(name: string, scope?: string) {
    try {
      return await ipcClient.removeMcpServer(name, scope);
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
