<script lang="ts">
  import { skillsStore } from "$lib/stores/skills.svelte";

  // Group skills: user skills first, then plugin skills
  let userSkills = $derived(
    skillsStore.skills.filter((s) => s.source === "user")
  );
  let pluginSkills = $derived(
    skillsStore.skills.filter((s) => s.source !== "user")
  );
</script>

<div class="flex-1 overflow-auto p-6">
  {#if skillsStore.loading}
    <p class="text-sm text-gray-500">Loading skills...</p>
  {:else if skillsStore.error}
    <div class="mb-4 rounded border border-red-800 bg-red-950 px-4 py-2">
      <p class="text-xs text-red-400">{skillsStore.error}</p>
    </div>
  {/if}

  {#if skillsStore.skills.length === 0 && !skillsStore.loading}
    <div class="flex h-full items-center justify-center">
      <p class="text-sm text-gray-600">No skills found</p>
    </div>
  {:else}
    <!-- User skills -->
    {#if userSkills.length > 0}
      <div class="mb-4">
        <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">
          User Skills
        </h3>
        <div class="space-y-2">
          {#each userSkills as skill (skill.id)}
            <button
              class="w-full rounded-lg border px-4 py-3 text-left transition-colors
                {skillsStore.selectedSkillId === skill.id
                ? 'border-blue-700 bg-blue-950'
                : 'border-gray-800 bg-gray-900 hover:border-gray-700'}"
              onclick={() => skillsStore.selectSkill(skill.id)}
            >
              <div class="flex items-start justify-between gap-2">
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="font-semibold text-gray-100">{skill.name}</span>
                    <!-- Source badge -->
                    <span class="rounded bg-blue-900 px-1.5 py-0.5 text-xs font-medium text-blue-300">
                      User
                    </span>
                  </div>
                  {#if skill.description}
                    <p class="mt-0.5 truncate text-xs text-gray-400">{skill.description}</p>
                  {/if}
                </div>
                <!-- Validation status -->
                {#if skill.valid}
                  <span class="flex-shrink-0 text-green-400" title="Valid">✓</span>
                {:else}
                  <span
                    class="flex-shrink-0 cursor-help text-red-400"
                    title={skill.validationError ?? "Invalid"}
                  >✗</span>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Plugin skills -->
    {#if pluginSkills.length > 0}
      <div>
        <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">
          Plugin Skills
        </h3>
        <div class="space-y-2">
          {#each pluginSkills as skill (skill.id)}
            <button
              class="w-full rounded-lg border px-4 py-3 text-left transition-colors
                {skillsStore.selectedSkillId === skill.id
                ? 'border-blue-700 bg-blue-950'
                : 'border-gray-800 bg-gray-900 hover:border-gray-700'}"
              onclick={() => skillsStore.selectSkill(skill.id)}
            >
              <div class="flex items-start justify-between gap-2">
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="font-semibold text-gray-100">{skill.name}</span>
                    <!-- Source badge -->
                    <span class="rounded bg-gray-700 px-1.5 py-0.5 text-xs font-medium text-gray-300">
                      Plugin: {skill.source}
                    </span>
                  </div>
                  {#if skill.description}
                    <p class="mt-0.5 truncate text-xs text-gray-400">{skill.description}</p>
                  {/if}
                </div>
                <!-- Validation status -->
                {#if skill.valid}
                  <span class="flex-shrink-0 text-green-400" title="Valid">✓</span>
                {:else}
                  <span
                    class="flex-shrink-0 cursor-help text-red-400"
                    title={skill.validationError ?? "Invalid"}
                  >✗</span>
                {/if}
              </div>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>
