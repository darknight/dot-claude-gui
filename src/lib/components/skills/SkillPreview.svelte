<script lang="ts">
  import { skillsStore } from "$lib/stores/skills.svelte";
  import { t } from "$lib/i18n";
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !skillsStore.selectedSkill}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm" style="color: var(--text-muted)">{t("skills.selectSkill")}</p>
    </div>
  {:else}
    {@const skill = skillsStore.selectedSkill}
    <div class="flex-1 overflow-auto p-6">
      <!-- Header: frontmatter fields -->
      <div class="card mb-6 px-5 py-4">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h2 class="text-lg font-semibold" style="color: var(--text-primary)">{skill.name}</h2>
            {#if skill.description}
              <p class="mt-1 text-sm" style="color: var(--text-secondary)">{skill.description}</p>
            {/if}
          </div>
          <!-- Validation status badge -->
          {#if skill.valid}
            <span class="badge badge-success flex-shrink-0">
              ✓ Valid
            </span>
          {:else}
            <span
              class="badge badge-error flex-shrink-0"
              title={skill.validationError}
            >
              ✗ Invalid
            </span>
          {/if}
        </div>

        <!-- Metadata fields -->
        <dl class="mt-4 grid grid-cols-2 gap-x-6 gap-y-2 text-xs">
          <div>
            <dt style="color: var(--text-muted)">Source</dt>
            <dd class="mt-0.5" style="color: var(--text-secondary)">
              {#if skill.source === "user"}
                <span class="badge badge-info">User</span>
              {:else}
                <span class="badge badge-neutral">Plugin: {skill.source}</span>
              {/if}
            </dd>
          </div>
          <div>
            <dt style="color: var(--text-muted)">ID</dt>
            <dd class="mt-0.5 font-mono" style="color: var(--text-secondary)">{skill.id}</dd>
          </div>
          <div class="col-span-2">
            <dt style="color: var(--text-muted)">Path</dt>
            <dd class="mt-0.5 break-all font-mono" style="color: var(--text-secondary)">{skill.path}</dd>
          </div>
          {#if !skill.valid && skill.validationError}
            <div class="col-span-2">
              <dt style="color: var(--text-muted)">Validation Error</dt>
              <dd class="mt-0.5" style="color: var(--status-error-text)">{skill.validationError}</dd>
            </div>
          {/if}
        </dl>
      </div>

      <!-- SKILL.md content -->
      <div>
        <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider" style="color: var(--text-muted)">
          SKILL.md
        </h3>
        <div class="rounded-lg">
          {#if skillsStore.contentLoading}
            <div class="code-block p-4 text-xs" style="color: var(--text-muted)">{t("common.loading")}</div>
          {:else if skillsStore.skillContent != null}
            <pre class="code-block overflow-auto p-4 text-xs leading-relaxed whitespace-pre-wrap">{skillsStore.skillContent}</pre>
          {:else}
            <div class="code-block p-4 text-xs" style="color: var(--text-muted)">{t("skills.noContent")}</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
