<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";
  import type { MarketplaceSource } from "$lib/api/types";

  const settings = $derived(configStore.activeSettings);

  let strictKnown = $state<MarketplaceSource[]>(
    (settings.strictKnownMarketplaces as MarketplaceSource[]) ?? [],
  );
  let blocked = $state<MarketplaceSource[]>(
    (settings.blockedMarketplaces as MarketplaceSource[]) ?? [],
  );
  let skippedMarketsText = $state(
    ((settings.skippedMarketplaces as string[]) ?? []).join("\n"),
  );
  let skippedPluginsText = $state(
    ((settings.skippedPlugins as string[]) ?? []).join("\n"),
  );
  let trustMessage = $state((settings.pluginTrustMessage as string) ?? "");
  let pluginConfigsText = $state(
    JSON.stringify((settings.pluginConfigs as object) ?? {}, null, 2),
  );
  let skillOverridesText = $state(
    JSON.stringify((settings.skillOverrides as object) ?? {}, null, 2),
  );
  let pluginConfigsError = $state("");
  let skillOverridesError = $state("");

  let newSrc = $state("");
  let newRepo = $state("");
  let targetList = $state<"strict" | "blocked">("strict");

  $effect(() => {
    strictKnown = (settings.strictKnownMarketplaces as MarketplaceSource[]) ?? [];
    blocked = (settings.blockedMarketplaces as MarketplaceSource[]) ?? [];
    skippedMarketsText = ((settings.skippedMarketplaces as string[]) ?? []).join("\n");
    skippedPluginsText = ((settings.skippedPlugins as string[]) ?? []).join("\n");
    trustMessage = (settings.pluginTrustMessage as string) ?? "";
    pluginConfigsText = JSON.stringify((settings.pluginConfigs as object) ?? {}, null, 2);
    skillOverridesText = JSON.stringify(
      (settings.skillOverrides as object) ?? {},
      null,
      2,
    );
  });

  function addMp() {
    if (!newSrc.trim() || !newRepo.trim()) return;
    const entry: MarketplaceSource = {
      source: { source: newSrc.trim() } as { source: string; [key: string]: unknown },
      repo: newRepo.trim(),
    } as unknown as MarketplaceSource;
    if (targetList === "strict") strictKnown = [...strictKnown, entry];
    else blocked = [...blocked, entry];
    newSrc = "";
    newRepo = "";
    configStore.markDirty();
  }

  function removeMp(target: "strict" | "blocked", i: number) {
    if (target === "strict") strictKnown = strictKnown.filter((_, idx) => idx !== i);
    else blocked = blocked.filter((_, idx) => idx !== i);
    configStore.markDirty();
  }

  function parseLines(text: string): string[] | undefined {
    const lines = text.split("\n").map((s) => s.trim()).filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  function parseJson(text: string, setError: (e: string) => void): object | undefined {
    const trimmed = text.trim();
    if (!trimmed || trimmed === "{}") {
      setError("");
      return undefined;
    }
    try {
      const parsed = JSON.parse(trimmed);
      setError("");
      return parsed;
    } catch (e) {
      setError(t("settings.pluginsMarketplace.invalidJson") + ": " + String(e));
      return undefined;
    }
  }

  function save() {
    const pc = parseJson(pluginConfigsText, (e) => (pluginConfigsError = e));
    const so = parseJson(skillOverridesText, (e) => (skillOverridesError = e));
    if (pluginConfigsError || skillOverridesError) return;

    configStore.save({
      strictKnownMarketplaces: strictKnown.length === 0 ? undefined : strictKnown,
      blockedMarketplaces: blocked.length === 0 ? undefined : blocked,
      skippedMarketplaces: parseLines(skippedMarketsText),
      skippedPlugins: parseLines(skippedPluginsText),
      pluginTrustMessage: trustMessage || undefined,
      pluginConfigs: pc as Record<string, unknown> | undefined,
      skillOverrides: so as Record<string, unknown> | undefined,
    });
  }

  function mpLabel(mp: MarketplaceSource): string {
    const srcInfo = mp.source as { source?: string } | undefined;
    const repo = (mp as unknown as { repo?: string }).repo;
    return `${srcInfo?.source ?? "?"} / ${repo ?? "?"}`;
  }
</script>

<div class="space-y-6 max-w-2xl">
  <section>
    <div class="flex items-center gap-2 mb-2">
      <select bind:value={targetList} class="input-base w-40 text-sm">
        <option value="strict">{t("settings.pluginsMarketplace.strictKnown")}</option>
        <option value="blocked">{t("settings.pluginsMarketplace.blocked")}</option>
      </select>
      <input type="text" bind:value={newSrc}
             placeholder={t("settings.pluginsMarketplace.mpSourcePlaceholder")}
             class="input-base flex-1" />
      <input type="text" bind:value={newRepo}
             placeholder={t("settings.pluginsMarketplace.mpRepoPlaceholder")}
             class="input-base flex-1" />
      <button type="button" onclick={addMp}
              class="btn-secondary text-xs px-3 py-1">
        {t("settings.pluginsMarketplace.addMarketplace")}
      </button>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <h4 class="text-xs font-semibold mb-1" style="color: var(--text-muted)">
          {t("settings.pluginsMarketplace.strictKnown")}
        </h4>
        <ul class="space-y-1 text-sm">
          {#each strictKnown as mp, i (i + ":strict")}
            <li class="flex items-center gap-2">
              <span style="color: var(--text-primary)">{mpLabel(mp)}</span>
              <button type="button" onclick={() => removeMp("strict", i)}
                      class="text-xs px-2 py-0.5 rounded"
                      style="color: var(--status-error-text); background-color: var(--status-error-bg)">
                ✕
              </button>
            </li>
          {/each}
        </ul>
      </div>
      <div>
        <h4 class="text-xs font-semibold mb-1" style="color: var(--text-muted)">
          {t("settings.pluginsMarketplace.blocked")}
        </h4>
        <ul class="space-y-1 text-sm">
          {#each blocked as mp, i (i + ":blocked")}
            <li class="flex items-center gap-2">
              <span style="color: var(--text-primary)">{mpLabel(mp)}</span>
              <button type="button" onclick={() => removeMp("blocked", i)}
                      class="text-xs px-2 py-0.5 rounded"
                      style="color: var(--status-error-text); background-color: var(--status-error-bg)">
                ✕
              </button>
            </li>
          {/each}
        </ul>
      </div>
    </div>
  </section>

  <section class="grid grid-cols-2 gap-4">
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.pluginsMarketplace.skippedMarkets")}
      </label>
      <textarea bind:value={skippedMarketsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.pluginsMarketplace.addLinePlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.pluginsMarketplace.skippedPlugins")}
      </label>
      <textarea bind:value={skippedPluginsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.pluginsMarketplace.addLinePlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
  </section>

  <section>
    <label class="text-sm font-semibold" style="color: var(--text-primary)"
           title={t("settings.fields.pluginTrustMessage.tooltip")}>
      {t("settings.fields.pluginTrustMessage.label")}
    </label>
    <textarea bind:value={trustMessage} rows="3"
              oninput={() => configStore.markDirty()}
              class="input-base text-sm mt-1"></textarea>
  </section>

  <section>
    <label class="text-sm font-semibold" style="color: var(--text-primary)">
      {t("settings.pluginsMarketplace.pluginConfigs")}
    </label>
    <textarea bind:value={pluginConfigsText} rows="6"
              oninput={() => configStore.markDirty()}
              class="input-base font-mono text-xs mt-1"></textarea>
    {#if pluginConfigsError}
      <p class="text-xs" style="color: var(--status-error-text)">{pluginConfigsError}</p>
    {/if}
  </section>

  <section>
    <label class="text-sm font-semibold" style="color: var(--text-primary)">
      {t("settings.pluginsMarketplace.skillOverrides")}
    </label>
    <textarea bind:value={skillOverridesText} rows="4"
              oninput={() => configStore.markDirty()}
              class="input-base font-mono text-xs mt-1"></textarea>
    {#if skillOverridesError}
      <p class="text-xs" style="color: var(--status-error-text)">{skillOverridesError}</p>
    {/if}
  </section>

  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button type="button" onclick={save}
            disabled={!configStore.isDirty || configStore.saving}
            class="btn-primary text-sm px-4 py-2">
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button type="button" onclick={() => configStore.revert()}
            disabled={!configStore.isDirty}
            class="btn-secondary text-sm px-4 py-2">
      {t("common.revert")}
    </button>
  </div>

  <JsonPreview data={{
    strictKnownMarketplaces: strictKnown.length === 0 ? undefined : strictKnown,
    blockedMarketplaces: blocked.length === 0 ? undefined : blocked,
    pluginTrustMessage: trustMessage || undefined,
  }} title="Plugins & Marketplace JSON" />
</div>
