<script lang="ts">
  import { skillsStore } from "$lib/stores/skills.svelte";
  import type { SkillInfo } from "$lib/api/types";
  import { t } from "$lib/i18n";

  type SortMode = "name-asc" | "name-desc";

  let sortBy = $state<SortMode>("name-asc");
  let collapsed = $state<Record<string, boolean>>({});

  function pluginNameOf(skill: SkillInfo): string {
    if (skill.source === "user") return t("skills.userGroup");
    const m = skill.source.match(/^plugin:([^@]+)@/);
    return m?.[1] ?? skill.source;
  }

  const sortedSkills = $derived(
    [...skillsStore.skills].sort((a, b) => {
      const cmp = a.name.localeCompare(b.name);
      return sortBy === "name-asc" ? cmp : -cmp;
    }),
  );

  const groups = $derived.by(() => {
    const userGroup = t("skills.userGroup");
    const map = new Map<string, SkillInfo[]>();
    for (const s of sortedSkills) {
      const k = pluginNameOf(s);
      if (!map.has(k)) map.set(k, []);
      map.get(k)!.push(s);
    }
    return [...map.entries()].sort(([a], [b]) => {
      if (a === userGroup) return -1;
      if (b === userGroup) return 1;
      return a.localeCompare(b);
    });
  });

  function toggleGroup(name: string) {
    collapsed = { ...collapsed, [name]: !collapsed[name] };
  }
</script>

<div class="flex h-full flex-col overflow-hidden">
  <!-- Header + toolbar (replaces shared sub-panel header for this module) -->
  <div
    class="flex items-center justify-between gap-2 px-4 py-3"
    style="border-bottom: 1px solid var(--border-color)"
  >
    <h2
      class="truncate text-xs font-semibold uppercase tracking-wider"
      style="color: var(--text-muted)"
    >
      {t("skills.title")} <span class="normal-case">({skillsStore.skills.length})</span>
    </h2>
    <select
      bind:value={sortBy}
      class="input-base shrink-0 w-auto text-xs"
      style="padding: 0.25rem 0.375rem"
    >
      <option value="name-asc">A→Z</option>
      <option value="name-desc">Z→A</option>
    </select>
  </div>

  <!-- Body -->
  <ul class="flex-1 overflow-y-auto py-1">
    {#if skillsStore.loading && skillsStore.skills.length === 0}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">{t("common.loading")}</li>
    {:else if skillsStore.error}
      <li class="px-4 py-2 text-xs" style="color: var(--status-error-text)">{skillsStore.error}</li>
    {:else if skillsStore.skills.length === 0}
      <li class="px-4 py-2 text-xs" style="color: var(--text-muted)">{t("skills.noSkills")}</li>
    {:else}
      {#each groups as [groupName, skills], groupIndex (groupName)}
        {@const isCollapsed = collapsed[groupName] ?? false}
        <li class={groupIndex === 0 ? "" : "mt-3"}>
          <button
            type="button"
            class="flex w-full items-center gap-1 px-3 py-1.5 text-left text-xs font-semibold uppercase tracking-wider hover:opacity-80"
            style="color: var(--text-muted)"
            onclick={() => toggleGroup(groupName)}
          >
            <span class="inline-block w-3 text-center">{isCollapsed ? "▸" : "▾"}</span>
            <span class="truncate">{groupName}</span>
            <span style="color: var(--text-muted)">({skills.length})</span>
          </button>
        </li>
        {#if !isCollapsed}
          {#each skills as skill (skill.id + ":" + skill.source)}
            <li>
              {#if skillsStore.selectedSkillId === skill.id}
                <button
                  class="flex w-full items-center justify-between gap-2 py-1.5 pl-8 pr-4 text-left text-sm transition-colors"
                  style="background-color: var(--accent-bg); color: var(--text-primary)"
                  onclick={() => skillsStore.selectSkill(skill.id)}
                >
                  <span class="truncate">{skill.name}</span>
                  {#if skill.valid}
                    <span class="flex-shrink-0 text-xs" style="color: var(--status-success-text)">✓</span>
                  {:else}
                    <span
                      class="flex-shrink-0 cursor-help text-xs"
                      style="color: var(--status-error-text)"
                      title={skill.validationError ?? t("skills.invalid")}
                    >✗</span>
                  {/if}
                </button>
              {:else}
                <button
                  class="flex w-full items-center justify-between gap-2 py-1.5 pl-8 pr-4 text-left text-sm transition-colors hover:bg-[var(--bg-card-hover)]"
                  style="color: var(--text-secondary)"
                  onclick={() => skillsStore.selectSkill(skill.id)}
                >
                  <span class="truncate">{skill.name}</span>
                  {#if skill.valid}
                    <span class="flex-shrink-0 text-xs" style="color: var(--status-success-text)">✓</span>
                  {:else}
                    <span
                      class="flex-shrink-0 cursor-help text-xs"
                      style="color: var(--status-error-text)"
                      title={skill.validationError ?? t("skills.invalid")}
                    >✗</span>
                  {/if}
                </button>
              {/if}
            </li>
          {/each}
        {/if}
      {/each}
    {/if}
  </ul>
</div>
