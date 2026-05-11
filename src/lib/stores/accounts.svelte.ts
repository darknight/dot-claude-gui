import { ipcClient } from "$lib/ipc/client";
import { appSettingsStore } from "./appsettings.svelte";
import type { Account, AccountStatus, DiskAccount } from "$lib/api/types";

function unixToIso(secs: number): string {
  return new Date(secs * 1000).toISOString();
}

/**
 * Reconcile disk truth with config metadata. Disk wins (orphan dirs surface).
 * `createdAt` prefers config metadata; falls back to disk mtime.
 */
function reconcile(disk: DiskAccount[], configAccounts: Account[]): Account[] {
  const byName = new Map(configAccounts.map((a) => [a.name, a]));
  return disk.map((d) => {
    const fromConfig = byName.get(d.name);
    return {
      name: d.name,
      displayName: fromConfig?.displayName ?? d.name,
      isNative: false, // disk accounts are never native; default is virtual
      createdAt: fromConfig?.createdAt ?? unixToIso(d.createdAtUnix),
    };
  });
}

class AccountsStore {
  accounts = $state<Account[]>([]);
  /** Per-account login status, keyed by name. Refreshed on focus. */
  statuses = $state<Record<string, AccountStatus>>({});

  async loadAccounts(): Promise<void> {
    try {
      const disk = await ipcClient.listAccounts();
      const configAccounts = appSettingsStore.preferences.accounts ?? [];
      const fromDisk = reconcile(disk, configAccounts);
      const native = configAccounts.filter((a) => a.isNative);
      // Native first, then disk-backed, sorted by name.
      const sorted = [...fromDisk].sort((a, b) => a.name.localeCompare(b.name));
      this.accounts = [...native, ...sorted];
    } catch {
      this.accounts = [];
    }
    await this.loadStatuses();
  }

  /** Fetch login status for every known account in parallel. */
  async loadStatuses(): Promise<void> {
    const next: Record<string, AccountStatus> = {};
    await Promise.all(
      this.accounts.map(async (a) => {
        try {
          next[a.name] = await ipcClient.getAccountStatus(a.name);
        } catch {
          next[a.name] = { loggedIn: false };
        }
      }),
    );
    this.statuses = next;
  }

  async refreshStatus(name: string): Promise<void> {
    let status: AccountStatus;
    try {
      status = await ipcClient.getAccountStatus(name);
    } catch {
      status = { loggedIn: false };
    }
    this.statuses = { ...this.statuses, [name]: status };
  }

  async createAccount(name: string): Promise<Account> {
    const disk = await ipcClient.createAccount(name);
    const acct: Account = {
      name: disk.name,
      displayName: disk.name,
      isNative: false,
      createdAt: unixToIso(disk.createdAtUnix),
    };
    this.accounts = [...this.accounts, acct].sort((a, b) =>
      a.name.localeCompare(b.name),
    );
    const next = [...(appSettingsStore.preferences.accounts ?? []), acct];
    await appSettingsStore.update({ accounts: next });
    await this.refreshStatus(name);
    return acct;
  }

  async deleteAccount(name: string): Promise<void> {
    await ipcClient.deleteAccount(name);
    this.accounts = this.accounts.filter((a) => a.name !== name);
    const next = (appSettingsStore.preferences.accounts ?? []).filter(
      (a) => a.name !== name,
    );
    await appSettingsStore.update({ accounts: next });
    const { [name]: _removed, ...rest } = this.statuses;
    this.statuses = rest;
  }

  has(name: string | undefined): boolean {
    if (!name) return false;
    return this.accounts.some((a) => a.name === name);
  }
}

export const accountsStore = new AccountsStore();
