<script lang="ts">
  import { skillsStore } from "$lib/stores/skills.svelte";
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !skillsStore.selectedSkill}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Select a skill to view details</p>
    </div>
  {:else}
    {@const skill = skillsStore.selectedSkill}
    <div class="flex-1 overflow-auto p-6">
      <!-- Header: frontmatter fields -->
      <div class="mb-6 rounded-lg border border-gray-800 bg-gray-900 px-5 py-4">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h2 class="text-lg font-semibold text-gray-100">{skill.name}</h2>
            {#if skill.description}
              <p class="mt-1 text-sm text-gray-400">{skill.description}</p>
            {/if}
          </div>
          <!-- Validation status badge -->
          {#if skill.valid}
            <span class="flex-shrink-0 rounded bg-green-900 px-2 py-1 text-xs font-medium text-green-300">
              ✓ Valid
            </span>
          {:else}
            <span
              class="flex-shrink-0 rounded bg-red-900 px-2 py-1 text-xs font-medium text-red-300"
              title={skill.validationError}
            >
              ✗ Invalid
            </span>
          {/if}
        </div>

        <!-- Metadata fields -->
        <dl class="mt-4 grid grid-cols-2 gap-x-6 gap-y-2 text-xs">
          <div>
            <dt class="text-gray-500">Source</dt>
            <dd class="mt-0.5 text-gray-300">
              {#if skill.source === "user"}
                <span class="rounded bg-blue-900 px-1.5 py-0.5 font-medium text-blue-300">User</span>
              {:else}
                <span class="rounded bg-gray-700 px-1.5 py-0.5 font-medium text-gray-300">Plugin: {skill.source}</span>
              {/if}
            </dd>
          </div>
          <div>
            <dt class="text-gray-500">ID</dt>
            <dd class="mt-0.5 font-mono text-gray-300">{skill.id}</dd>
          </div>
          <div class="col-span-2">
            <dt class="text-gray-500">Path</dt>
            <dd class="mt-0.5 break-all font-mono text-gray-300">{skill.path}</dd>
          </div>
          {#if !skill.valid && skill.validationError}
            <div class="col-span-2">
              <dt class="text-gray-500">Validation Error</dt>
              <dd class="mt-0.5 text-red-400">{skill.validationError}</dd>
            </div>
          {/if}
        </dl>
      </div>

      <!-- SKILL.md content -->
      <div>
        <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">
          SKILL.md
        </h3>
        <div class="rounded-lg border border-gray-800 bg-gray-950">
          {#if skillsStore.contentLoading}
            <div class="p-4 text-xs text-gray-500">Loading...</div>
          {:else if skillsStore.skillContent != null}
            <pre class="overflow-auto p-4 text-xs leading-relaxed text-gray-300 whitespace-pre-wrap">{skillsStore.skillContent}</pre>
          {:else}
            <div class="p-4 text-xs text-gray-600">No content available</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
