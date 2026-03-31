import { invoke } from "@tauri-apps/api/core";
import type { ConnectionEntry, ConnectionsFile } from "$lib/api/types.js";

class ConnectionsStore {
  connections = $state<ConnectionEntry[]>([]);
  activeConnectionId = $state<string>("local");
  loading = $state<boolean>(false);
  error = $state<string>("");

  get activeConnection(): ConnectionEntry | undefined {
    return this.connections.find((c) => c.id === this.activeConnectionId);
  }

  get localConnection(): ConnectionEntry | undefined {
    return this.connections.find((c) => c.id === "local");
  }

  async load(): Promise<void> {
    this.loading = true;
    this.error = "";
    try {
      const json = await invoke<string>("read_connections");
      const data: ConnectionsFile = JSON.parse(json);
      this.connections = data.connections;
      this.activeConnectionId = data.activeConnectionId;
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    } finally {
      this.loading = false;
    }
  }

  async save(): Promise<void> {
    const data: ConnectionsFile = {
      activeConnectionId: this.activeConnectionId,
      connections: this.connections,
    };
    try {
      await invoke("write_connections", {
        json: JSON.stringify(data, null, 2),
      });
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    }
  }

  async addConnection(entry: Omit<ConnectionEntry, "id">): Promise<void> {
    const id = crypto.randomUUID();
    const newEntry: ConnectionEntry = { ...entry, id };
    this.connections = [...this.connections, newEntry];
    await this.save();
  }

  async updateConnection(
    id: string,
    updates: Partial<ConnectionEntry>
  ): Promise<void> {
    this.connections = this.connections.map((c) =>
      c.id === id ? { ...c, ...updates } : c
    );
    await this.save();
  }

  async deleteConnection(id: string): Promise<void> {
    if (id === "local") return; // Cannot delete local
    this.connections = this.connections.filter((c) => c.id !== id);
    if (this.activeConnectionId === id) {
      this.activeConnectionId = "local";
    }
    await this.save();
  }

  async setActive(id: string): Promise<void> {
    this.activeConnectionId = id;
    await this.save();
  }
}

export const connectionsStore = new ConnectionsStore();
