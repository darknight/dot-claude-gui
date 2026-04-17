<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";

  const settings = $derived(configStore.activeSettings);

  let model = $state((settings.model as string) ?? "");
  let outputStyle = $state((settings.outputStyle as string) ?? "");
  let fastMode = $state((settings.fastMode as boolean) ?? false);
  let fastModePerSessionOptIn = $state(
    (settings.fastModePerSessionOptIn as boolean) ?? false,
  );
  let availableModelsText = $state(
    ((settings.availableModels as string[]) ?? []).join("\n"),
  );
  let autoCompactWindow = $state<number | string>(
    (settings.autoCompactWindow as number | undefined) ?? "",
  );
  let showClearContextOnPlanAccept = $state(
    (settings.showClearContextOnPlanAccept as boolean) ?? false,
  );
  let promptSuggestionEnabled = $state(
    (settings.promptSuggestionEnabled as boolean) ?? false,
  );

  $effect(() => {
    model = (settings.model as string) ?? "";
    outputStyle = (settings.outputStyle as string) ?? "";
    fastMode = (settings.fastMode as boolean) ?? false;
    fastModePerSessionOptIn = (settings.fastModePerSessionOptIn as boolean) ?? false;
    availableModelsText = ((settings.availableModels as string[]) ?? []).join("\n");
    autoCompactWindow = (settings.autoCompactWindow as number | undefined) ?? "";
    showClearContextOnPlanAccept =
      (settings.showClearContextOnPlanAccept as boolean) ?? false;
    promptSuggestionEnabled =
      (settings.promptSuggestionEnabled as boolean) ?? false;
  });

  const modelDirty = $derived(model !== ((settings.model as string) ?? ""));
  const outputStyleDirty = $derived(
    outputStyle !== ((settings.outputStyle as string) ?? ""),
  );
  const fastModeDirty = $derived(
    fastMode !== ((settings.fastMode as boolean) ?? false),
  );
  const fastModePerSessionDirty = $derived(
    fastModePerSessionOptIn !==
      ((settings.fastModePerSessionOptIn as boolean) ?? false),
  );
  const availableModelsDirty = $derived(
    availableModelsText !==
      ((settings.availableModels as string[]) ?? []).join("\n"),
  );
  const autoCompactWindowDirty = $derived(
    String(autoCompactWindow) !==
      String((settings.autoCompactWindow as number | undefined) ?? ""),
  );
  const showClearCtxDirty = $derived(
    showClearContextOnPlanAccept !==
      ((settings.showClearContextOnPlanAccept as boolean) ?? false),
  );
  const promptSuggestionDirty = $derived(
    promptSuggestionEnabled !==
      ((settings.promptSuggestionEnabled as boolean) ?? false),
  );

  function parseAvailableModels(): string[] | undefined {
    const lines = availableModelsText
      .split("\n")
      .map((s) => s.trim())
      .filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  function parsedWindow(): number | undefined {
    const n = Number(autoCompactWindow);
    return Number.isFinite(n) && n > 0 ? n : undefined;
  }

  const previewData = $derived({
    model: model || undefined,
    outputStyle: outputStyle || undefined,
    fastMode,
    fastModePerSessionOptIn,
    availableModels: parseAvailableModels(),
    autoCompactWindow: parsedWindow(),
    showClearContextOnPlanAccept,
    promptSuggestionEnabled,
  });

  function save() {
    configStore.save({
      model: model || undefined,
      outputStyle: outputStyle || undefined,
      fastMode,
      fastModePerSessionOptIn,
      availableModels: parseAvailableModels(),
      autoCompactWindow: parsedWindow(),
      showClearContextOnPlanAccept,
      promptSuggestionEnabled,
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <div class="space-y-1">
    <label for="model" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.model.tooltip")}>
      {t("settings.fields.model.label")}
      <DirtyDot dirty={modelDirty} />
    </label>
    <input id="model" type="text" bind:value={model}
           oninput={() => configStore.markDirty()}
           placeholder="opus"
           class="input-base" />
  </div>

  <div class="space-y-1">
    <label for="outputStyle" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.outputStyle.tooltip")}>
      {t("settings.fields.outputStyle.label")}
      <DirtyDot dirty={outputStyleDirty} />
    </label>
    <input id="outputStyle" type="text" bind:value={outputStyle}
           oninput={() => configStore.markDirty()}
           class="input-base" />
  </div>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={fastMode}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.fastMode.tooltip")}>
      {t("settings.fields.fastMode.label")}
      <DirtyDot dirty={fastModeDirty} />
    </span>
  </label>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={fastModePerSessionOptIn}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.fastModePerSessionOptIn.tooltip")}>
      {t("settings.fields.fastModePerSessionOptIn.label")}
      <DirtyDot dirty={fastModePerSessionDirty} />
    </span>
  </label>

  <div class="space-y-1">
    <label for="availableModels" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.availableModels.tooltip")}>
      {t("settings.fields.availableModels.label")}
      <DirtyDot dirty={availableModelsDirty} />
    </label>
    <textarea id="availableModels" rows="4"
              bind:value={availableModelsText}
              oninput={() => configStore.markDirty()}
              placeholder={"opus\nsonnet\nhaiku"}
              class="input-base font-mono text-xs"></textarea>
    <p class="text-xs" style="color: var(--text-muted)">one model id per line</p>
  </div>

  <div class="space-y-1">
    <label for="autoCompactWindow" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.autoCompactWindow.tooltip")}>
      {t("settings.fields.autoCompactWindow.label")}
      <DirtyDot dirty={autoCompactWindowDirty} />
    </label>
    <input id="autoCompactWindow" type="number"
           bind:value={autoCompactWindow}
           oninput={() => configStore.markDirty()}
           min="100000" max="1000000"
           placeholder="200000"
           class="input-base" />
  </div>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={showClearContextOnPlanAccept}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.showClearContextOnPlanAccept.tooltip")}>
      {t("settings.fields.showClearContextOnPlanAccept.label")}
      <DirtyDot dirty={showClearCtxDirty} />
    </span>
  </label>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={promptSuggestionEnabled}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.promptSuggestionEnabled.tooltip")}>
      {t("settings.fields.promptSuggestionEnabled.label")}
      <DirtyDot dirty={promptSuggestionDirty} />
    </span>
  </label>

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

  <JsonPreview data={previewData} title={t("settings.runtime") + " JSON"} />
</div>
