<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "$lib/i18n";
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { appSettingsStore } from "$lib/stores/appsettings.svelte.js";
  import { ipcClient } from "$lib/ipc/client";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { parseClaudeHelp, type ClaudeArg } from "$lib/data/parseClaudeHelp";

  let { path }: { path: string } = $props();

  const binding = $derived(projectsStore.entries.find((e) => e.path === path));
  const account = $derived(binding?.account ?? "");
  const isBound = $derived(account !== "");

  // ── Binding section state ──────────────────────────────────────────
  let selectedAccount = $state<string>("");
  $effect(() => {
    selectedAccount = binding?.account ?? "";
  });

  // ── Launch section state (only meaningful while bound) ─────────────
  let envEntries = $state<Array<{ k: string; v: string }>>([]);
  let argEntries = $state<string[]>([]);
  let argSuggestions = $state<ClaudeArg[]>([]);
  let dirty = $state(false);
  let saving = $state(false);

  $effect(() => {
    const l = binding?.launch;
    envEntries = Object.entries(l?.env ?? {}).map(([k, v]) => ({ k, v }));
    argEntries = [...(l?.args ?? [])];
    dirty = false;
  });

  onMount(async () => {
    if (accountsStore.accounts.length === 0) {
      await accountsStore.loadAccounts();
    }
    try {
      const raw = await ipcClient.getClaudeArgs();
      argSuggestions = parseClaudeHelp(raw);
    } catch {
      argSuggestions = [];
    }
  });

  // ── Binding actions ────────────────────────────────────────────────
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

  // ── Launch section actions ─────────────────────────────────────────
  function addEnv() {
    envEntries = [...envEntries, { k: "", v: "" }];
    dirty = true;
  }
  function removeEnv(i: number) {
    envEntries = envEntries.filter((_, j) => j !== i);
    dirty = true;
  }
  function addArg() {
    argEntries = [...argEntries, ""];
    dirty = true;
  }
  function removeArg(i: number) {
    argEntries = argEntries.filter((_, j) => j !== i);
    dirty = true;
  }

  function pack(): { env: Record<string, string>; args: string[] } {
    const env: Record<string, string> = {};
    for (const { k, v } of envEntries) if (k) env[k] = v;
    return { env, args: argEntries.filter((a) => a.length > 0) };
  }

  async function save() {
    saving = true;
    try {
      await projectsStore.updateLaunch(path, pack());
      dirty = false;
      toastStore.success(t("projectMode.launch.saved"));
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      saving = false;
    }
  }

  async function launch() {
    if (dirty) await save();
    const packed = pack();
    try {
      await ipcClient.launchClaude({
        projectPath: path,
        env: packed.env,
        args: packed.args,
        account,
        preferredTerminal: appSettingsStore.preferences.preferredTerminal,
      });
    } catch (e) {
      toastStore.error(String(e));
    }
  }
</script>

<section class="launch-facet">
  <!-- ── Binding section ─────────────────────────────────────────── -->
  <h2>{t("projectMode.binding.title")}</h2>

  {#if !isBound}
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
        disabled={!selectedAccount || selectedAccount === account}
      >{t("projectMode.binding.bindBtn")}</button>
    </dd>
  </dl>

  <!-- ── Launch configuration section (collapsed when unbound) ───── -->
  {#if isBound}
    <h3 class="section-h3">{t("projectMode.launch.title")}</h3>
    <p class="hint">{t("projectMode.launch.account", { account })}</p>

    <h4>{t("projectMode.launch.envTitle")}</h4>
    <table class="env-table">
      <tbody>
        {#each envEntries as e, i (i)}
          <tr>
            <td><input bind:value={e.k} placeholder="KEY" oninput={() => (dirty = true)} /></td>
            <td><input bind:value={e.v} placeholder="value" oninput={() => (dirty = true)} /></td>
            <td>
              <button class="remove" onclick={() => removeEnv(i)} aria-label="remove">×</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    <button class="add" onclick={addEnv}>{t("projectMode.launch.addEnv")}</button>

    <h4>{t("projectMode.launch.argsTitle")}</h4>
    <datalist id="claude-arg-suggestions">
      {#each argSuggestions as s (s.flag)}
        <option value={s.flag}>{s.description}</option>
      {/each}
    </datalist>
    <ul class="args">
      {#each argEntries as a, i (i)}
        <li>
          <input bind:value={argEntries[i]} list="claude-arg-suggestions" oninput={() => (dirty = true)} />
          <button class="remove" onclick={() => removeArg(i)} aria-label="remove">×</button>
        </li>
      {/each}
    </ul>
    <button class="add" onclick={addArg}>{t("projectMode.launch.addArg")}</button>

    <div class="actions">
      <button onclick={save} disabled={!dirty || saving}>
        {dirty ? t("projectMode.launch.saveBtn") : t("projectMode.launch.savedLabel")}
      </button>
      <button onclick={launch} class="primary">
        {t("projectMode.launch.launchBtn")}
      </button>
    </div>
  {/if}

  <!-- ── Danger zone ─────────────────────────────────────────────── -->
  <h3 class="section-h3 danger-section">{t("projectMode.dangerZone.title")}</h3>
  <div class="actions">
    <button onclick={onUnbind} disabled={!isBound}>
      {t("projectMode.binding.unbindBtn")}
    </button>
    <button onclick={onRemove} class="danger">
      {t("projectMode.binding.removeBtn")}
    </button>
  </div>
</section>

<style>
  .launch-facet {
    padding: 16px;
    color: var(--text-primary);
  }
  h2 {
    margin: 0 0 16px;
    font-size: 16px;
    font-weight: 600;
  }
  .section-h3 {
    margin: 28px 0 8px;
    font-size: 14px;
    font-weight: 600;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border);
  }
  .danger-section {
    color: var(--danger, #c44);
    border-bottom-color: var(--danger, #c44);
  }
  h4 {
    margin: 16px 0 8px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted);
  }
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin: 0 0 16px;
  }
  .unbound-banner {
    margin: 0 0 16px;
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
    margin: 0 0 8px;
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
  table.env-table {
    width: 100%;
    border-collapse: collapse;
  }
  table.env-table td {
    padding: 2px 4px 2px 0;
  }
  table.env-table input {
    width: 100%;
  }
  ul.args {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  ul.args li {
    display: flex;
    gap: 8px;
    margin: 4px 0;
  }
  ul.args li input {
    flex: 1;
  }
  input {
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input, transparent);
    color: inherit;
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
  button.add {
    margin-top: 4px;
    font-size: 12px;
  }
  button.remove {
    padding: 2px 8px;
    line-height: 1;
  }
  .actions {
    margin-top: 16px;
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  button.primary {
    background: var(--accent, #2c6cff);
    border-color: var(--accent, #2c6cff);
    color: white;
  }
  button.danger {
    color: var(--danger, #c44);
    border-color: var(--danger, #c44);
  }
</style>
