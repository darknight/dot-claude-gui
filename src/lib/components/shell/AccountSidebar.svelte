<script lang="ts">
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t } from "$lib/i18n";

  let addingName = $state("");
  let creating = $state(false);

  function selectAccount(name: string) {
    if (modeStore.selectedAccount === name) return;
    // AccountModeView's $effect on modeStore.selectedAccount owns the
    // setActiveAccount IPC + all store reloads. Don't call setActiveAccount
    // here too — it'd produce two IPCs per click.
    modeStore.setSelectedAccount(name);
  }

  async function addAccount(e: SubmitEvent) {
    e.preventDefault();
    const name = addingName.trim();
    if (!name || creating) return;
    if (accountsStore.has(name)) {
      toastStore.error(t("shell.accountAlreadyExists"));
      return;
    }
    creating = true;
    try {
      await accountsStore.createAccount(name);
      addingName = "";
      selectAccount(name);
    } catch (e) {
      toastStore.error(t("shell.createAccountFailed"));
      console.error("createAccount failed", e);
    } finally {
      creating = false;
    }
  }
</script>

<div class="flex flex-col h-full" style="background-color: var(--bg-secondary)">
  <div class="px-4 py-3" style="border-bottom: 1px solid var(--border-color)">
    <h2 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
      {t("shell.accountsList")}
    </h2>
  </div>

  <ul class="flex-1 overflow-y-auto py-2">
    {#each accountsStore.accounts as account (account.name)}
      {@const isActive = modeStore.selectedAccount === account.name}
      {@const status = accountsStore.statuses[account.name]}
      <li>
        <button
          class="w-full px-4 py-2 text-left text-sm flex items-center gap-2 transition-colors {isActive ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
          style="background-color: {isActive ? 'var(--accent-bg)' : 'transparent'}; color: {isActive ? 'var(--accent-text)' : 'var(--text-primary)'}"
          onclick={() => selectAccount(account.name)}
        >
          <span class="flex-1 truncate">
            {account.displayName}
            {#if account.isNative}
              <span class="text-xs" style="color: var(--text-muted)">· {t("shell.native")}</span>
            {/if}
          </span>
          {#if status?.loggedIn}
            <span class="w-2 h-2 rounded-full" style="background-color: var(--accent-text)" title={status.email ?? ""}></span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>

  <form class="px-3 py-3 flex gap-2" style="border-top: 1px solid var(--border-color)" onsubmit={addAccount}>
    <input
      class="min-w-0 flex-1 px-2 py-1 text-sm rounded border"
      style="background-color: var(--bg-primary); border-color: var(--border-color); color: var(--text-primary)"
      bind:value={addingName}
      placeholder={t("shell.newAccountPlaceholder")}
      disabled={creating}
    />
    <button
      type="submit"
      class="shrink-0 px-3 py-1 text-sm rounded transition-colors"
      style="background-color: var(--accent-bg); color: var(--accent-text)"
      disabled={creating || !addingName.trim()}
    >
      {t("shell.addAccount")}
    </button>
  </form>
</div>
