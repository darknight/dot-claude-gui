<script lang="ts">
  import { projectsStore } from "$lib/stores/projects.svelte";
  import { modeStore } from "$lib/stores/mode.svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { toastStore } from "$lib/stores/toast.svelte";
  import { t } from "$lib/i18n";
  import type { ProjectEntry } from "$lib/api/types";

  // Group projects by bound account name (or sentinel for unbound).
  const groups = $derived.by(() => {
    const map = new Map<string, ProjectEntry[]>();
    for (const entry of projectsStore.entries) {
      const key = entry.account ?? "__unbound__";
      const list = map.get(key) ?? [];
      list.push(entry);
      map.set(key, list);
    }
    return Array.from(map.entries()).sort(([a], [b]) => {
      if (a === "__unbound__") return 1;
      if (b === "__unbound__") return -1;
      return a.localeCompare(b);
    });
  });

  function basename(path: string): string {
    const parts = path.split("/").filter(Boolean);
    return parts[parts.length - 1] ?? path;
  }

  async function addProject() {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    try {
      await projectsStore.add(selected);
      modeStore.setSelectedProject(selected);
    } catch (e) {
      toastStore.error(t("shell.addProjectFailed"));
      console.error("addProject failed", e);
    }
  }
</script>

<div class="flex flex-col h-full" style="background-color: var(--bg-secondary)">
  <div class="px-4 py-3" style="border-bottom: 1px solid var(--border-color)">
    <h2 class="text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
      {t("shell.projectsList")}
    </h2>
  </div>

  <ul class="flex-1 overflow-y-auto py-2">
    {#each groups as [account, projects] (account)}
      <li class="mt-2 first:mt-0">
        <h3 class="px-4 py-1 text-xs uppercase tracking-wider" style="color: var(--text-muted)">
          {account === "__unbound__" ? t("shell.unbound") : "@" + account}
        </h3>
        <ul>
          {#each projects as project (project.path)}
            {@const isActive = modeStore.selectedProject === project.path}
            <li>
              <button
                class="w-full px-4 py-2 text-left text-sm flex items-center gap-2 transition-colors {isActive ? '' : 'hover:bg-[var(--bg-card-hover)]'}"
                style="background-color: {isActive ? 'var(--accent-bg)' : 'transparent'}; color: {isActive ? 'var(--accent-text)' : project.stale ? 'var(--text-muted)' : 'var(--text-primary)'}"
                onclick={() => modeStore.setSelectedProject(project.path)}
                title={project.path}
              >
                <span class="flex-1 truncate">{basename(project.path)}</span>
                {#if project.stale}
                  <span class="text-xs" style="color: var(--text-muted)">· {t("shell.stale")}</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      </li>
    {/each}
  </ul>

  <button
    class="mx-3 mb-3 mt-3 px-3 py-1.5 text-sm rounded transition-colors"
    style="background-color: var(--accent-bg); color: var(--accent-text); border-top: 1px solid var(--border-color)"
    onclick={addProject}
  >
    + {t("shell.addProject")}
  </button>
</div>
