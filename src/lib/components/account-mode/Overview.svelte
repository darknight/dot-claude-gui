<script lang="ts">
  import { ipcClient } from "$lib/ipc/client";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t } from "$lib/i18n";
  import type { AccountOverview } from "$lib/api/types";

  let { accountName } = $props<{ accountName: string }>();

  let overview = $state<AccountOverview | null>(null);
  let loading = $state(false);
  let err = $state<string | null>(null);

  async function load() {
    loading = true;
    err = null;
    try {
      overview = await ipcClient.accountOverview(accountName);
    } catch (e) {
      err = String(e);
      overview = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    // Re-fetch whenever accountName changes.
    void accountName;
    void load();
  });

  function relogin() {
    // Stage 3 Project > Launch will absorb the relogin flow.
    toastStore.info(t("accountMode.reloginHint"));
  }

  async function copyDir() {
    if (!overview) return;
    try {
      await navigator.clipboard.writeText(overview.configDir);
      toastStore.info(t("accountMode.dirCopied"));
    } catch (e) {
      toastStore.error(t("accountMode.openDirFailed"));
      console.error("clipboard write failed", e);
    }
  }

  async function deleteAcct() {
    if (!overview || overview.isNative) return;
    if (!confirm(t("accountMode.deleteConfirm", { name: overview.name }))) return;
    try {
      await accountsStore.deleteAccount(overview.name);
      // Clear selection so the AccountModeView $effect doesn't try to
      // setActiveAccount on the just-deleted name. The default-selection
      // $effect picks the next account automatically.
      modeStore.setSelectedAccount(null);
      toastStore.info(t("accountMode.deleteSuccess"));
    } catch (e) {
      toastStore.error(t("accountMode.deleteFailed"));
      console.error("deleteAccount failed", e);
    }
  }
</script>

<div class="flex-1 overflow-auto p-6">
  {#if loading}
    <p class="text-sm" style="color: var(--text-muted)">{t("accountMode.loading")}</p>
  {:else if err}
    <p class="text-sm" style="color: var(--text-error, #dc2626)">{err}</p>
  {:else if overview}
    <div class="rounded-lg p-4 mb-4" style="background-color: var(--bg-card); border: 1px solid var(--border-color)">
      <h2 class="text-lg font-semibold mb-3" style="color: var(--text-primary)">
        {overview.displayName}
        {#if overview.isNative}
          <span class="text-xs ml-2" style="color: var(--text-muted)">{t("shell.native")}</span>
        {/if}
      </h2>
      <dl class="grid grid-cols-2 gap-y-2 text-sm">
        <dt style="color: var(--text-muted)">{t("accountMode.configDir")}</dt>
        <dd style="color: var(--text-primary)" class="font-mono text-xs break-all">{overview.configDir}</dd>
        <dt style="color: var(--text-muted)">{t("accountMode.status")}</dt>
        <dd style="color: var(--text-primary)">
          {#if overview.loggedIn}
            ✓ {overview.email ?? t("accountMode.loggedIn")}
          {:else}
            {t("accountMode.notLoggedIn")}
          {/if}
        </dd>
        <dt style="color: var(--text-muted)">{t("accountMode.projectCount")}</dt>
        <dd style="color: var(--text-primary)">{overview.projectCount}</dd>
        <dt style="color: var(--text-muted)">{t("accountMode.pluginCount")}</dt>
        <dd style="color: var(--text-primary)">{overview.pluginCount}</dd>
        <dt style="color: var(--text-muted)">{t("accountMode.skillCount")}</dt>
        <dd style="color: var(--text-primary)">{overview.skillCount}</dd>
      </dl>
    </div>

    <div class="flex gap-2">
      <button
        type="button"
        class="px-3 py-1.5 text-sm rounded transition-colors hover:bg-[var(--bg-card-hover)]"
        style="background-color: var(--bg-card); border: 1px solid var(--border-color); color: var(--text-primary)"
        onclick={relogin}
      >
        {t("accountMode.relogin")}
      </button>
      <button
        type="button"
        class="px-3 py-1.5 text-sm rounded transition-colors hover:bg-[var(--bg-card-hover)]"
        style="background-color: var(--bg-card); border: 1px solid var(--border-color); color: var(--text-primary)"
        onclick={copyDir}
      >
        {t("accountMode.copyDir")}
      </button>
      {#if !overview.isNative}
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded transition-colors"
          style="background-color: var(--bg-card); border: 1px solid var(--border-color); color: var(--text-error, #dc2626)"
          onclick={deleteAcct}
        >
          {t("accountMode.delete")}
        </button>
      {/if}
    </div>
  {/if}
</div>
