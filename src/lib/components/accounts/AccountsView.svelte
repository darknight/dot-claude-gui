<script lang="ts">
  import { onMount } from "svelte";
  import { ipcClient } from "$lib/ipc/client";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte";
  import { configStore } from "$lib/stores/config.svelte";
  import { t } from "$lib/i18n";

  let newName = $state("");
  let nameError = $state<string>("");
  let busy = $state(false);

  const NAME_RE = /^[a-z0-9_-]{1,32}$/;

  function validateName(name: string): string {
    if (!name) return "";
    if (!NAME_RE.test(name)) return t("accounts.invalidName");
    if (accountsStore.accounts.some((a) => a.name === name)) {
      return t("accounts.duplicate", { name });
    }
    return "";
  }

  $effect(() => {
    nameError = validateName(newName.trim());
  });

  async function accountConfigDir(name: string): Promise<string> {
    const root = await ipcClient.getConfigDir();
    return `${root}/accounts/${name}`;
  }

  async function launchOAuth(name: string): Promise<void> {
    const dir = await accountConfigDir(name);
    const userEnv = configStore.userSettings.env ?? {};
    // Carry over user-level Claude env (proxies, ANTHROPIC_BASE_URL, …) so OAuth
    // works behind corporate proxies / on self-hosted endpoints. CLAUDE_CONFIG_DIR
    // is appended last so it can't be shadowed by a settings.json entry.
    const env: Record<string, string> = {
      ...Object.fromEntries(
        Object.entries(userEnv).map(([k, v]) => [k, String(v)]),
      ),
      CLAUDE_CONFIG_DIR: dir,
    };
    // No projectPath: claude auto-enters onboarding for fresh config dirs.
    // `--setting-sources project,local` keeps claude from reading the global
    // ~/.claude/settings.json, so the new account doesn't silently inherit
    // enabledPlugins / extraKnownMarketplaces / hooks from the user's main
    // setup. Each account stays managed entirely by dot-claude-gui.
    await ipcClient.launchClaude({
      env,
      args: ["--setting-sources", "project,local"],
      preferredTerminal: appSettingsStore.preferences.preferredTerminal ?? "terminal",
    });
  }

  // When the user returns from terminal-side OAuth, refresh statuses so the
  // UI flips from "not logged in" to the new email.
  onMount(() => {
    const handler = () => { void accountsStore.loadStatuses(); };
    window.addEventListener("focus", handler);
    return () => window.removeEventListener("focus", handler);
  });

  async function add() {
    const name = newName.trim();
    if (!name || nameError) return;
    busy = true;
    try {
      await accountsStore.createAccount(name);
      newName = "";
      // Auto-launch OAuth so the new account is immediately usable.
      await launchOAuth(name);
    } catch (e) {
      nameError = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function login(name: string) {
    busy = true;
    try {
      await launchOAuth(name);
    } finally {
      busy = false;
    }
  }

  async function remove(name: string) {
    if (!confirm(t("accounts.confirmDelete", { name }))) return;
    busy = true;
    try {
      await accountsStore.deleteAccount(name);
    } finally {
      busy = false;
    }
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleDateString();
    } catch {
      return iso;
    }
  }
</script>

<div class="p-6 space-y-6 max-w-3xl">
  <!-- Header -->
  <header class="space-y-1">
    <h2 class="text-lg font-semibold" style="color: var(--text-primary)">{t("accounts.title")}</h2>
    <p class="text-xs" style="color: var(--text-muted)">{t("accounts.description")}</p>
  </header>

  <!-- Account list -->
  {#if accountsStore.accounts.length === 0}
    <div
      class="rounded-lg border border-dashed py-10 text-center"
      style="border-color: var(--border-color)"
    >
      <p class="text-sm" style="color: var(--text-muted)">{t("accounts.empty")}</p>
    </div>
  {:else}
    <ul class="space-y-2.5">
      {#each accountsStore.accounts as account (account.name)}
        {@const status = accountsStore.statuses[account.name]}
        <li class="card flex items-center gap-4 py-3.5">
          <!-- Avatar -->
          <div
            class="h-10 w-10 rounded-full flex items-center justify-center font-mono text-base font-semibold flex-shrink-0 select-none"
            style="background-color: var(--bg-tertiary); color: var(--text-secondary)"
          >
            {account.name.charAt(0).toUpperCase()}
          </div>

          <!-- Name + status badge + meta -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <span class="font-medium text-sm truncate" style="color: var(--text-primary)">{account.name}</span>
              {#if status?.loggedIn}
                <span
                  class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium flex-shrink-0"
                  style="background-color: var(--status-success-bg); color: var(--status-success-text)"
                >
                  <span class="h-1.5 w-1.5 rounded-full" style="background-color: currentColor"></span>
                  {t("accounts.statusLoggedIn")}
                </span>
              {:else}
                <span
                  class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium flex-shrink-0"
                  style="background-color: var(--bg-tertiary); color: var(--text-muted); border: 1px solid var(--border-color)"
                >
                  <span class="h-1.5 w-1.5 rounded-full" style="background-color: currentColor; opacity: 0.5"></span>
                  {t("accounts.notLoggedIn")}
                </span>
              {/if}
            </div>
            {#if status?.loggedIn && status.email}
              <div class="text-xs truncate" style="color: var(--text-secondary)">{status.email}</div>
            {/if}
            <div class="text-[11px] mt-0.5" style="color: var(--text-muted)">
              {t("accounts.created", { date: formatDate(account.createdAt) })}
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2 flex-shrink-0">
            <button
              class="btn-secondary rounded-md px-3 py-1.5 text-xs font-medium disabled:opacity-50"
              onclick={() => login(account.name)}
              disabled={busy}
            >
              {status?.loggedIn ? t("accounts.relogin") : t("accounts.login")}
            </button>
            <button
              class="btn-danger-ghost text-xs"
              onclick={() => remove(account.name)}
              disabled={busy}
            >
              {t("accounts.delete")}
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Add account form -->
  <div class="pt-5 space-y-3" style="border-top: 1px solid var(--border-color)">
    <div>
      <h3 class="text-sm font-medium" style="color: var(--text-primary)">{t("accounts.add")}</h3>
      <p class="mt-0.5 text-xs" style="color: var(--text-muted)">{t("accounts.addHint")}</p>
    </div>
    <div class="flex items-start gap-2">
      <div class="flex-1 space-y-1">
        <input
          type="text"
          placeholder={t("accounts.namePlaceholder")}
          class="input-base font-mono"
          bind:value={newName}
          onkeydown={(e) => { if (e.key === "Enter") add(); }}
          disabled={busy}
        />
        {#if nameError}
          <p class="text-xs" style="color: var(--status-error-text)">{nameError}</p>
        {/if}
      </div>
      <button
        class="btn-primary flex-shrink-0 rounded-md px-4 py-1.5 text-sm font-medium disabled:opacity-50"
        onclick={add}
        disabled={busy || !newName.trim() || !!nameError}
      >
        {t("accounts.add")}
      </button>
    </div>
  </div>
</div>
