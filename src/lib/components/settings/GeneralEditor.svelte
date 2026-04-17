<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";

  const settings = $derived(configStore.activeSettings);

  let language = $state(settings.language ?? "");
  let alwaysThinkingEnabled = $state(settings.alwaysThinkingEnabled ?? false);
  let autoUpdatesChannel = $state(settings.autoUpdatesChannel ?? "stable");
  let minimumVersion = $state(settings.minimumVersion ?? "");
  let includeCoAuthoredBy = $state(settings.includeCoAuthoredBy ?? false);
  let skipDangerousModePermissionPrompt = $state(
    settings.skipDangerousModePermissionPrompt ?? false,
  );
  let tui = $state((settings.tui as string | undefined) ?? "");
  let effortLevel = $state((settings.effortLevel as string | undefined) ?? "");
  let autoMemoryEnabled = $state((settings.autoMemoryEnabled as boolean) ?? false);
  let includeGitInstructions = $state(
    (settings.includeGitInstructions as boolean) ?? false,
  );
  let respectGitignore = $state((settings.respectGitignore as boolean) ?? false);
  let cleanupPeriodDays = $state<number | string>(
    (settings.cleanupPeriodDays as number | undefined) ?? "",
  );
  let claudeMdExcludesText = $state(
    ((settings.claudeMdExcludes as string[]) ?? []).join("\n"),
  );
  let plansDirectory = $state((settings.plansDirectory as string) ?? "");
  let syntaxHighlightingDisabled = $state(
    (settings.syntaxHighlightingDisabled as boolean) ?? false,
  );

  $effect(() => {
    language = settings.language ?? "";
    alwaysThinkingEnabled = settings.alwaysThinkingEnabled ?? false;
    autoUpdatesChannel = settings.autoUpdatesChannel ?? "stable";
    minimumVersion = settings.minimumVersion ?? "";
    includeCoAuthoredBy = settings.includeCoAuthoredBy ?? false;
    skipDangerousModePermissionPrompt =
      settings.skipDangerousModePermissionPrompt ?? false;
    tui = (settings.tui as string | undefined) ?? "";
    effortLevel = (settings.effortLevel as string | undefined) ?? "";
    autoMemoryEnabled = (settings.autoMemoryEnabled as boolean) ?? false;
    includeGitInstructions = (settings.includeGitInstructions as boolean) ?? false;
    respectGitignore = (settings.respectGitignore as boolean) ?? false;
    cleanupPeriodDays = (settings.cleanupPeriodDays as number | undefined) ?? "";
    claudeMdExcludesText = ((settings.claudeMdExcludes as string[]) ?? []).join("\n");
    plansDirectory = (settings.plansDirectory as string) ?? "";
    syntaxHighlightingDisabled =
      (settings.syntaxHighlightingDisabled as boolean) ?? false;
  });

  const languageDirty = $derived(language !== (settings.language ?? ""));
  const thinkingDirty = $derived(
    alwaysThinkingEnabled !== (settings.alwaysThinkingEnabled ?? false),
  );
  const channelDirty = $derived(
    autoUpdatesChannel !== (settings.autoUpdatesChannel ?? "stable"),
  );
  const versionDirty = $derived(
    minimumVersion !== (settings.minimumVersion ?? ""),
  );
  const coauthorDirty = $derived(
    includeCoAuthoredBy !== (settings.includeCoAuthoredBy ?? false),
  );
  const skipDangerousDirty = $derived(
    skipDangerousModePermissionPrompt !==
      (settings.skipDangerousModePermissionPrompt ?? false),
  );
  const tuiDirty = $derived(
    tui !== ((settings.tui as string | undefined) ?? ""),
  );
  const effortLevelDirty = $derived(
    effortLevel !== ((settings.effortLevel as string | undefined) ?? ""),
  );
  const autoMemoryDirty = $derived(
    autoMemoryEnabled !== ((settings.autoMemoryEnabled as boolean) ?? false),
  );
  const gitInstrDirty = $derived(
    includeGitInstructions !==
      ((settings.includeGitInstructions as boolean) ?? false),
  );
  const gitignoreDirty = $derived(
    respectGitignore !== ((settings.respectGitignore as boolean) ?? false),
  );
  const cleanupDirty = $derived(
    String(cleanupPeriodDays) !==
      String((settings.cleanupPeriodDays as number | undefined) ?? ""),
  );
  const excludesDirty = $derived(
    claudeMdExcludesText !==
      ((settings.claudeMdExcludes as string[]) ?? []).join("\n"),
  );
  const plansDirDirty = $derived(
    plansDirectory !== ((settings.plansDirectory as string) ?? ""),
  );
  const syntaxDisabledDirty = $derived(
    syntaxHighlightingDisabled !==
      ((settings.syntaxHighlightingDisabled as boolean) ?? false),
  );

  function parsedCleanup(): number | undefined {
    const n = Number(cleanupPeriodDays);
    return Number.isFinite(n) && n > 0 ? n : undefined;
  }
  function parsedExcludes(): string[] | undefined {
    const lines = claudeMdExcludesText
      .split("\n")
      .map((s) => s.trim())
      .filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  const previewData = $derived({
    language: language || undefined,
    alwaysThinkingEnabled,
    autoUpdatesChannel,
    minimumVersion: minimumVersion || undefined,
    includeCoAuthoredBy,
    skipDangerousModePermissionPrompt,
    tui: tui || undefined,
    effortLevel: effortLevel || undefined,
    autoMemoryEnabled,
    includeGitInstructions,
    respectGitignore,
    cleanupPeriodDays: parsedCleanup(),
    claudeMdExcludes: parsedExcludes(),
    plansDirectory: plansDirectory || undefined,
    syntaxHighlightingDisabled,
  });

  function save() {
    configStore.save({
      language: language || undefined,
      alwaysThinkingEnabled,
      autoUpdatesChannel,
      minimumVersion: minimumVersion || undefined,
      includeCoAuthoredBy,
      skipDangerousModePermissionPrompt,
      tui: (tui || undefined) as "default" | "fullscreen" | undefined,
      effortLevel: (effortLevel || undefined) as
        | "low"
        | "medium"
        | "high"
        | "xhigh"
        | undefined,
      autoMemoryEnabled,
      includeGitInstructions,
      respectGitignore,
      cleanupPeriodDays: parsedCleanup(),
      claudeMdExcludes: parsedExcludes(),
      plansDirectory: plansDirectory || undefined,
      syntaxHighlightingDisabled,
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <!-- Language -->
  <div class="space-y-1">
    <label
      for="language"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      {t("settings.languageLabel")}
      <DirtyDot dirty={languageDirty} />
    </label>
    <input
      id="language"
      type="text"
      bind:value={language}
      oninput={() => configStore.markDirty()}
      placeholder={t("settings.languagePlaceholder")}
      class="input-base"
    />
  </div>

  <!-- Always Thinking Enabled -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={alwaysThinkingEnabled}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <span class="text-sm" style="color: var(--text-secondary)">
      {t("settings.alwaysThinkingEnabled")}
      <DirtyDot dirty={thinkingDirty} />
    </span>
  </label>

  <!-- Auto Updates Channel -->
  <div class="space-y-1">
    <label
      for="autoUpdatesChannel"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      {t("settings.autoUpdatesChannel")}
      <DirtyDot dirty={channelDirty} />
    </label>
    <select
      id="autoUpdatesChannel"
      bind:value={autoUpdatesChannel}
      onchange={() => configStore.markDirty()}
      class="input-base"
    >
      <option value="stable">stable</option>
      <option value="latest">latest</option>
    </select>
  </div>

  <!-- Minimum Version -->
  <div class="space-y-1">
    <label
      for="minimumVersion"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
    >
      {t("settings.minimumVersion")}
      <DirtyDot dirty={versionDirty} />
    </label>
    <input
      id="minimumVersion"
      type="text"
      bind:value={minimumVersion}
      oninput={() => configStore.markDirty()}
      placeholder={t("settings.minimumVersionPlaceholder")}
      class="input-base"
    />
  </div>

  <!-- Include Co-authored-by -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={includeCoAuthoredBy}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <span class="text-sm" style="color: var(--text-secondary)">
      {t("settings.includeCoAuthoredBy")}
      <DirtyDot dirty={coauthorDirty} />
    </span>
  </label>

  <!-- Skip Dangerous Mode Permission Prompt -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input
      type="checkbox"
      bind:checked={skipDangerousModePermissionPrompt}
      onchange={() => configStore.markDirty()}
      class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"
    />
    <span class="text-sm" style="color: var(--text-secondary)">
      {t("settings.skipDangerousModePrompt")}
      <DirtyDot dirty={skipDangerousDirty} />
    </span>
  </label>

  <!-- TUI Renderer -->
  <div class="space-y-1">
    <label
      for="tui"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
      title={t("settings.fields.tui.tooltip")}
    >
      {t("settings.fields.tui.label")}
      <DirtyDot dirty={tuiDirty} />
    </label>
    <select
      id="tui"
      bind:value={tui}
      onchange={() => configStore.markDirty()}
      class="input-base"
    >
      <option value="">(unset)</option>
      <option value="default">default</option>
      <option value="fullscreen">fullscreen</option>
    </select>
  </div>

  <!-- Effort Level -->
  <div class="space-y-1">
    <label
      for="effortLevel"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
      title={t("settings.fields.effortLevel.tooltip")}
    >
      {t("settings.fields.effortLevel.label")}
      <DirtyDot dirty={effortLevelDirty} />
    </label>
    <select
      id="effortLevel"
      bind:value={effortLevel}
      onchange={() => configStore.markDirty()}
      class="input-base"
    >
      <option value="">(unset)</option>
      <option value="low">low</option>
      <option value="medium">medium</option>
      <option value="high">high</option>
      <option value="xhigh">xhigh (Opus 4.7)</option>
    </select>
  </div>

  <!-- Auto Memory -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={autoMemoryEnabled}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.autoMemoryEnabled.tooltip")}>
      {t("settings.fields.autoMemoryEnabled.label")}
      <DirtyDot dirty={autoMemoryDirty} />
    </span>
  </label>

  <!-- Include Git Instructions -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={includeGitInstructions}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.includeGitInstructions.tooltip")}>
      {t("settings.fields.includeGitInstructions.label")}
      <DirtyDot dirty={gitInstrDirty} />
    </span>
  </label>

  <!-- Respect .gitignore -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={respectGitignore}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.respectGitignore.tooltip")}>
      {t("settings.fields.respectGitignore.label")}
      <DirtyDot dirty={gitignoreDirty} />
    </span>
  </label>

  <!-- Cleanup Period Days -->
  <div class="space-y-1">
    <label for="cleanupPeriodDays" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.cleanupPeriodDays.tooltip")}>
      {t("settings.fields.cleanupPeriodDays.label")}
      <DirtyDot dirty={cleanupDirty} />
    </label>
    <input id="cleanupPeriodDays" type="number" min="1"
           bind:value={cleanupPeriodDays}
           oninput={() => configStore.markDirty()}
           placeholder="30"
           class="input-base" />
  </div>

  <!-- CLAUDE.md Excludes -->
  <div class="space-y-1">
    <label for="claudeMdExcludes" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.claudeMdExcludes.tooltip")}>
      {t("settings.fields.claudeMdExcludes.label")}
      <DirtyDot dirty={excludesDirty} />
    </label>
    <textarea id="claudeMdExcludes" rows="3"
              bind:value={claudeMdExcludesText}
              oninput={() => configStore.markDirty()}
              placeholder={"vendor/\n.venv/"}
              class="input-base font-mono text-xs"></textarea>
  </div>

  <!-- Plans Directory -->
  <div class="space-y-1">
    <label for="plansDirectory" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.plansDirectory.tooltip")}>
      {t("settings.fields.plansDirectory.label")}
      <DirtyDot dirty={plansDirDirty} />
    </label>
    <input id="plansDirectory" type="text"
           bind:value={plansDirectory}
           oninput={() => configStore.markDirty()}
           placeholder="~/.claude/plans"
           class="input-base" />
  </div>

  <!-- Syntax Highlighting Disabled -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={syntaxHighlightingDisabled}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.syntaxHighlightingDisabled.tooltip")}>
      {t("settings.fields.syntaxHighlightingDisabled.label")}
      <DirtyDot dirty={syntaxDisabledDirty} />
    </span>
  </label>

  <!-- Save / Revert -->
  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button
      type="button"
      onclick={save}
      disabled={!configStore.isDirty || configStore.saving}
      class="btn-primary text-sm px-4 py-2"
    >
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button
      type="button"
      onclick={() => configStore.revert()}
      disabled={!configStore.isDirty}
      class="btn-secondary text-sm px-4 py-2"
    >
      {t("common.revert")}
    </button>
  </div>

  <!-- JSON Preview -->
  <JsonPreview data={previewData} title={t("settings.generalSettingsJson")} />
</div>
