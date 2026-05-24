import { ipcClient, IpcError } from "$lib/ipc/client.js";
import type { PluginInfo, MarketplaceInfo, AvailablePlugin } from "$lib/api/types";

// Preserve the "kind: details" prefix that IpcError carries. Vanilla Error
// has no useful toString, so we fall back to .message.
function errMsg(e: unknown, fallback: string): string {
  if (e instanceof IpcError) return e.toString();
  if (e instanceof Error) return e.message;
  return fallback;
}

class PluginsStore {
  plugins = $state<PluginInfo[]>([]);
  marketplaces = $state<MarketplaceInfo[]>([]);
  availablePlugins = $state<AvailablePlugin[]>([]);
  loading = $state(false);
  error = $state<string>("");
  // Set by callers (e.g. SkillPreview's "open owning plugin") to request the
  // installed-plugins view briefly highlight a specific plugin. The view
  // clears this back to null after the flash so re-entering the facet later
  // doesn't replay it.
  highlightedPluginId = $state<string | null>(null);

  highlightPlugin(id: string | null) {
    this.highlightedPluginId = id;
  }

  async loadPlugins() {
    this.loading = true;
    try {
      this.plugins = await ipcClient.listPlugins();
    } catch (e) {
      this.error = errMsg(e, "Failed to load plugins");
    } finally {
      this.loading = false;
    }
  }

  async loadMarketplaces() {
    try {
      this.marketplaces = await ipcClient.listMarketplaces();
    } catch (e) {
      this.error = errMsg(e, "Failed to load marketplaces");
    }
  }

  async loadMarketplacePlugins(marketplaceId: string) {
    try {
      this.availablePlugins = await ipcClient.getMarketplacePlugins(marketplaceId);
    } catch (e) {
      this.error = errMsg(e, "Failed");
    }
  }

  async togglePlugin(id: string, enabled: boolean) {
    try {
      await ipcClient.togglePlugin(id, enabled);
      // Update local state
      this.plugins = this.plugins.map(p => p.id === id ? { ...p, enabled } : p);
    } catch (e) {
      this.error = errMsg(e, "Failed");
    }
  }

  async installPlugin(name: string, marketplace: string) {
    try {
      return await ipcClient.installPlugin(name, marketplace);
    } catch (e) {
      this.error = errMsg(e, "Failed");
    }
  }

  async uninstallPlugin(
    id: string,
    opts?: { accountName?: string; cwd?: string; scope?: string },
  ) {
    try {
      return await ipcClient.uninstallPlugin(id, opts);
    } catch (e) {
      this.error = errMsg(e, "Failed");
    }
  }

  async addMarketplace(repo: string) {
    try {
      return await ipcClient.addMarketplace(repo);
    } catch (e) {
      this.error = errMsg(e, "Failed to add marketplace");
    }
  }

  async removeMarketplace(id: string) {
    try {
      return await ipcClient.removeMarketplace(id);
    } catch (e) {
      this.error = errMsg(e, "Failed to remove marketplace");
    }
  }

  reset(): void {
    this.plugins = [];
    this.marketplaces = [];
    this.availablePlugins = [];
    this.loading = false;
    this.error = "";
    this.highlightedPluginId = null;
  }
}

export const pluginsStore = new PluginsStore();
