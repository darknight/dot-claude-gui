<script lang="ts">
  import { t } from "$lib/i18n";
  import { ipcClient } from "$lib/ipc/client";
  import type { ProjectEffectiveResponse } from "$lib/api/types";

  let { path }: { path: string } = $props();

  let resp = $state<ProjectEffectiveResponse | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function load() {
    loading = true;
    error = null;
    try {
      resp = await ipcClient.projectReadEffective(path);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => { void path; load(); });

  function sourceLabel(field: string): string {
    return resp?.fieldSources?.[field] ?? "user";
  }
</script>

<section class="effective-facet">
  <header>
    <h2>{t("projectMode.effective.title")}</h2>
    <button type="button" onclick={load} disabled={loading}>{t("projectMode.effective.refreshBtn")}</button>
  </header>

  {#if loading}
    <div class="empty">{t("projectMode.effective.loading")}</div>
  {:else if error}
    <div class="err">{error}</div>
  {:else if resp}
    <p class="hint">{t("projectMode.effective.account", { account: resp.account })}</p>

    <details open>
      <summary class="section-title">{t("projectMode.effective.mergedTitle")}</summary>
      <pre class="json">{JSON.stringify(resp.settings, null, 2)}</pre>
    </details>

    <details>
      <summary class="section-title">{t("projectMode.effective.sourcesTitle")}</summary>
      {#if Object.keys(resp.fieldSources ?? {}).length === 0}
        <div class="empty">{t("projectMode.effective.noOverrides")}</div>
      {:else}
        <table>
          <thead>
            <tr>
              <th>{t("projectMode.effective.field")}</th>
              <th>{t("projectMode.effective.source")}</th>
            </tr>
          </thead>
          <tbody>
            {#each Object.keys(resp.fieldSources).sort() as f (f)}
              {@const src = sourceLabel(f)}
              <tr>
                <td><code>{f}</code></td>
                <td><span class="badge {src}">{src}</span></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </details>
  {/if}
</section>

<style>
  .effective-facet {
    padding: 16px;
    color: var(--text-primary);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }
  header button {
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  header button[disabled] { opacity: 0.5; cursor: not-allowed; }
  .hint {
    color: var(--text-muted);
    font-size: 12px;
    margin: 0 0 16px;
  }
  details {
    margin-bottom: 16px;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0 12px;
  }
  details[open] {
    padding-bottom: 12px;
  }
  .section-title {
    cursor: pointer;
    padding: 8px 0;
    font-weight: 600;
    font-size: 13px;
    user-select: none;
  }
  .json {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    background: var(--bg-input, transparent);
    border: 1px solid var(--border);
    padding: 12px;
    border-radius: 4px;
    max-height: 400px;
    overflow: auto;
    margin: 0;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 4px;
  }
  th, td {
    text-align: left;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }
  th { color: var(--text-muted); font-weight: 600; }
  code {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 11px;
  }
  .badge {
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 11px;
    text-transform: capitalize;
  }
  .badge.user { background: #dbeafe; color: #1e3a8a; }
  .badge.project { background: #fef3c7; color: #92400e; }
  .badge.local { background: #fee2e2; color: #991b1b; }
  .badge.managed { background: #ede9fe; color: #5b21b6; }
  .badge.default { background: #f3f4f6; color: #6b7280; }
  .err {
    color: var(--danger, #c44);
    padding: 16px;
  }
  .empty {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
    font-size: 12px;
  }
</style>
