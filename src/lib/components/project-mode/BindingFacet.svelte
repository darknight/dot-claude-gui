<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "$lib/i18n";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";

  let { path }: { path: string } = $props();

  const binding = $derived(projectsStore.entries.find((e) => e.path === path));
  let selectedAccount = $state<string>("");

  $effect(() => {
    selectedAccount = binding?.account ?? "";
  });

  onMount(async () => {
    if (accountsStore.accounts.length === 0) {
      await accountsStore.loadAccounts();
    }
  });

  async function onBind() {
    if (!selectedAccount) return;
    try {
      await projectsStore.bind(path, selectedAccount);
      toastStore.success(t("projectMode.binding.bound", { account: selectedAccount }));
    } catch (e) {
      toastStore.error(String(e));
    }
  }

  async function onUnbind() {
    if (!confirm(t("projectMode.binding.confirmUnbind"))) return;
    try {
      await projectsStore.unbind(path);
    } catch (e) {
      toastStore.error(String(e));
    }
  }

  async function onRemove() {
    if (!confirm(t("projectMode.binding.confirmRemove"))) return;
    try {
      await projectsStore.remove(path);
    } catch (e) {
      toastStore.error(String(e));
    }
  }

  async function onOpenTerminal() {
    try {
      await ipcClient.launchClaude({
        projectPath: path,
        account: binding?.account ?? "default",
      });
    } catch (e) {
      toastStore.error(String(e));
    }
  }
</script>

<section class="binding-facet">
  <h2>{t("projectMode.binding.title")}</h2>

  {#if !binding?.account}
    <div class="unbound-banner" role="note">
      {t("projectMode.binding.unboundHint")}
    </div>
  {/if}

  <dl>
    <dt>{t("projectMode.binding.pathLabel")}</dt>
    <dd><code>{path}</code></dd>

    <dt>{t("projectMode.binding.accountLabel")}</dt>
    <dd class="account-row">
      <select bind:value={selectedAccount}>
        <option value="">{t("projectMode.binding.selectAccount")}</option>
        {#each accountsStore.accounts as a (a.name)}
          <option value={a.name}>{a.displayName} ({a.name})</option>
        {/each}
      </select>
      <button
        onclick={onBind}
        disabled={!selectedAccount || selectedAccount === (binding?.account ?? "")}
      >{t("projectMode.binding.bindBtn")}</button>
    </dd>
  </dl>

  <div class="actions">
    <button onclick={onOpenTerminal} disabled={!binding?.account}>
      {t("projectMode.binding.openTerminal")}
    </button>
    <button onclick={onUnbind} disabled={!binding?.account}>
      {t("projectMode.binding.unbindBtn")}
    </button>
    <button onclick={onRemove} class="danger">
      {t("projectMode.binding.removeBtn")}
    </button>
  </div>
</section>

<style>
  .binding-facet {
    padding: 16px;
    color: var(--text-primary);
  }
  h2 {
    margin: 0 0 16px 0;
    font-size: 16px;
    font-weight: 600;
  }
  .unbound-banner {
    margin: 0 0 16px 0;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent-primary);
    border-radius: 4px;
    background: var(--bg-secondary, transparent);
    color: var(--text-muted);
    font-size: 13px;
    line-height: 1.5;
  }
  dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 8px 16px;
    margin: 0 0 24px 0;
  }
  dt {
    color: var(--text-muted);
    font-size: 13px;
    align-self: center;
  }
  dd {
    margin: 0;
  }
  code {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    background: var(--bg-input, transparent);
    padding: 2px 6px;
    border-radius: 3px;
    word-break: break-all;
  }
  .account-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  select {
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: inherit;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  button {
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button:hover:not([disabled]) {
    background: var(--bg-hover, rgba(0,0,0,0.05));
  }
  button[disabled] {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button.danger {
    color: var(--danger, #c44);
    border-color: var(--danger, #c44);
  }
</style>
