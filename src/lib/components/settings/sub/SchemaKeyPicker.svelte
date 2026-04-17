<script lang="ts">
  import { t } from "$lib/i18n";
  import { schemaSnapshot, categorizeField, type SchemaField } from "$lib/api/schema-snapshot";

  let {
    settingsMap,
    selected = $bindable(),
  }: {
    settingsMap: Record<string, unknown>;
    selected: string | null;
  } = $props();

  let search = $state("");
  let hideSet = $state(false);

  const groups = $derived.by(() => {
    const g: Record<string, SchemaField[]> = {
      auth: [],
      experience: [],
      subObject: [],
      misc: [],
      unknown: [],
    };
    const modeled = new Set(schemaSnapshot.settingsFields.map((f) => f.name));

    for (const f of schemaSnapshot.settingsFields) {
      const cat = categorizeField(f.name);
      g[cat].push(f);
    }

    for (const key of Object.keys(settingsMap)) {
      if (!modeled.has(key) && key !== "extra") {
        g.unknown.push({ name: key, type: "unknown", describe: "Not in schema snapshot" });
      }
    }

    for (const arr of Object.values(g)) {
      arr.sort((a, b) => a.name.localeCompare(b.name));
    }
    return g;
  });

  function matches(f: SchemaField): boolean {
    if (search && !f.name.toLowerCase().includes(search.toLowerCase())) return false;
    if (hideSet && settingsMap[f.name] !== undefined) return false;
    return true;
  }

  const groupDefs: Array<[keyof typeof groups, string]> = [
    ["auth", "settings.advanced.groupAuth"],
    ["experience", "settings.advanced.groupExperience"],
    ["subObject", "settings.advanced.groupSubObjects"],
    ["misc", "settings.advanced.groupMisc"],
    ["unknown", "settings.advanced.groupUnknown"],
  ];
</script>

<div class="space-y-3 overflow-auto h-full" style="max-height: calc(100vh - 200px);">
  <div class="space-y-2 sticky top-0 z-10 pb-2"
       style="background-color: var(--bg-primary); border-bottom: 1px solid var(--border-color)">
    <input type="text" bind:value={search}
           placeholder={t("settings.advanced.searchPlaceholder")}
           class="input-base text-sm" />
    <label class="flex items-center gap-2 text-xs">
      <input type="checkbox" bind:checked={hideSet}
             class="h-3 w-3 rounded" style="accent-color: var(--accent-primary)" />
      <span style="color: var(--text-secondary)">
        {t("settings.advanced.hideSet")}
      </span>
    </label>
  </div>

  {#each groupDefs as [groupKey, labelKey] (groupKey)}
    {@const visibleFields = groups[groupKey].filter(matches)}
    {#if visibleFields.length > 0}
      <div class="space-y-1">
        <h4 class="text-xs font-semibold uppercase" style="color: var(--text-muted)">
          {t(labelKey)} ({visibleFields.length})
        </h4>
        <ul class="text-sm">
          {#each visibleFields as f (f.name)}
            <li>
              <button type="button"
                      onclick={() => (selected = f.name)}
                      class="w-full text-left px-2 py-1 rounded text-xs"
                      style={selected === f.name
                        ? "background-color: var(--bg-accent); color: var(--text-primary)"
                        : "color: var(--text-secondary)"}>
                <span class="font-mono">{f.name}</span>
                {#if settingsMap[f.name] !== undefined}
                  <span style="color: var(--accent-primary)">✓</span>
                {/if}
                <span class="ml-1" style="color: var(--text-muted)">
                  ({f.type}{f.enumValues ? ` · ${f.enumValues.length}` : ""})
                </span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  {/each}
</div>
