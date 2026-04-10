import { ipcClient } from "$lib/ipc/client.js";
import type { PluginInfo, MarketplaceInfo, AvailablePlugin } from "$lib/api/types";

class PluginsStore {
  plugins = $state<PluginInfo[]>([]);
  marketplaces = $state<MarketplaceInfo[]>([]);
  availablePlugins = $state<AvailablePlugin[]>([]);
  loading = $state(false);
  error = $state<string>("");

  async loadPlugins() {
    this.loading = true;
    try {
      this.plugins = await ipcClient.listPlugins();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load plugins";
    } finally {
      this.loading = false;
    }
  }

  async loadMarketplaces() {
    try {
      this.marketplaces = await ipcClient.listMarketplaces();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load marketplaces";
    }
  }

  async loadMarketplacePlugins(marketplaceId: string) {
    try {
      this.availablePlugins = await ipcClient.getMarketplacePlugins(marketplaceId);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async togglePlugin(id: string, enabled: boolean) {
    try {
      await ipcClient.togglePlugin(id, enabled);
      // Update local state
      this.plugins = this.plugins.map(p => p.id === id ? { ...p, enabled } : p);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async installPlugin(name: string, marketplace: string) {
    try {
      return await ipcClient.installPlugin(name, marketplace);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async uninstallPlugin(id: string) {
    try {
      return await ipcClient.uninstallPlugin(id);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed";
    }
  }

  async addMarketplace(repo: string) {
    try {
      return await ipcClient.addMarketplace(repo);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to add marketplace";
    }
  }

  async removeMarketplace(id: string) {
    try {
      return await ipcClient.removeMarketplace(id);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to remove marketplace";
    }
  }

  reset(): void {
    this.plugins = [];
    this.marketplaces = [];
    this.availablePlugins = [];
    this.loading = false;
    this.error = "";
  }
}

export const pluginsStore = new PluginsStore();
