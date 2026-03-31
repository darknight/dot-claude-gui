import { connectionStore } from "./connection.svelte";
import type { PluginInfo, MarketplaceInfo, AvailablePlugin } from "$lib/api/types";

class PluginsStore {
  plugins = $state<PluginInfo[]>([]);
  marketplaces = $state<MarketplaceInfo[]>([]);
  availablePlugins = $state<AvailablePlugin[]>([]);
  loading = $state(false);
  error = $state<string>("");

  async loadPlugins() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    try {
      this.plugins = await client.listPlugins();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load plugins";
    } finally {
      this.loading = false;
    }
  }

  async loadMarketplaces() {
    const client = connectionStore.client;
    if (!client) return;
    try {
      this.marketplaces = await client.listMarketplaces();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load marketplaces";
    }
  }

  async loadMarketplacePlugins(marketplaceId: string) {
    const client = connectionStore.client;
    if (!client) return;
    try {
      this.availablePlugins = await client.getMarketplacePlugins(marketplaceId);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async togglePlugin(id: string, enabled: boolean) {
    const client = connectionStore.client;
    if (!client) return;
    try {
      await client.togglePlugin(id, enabled);
      // Update local state
      this.plugins = this.plugins.map(p => p.id === id ? { ...p, enabled } : p);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async installPlugin(name: string, marketplace: string) {
    const client = connectionStore.client;
    if (!client) return;
    try {
      return await client.installPlugin(name, marketplace);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async uninstallPlugin(id: string) {
    const client = connectionStore.client;
    if (!client) return;
    try {
      return await client.uninstallPlugin(id);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }
}

export const pluginsStore = new PluginsStore();
