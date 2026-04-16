# Theme System Redesign — Light & Dark

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace 644+ hardcoded Tailwind color classes across 30+ Svelte components with a comprehensive CSS variable system, making both light and dark themes polished and professional.

**Architecture:** Expand `app.css` CSS variables from 11 to ~35 semantic tokens covering all color needs (surfaces, text, borders, status, interactive). Replace hardcoded Tailwind color classes with inline `style="..."` using CSS variables (matching App.svelte's existing pattern). Group replacements by module for manageable commits.

**Tech Stack:** Svelte 5, Tailwind CSS 4, CSS custom properties

**Aesthetic Direction:** "Refined Developer Tool" — light theme uses warm off-white with blue-gray surfaces; dark theme uses deep navy-blacks instead of pure gray. Both feel intentional and cohesive.

---

## Color Mapping Reference

All subsequent tasks reference this table. Each hardcoded Tailwind class maps to a CSS variable.

### Backgrounds

| Old Class | CSS Variable | Semantic Role |
|-----------|-------------|---------------|
| `bg-gray-950` | `--bg-primary` | Main content area background |
| `bg-gray-900` | `--bg-card` | Card/panel surface |
| `bg-gray-800` | `--bg-tertiary` | Input backgrounds, elevated surfaces |
| `bg-gray-800/50`, `hover:bg-gray-800/50` | `--bg-card-hover` | Hover state for list items |
| `bg-gray-700` | `--bg-tertiary` | Buttons (secondary), toggle off |
| `bg-gray-100 dark:bg-gray-800` | `--bg-tertiary` | Code blocks in light+dark |
| `bg-white dark:bg-gray-800` | `--bg-input` | Form inputs |
| `bg-blue-600`, `bg-blue-700` | `--accent-primary` | Primary action buttons |
| `hover:bg-blue-500`, `hover:bg-blue-600` | `--accent-primary-hover` | Button hover |
| `bg-blue-950`, `bg-blue-900` (selected) | `--accent-bg` | Selected/active item bg |
| `bg-emerald-600` | `--status-success-solid` | Upgrade button |
| `bg-green-600` | `--switch-on` | Toggle switch on |
| `bg-red-900`, `bg-red-950` | `--status-error-bg` | Error container bg |
| `bg-green-900` | `--status-success-bg` | Success badge/container bg |
| `bg-green-100 dark:bg-green-900` | `--status-success-bg` | Success badge |
| `bg-blue-900` | `--status-info-bg` | Info badge bg |
| `bg-blue-100 dark:bg-blue-900` | `--status-info-bg` | Info badge |
| `bg-yellow-900` | `--status-warning-bg` | Warning badge bg |
| `bg-orange-900` | `--status-warning-bg` | Warning badge bg |
| `bg-purple-900` | `--status-purple-bg` | Memory type badge |
| `bg-red-900/50`, `bg-red-900/60` | `--status-error-bg` | Blocked/error overlay |
| `bg-green-900/50` | `--status-success-bg` | Override enabled |
| `hover:bg-red-900/50` | `--status-error-bg` | Delete/remove hover |

### Text

| Old Class | CSS Variable | Semantic Role |
|-----------|-------------|---------------|
| `text-gray-100`, `text-gray-200` | `--text-primary` | Primary text (headings, names) |
| `text-gray-300` | `--text-secondary` | Secondary text (labels, items) |
| `text-gray-400` | `--text-secondary` | Description, muted labels |
| `text-gray-500` | `--text-muted` | Placeholder-like, loading |
| `text-gray-600` | `--text-muted` | Empty state messages |
| `text-gray-700 dark:text-gray-300` | `--text-secondary` | Label text |
| `text-gray-900 dark:text-gray-100` | `--text-primary` | Primary text |
| `text-white` | `--text-on-accent` | Text on colored buttons |
| `text-blue-500`, `text-blue-400` | `--accent-text` | Links, accent text |
| `text-blue-300` | `--status-info-text` | Info badge text |
| `text-green-400`, `text-green-300` | `--status-success-text` | Success text/badge |
| `text-green-700 dark:text-green-300` | `--status-success-text` | Success badge text |
| `text-red-400`, `text-red-300` | `--status-error-text` | Error text/badge |
| `text-red-700 dark:text-red-300` | `--status-error-text` | Error badge text |
| `text-yellow-300` | `--status-warning-text` | Warning badge text |
| `text-orange-300` | `--status-warning-text` | Warning badge text |
| `text-purple-300` | `--status-purple-text` | Memory type badge text |

### Borders

| Old Class | CSS Variable | Semantic Role |
|-----------|-------------|---------------|
| `border-gray-800` | `--border-color` | Default card/section borders |
| `border-gray-700` | `--border-strong` | Form input borders, dividers |
| `border-gray-600` | `--border-strong` | Input borders (with dark:) |
| `border-gray-300 dark:border-gray-600` | `--border-strong` | Form borders |
| `border-gray-200 dark:border-gray-700` | `--border-color` | Section borders |
| `border-blue-700`, `border-blue-500` | `--accent-primary` | Selected item border, focus ring |
| `border-red-800` | `--status-error-text` | Error container border |
| `border-green-800`, `border-green-700` | `--status-success-text` | Success container border |

### Interactive

| Old Class | CSS Variable | Semantic Role |
|-----------|-------------|---------------|
| `focus:ring-blue-500`, `focus:ring-2` | `--focus-ring` | Focus ring color |
| `focus:ring-offset-gray-900` | `--bg-primary` | Focus ring offset |
| `accent-blue-500` | `--accent-primary` | Checkbox/radio accent |
| `placeholder-gray-400`, `placeholder-gray-500`, `placeholder-gray-600` | `--text-muted` | Input placeholder |
| `divide-gray-800/50` | `--border-color` | Table row dividers |

### Approach per element type

- **Container backgrounds & borders**: Use `style="background-color: var(--xxx); border-color: var(--yyy)"`
- **Text colors**: Use `style="color: var(--xxx)"`
- **Buttons with hover**: Keep Tailwind classes but use theme-aware ones. For primary buttons use a shared CSS class `.btn-primary` / `.btn-secondary` defined in app.css
- **Badges**: Use `style="background-color: var(--xxx); color: var(--yyy)"`
- **Focus rings**: Use CSS class `.focus-ring` defined in app.css
- **Placeholder text**: Use `::placeholder { color: var(--text-muted) }` global rule in app.css

---

## Task 1: CSS Variable System + Utility Classes

**Files:**
- Modify: `src/app.css`

- [ ] **Step 1: Expand CSS variables to full design token set**

Replace the existing `:root` and `.dark` blocks in `src/app.css` with the comprehensive set below. Keep all existing non-color CSS (scrollbar, body font, etc.) unchanged.

```css
:root {
  --sidebar-width: 56px;
  --subpanel-width: 240px;
  --app-font-size: 14px;

  /* ── Surfaces ── */
  --bg-primary: #f8f9fb;
  --bg-secondary: #eef0f4;
  --bg-tertiary: #e2e6ec;
  --bg-input: #ffffff;
  --bg-card: #ffffff;
  --bg-card-hover: #f3f5f8;
  --bg-code: #f0f2f6;
  --bg-overlay: rgba(0, 0, 0, 0.4);

  /* ── Text ── */
  --text-primary: #1a1e26;
  --text-secondary: #4a5264;
  --text-muted: #838c9e;
  --text-on-accent: #ffffff;

  /* ── Borders ── */
  --border-color: #d5dae2;
  --border-subtle: #e2e6ec;
  --border-strong: #c2c9d4;

  /* ── Accent / Interactive ── */
  --accent-primary: #2563eb;
  --accent-primary-hover: #1d4ed8;
  --accent-bg: #eff5ff;
  --accent-text: #2563eb;
  --focus-ring: rgba(37, 99, 235, 0.4);

  /* ── Status: Success ── */
  --status-success-bg: #ecfdf5;
  --status-success-text: #057a55;
  --status-success-solid: #059669;
  --status-success-solid-hover: #047857;

  /* ── Status: Error ── */
  --status-error-bg: #fef2f2;
  --status-error-text: #dc2626;

  /* ── Status: Warning ── */
  --status-warning-bg: #fffbeb;
  --status-warning-text: #b45309;

  /* ── Status: Info ── */
  --status-info-bg: #eff6ff;
  --status-info-text: #1e40af;

  /* ── Status: Purple (memory type) ── */
  --status-purple-bg: #f5f3ff;
  --status-purple-text: #6d28d9;

  /* ── Components ── */
  --switch-on: #22c55e;
  --switch-off: #d1d5db;
  --badge-bg: #e2e6ec;
  --badge-text: #4a5264;
  --dirty-dot: #f97316;

  /* ── Nav ── */
  --nav-active-bg: #2563eb;
  --nav-active-text: #ffffff;
  --nav-hover-bg: #e2e6ec;
}

.dark {
  /* ── Surfaces ── */
  --bg-primary: #0a0e1a;
  --bg-secondary: #111828;
  --bg-tertiary: #1c2536;
  --bg-input: #151d2e;
  --bg-card: #151d2e;
  --bg-card-hover: #1c2536;
  --bg-code: #0c1119;
  --bg-overlay: rgba(0, 0, 0, 0.6);

  /* ── Text ── */
  --text-primary: #e4e8f0;
  --text-secondary: #99a3b8;
  --text-muted: #596478;
  --text-on-accent: #ffffff;

  /* ── Borders ── */
  --border-color: #232d42;
  --border-subtle: #1a2235;
  --border-strong: #2d3a52;

  /* ── Accent / Interactive ── */
  --accent-primary: #3b82f6;
  --accent-primary-hover: #60a5fa;
  --accent-bg: rgba(59, 130, 246, 0.15);
  --accent-text: #60a5fa;
  --focus-ring: rgba(59, 130, 246, 0.5);

  /* ── Status: Success ── */
  --status-success-bg: rgba(34, 197, 94, 0.15);
  --status-success-text: #4ade80;
  --status-success-solid: #22c55e;
  --status-success-solid-hover: #16a34a;

  /* ── Status: Error ── */
  --status-error-bg: rgba(239, 68, 68, 0.15);
  --status-error-text: #f87171;

  /* ── Status: Warning ── */
  --status-warning-bg: rgba(245, 158, 11, 0.15);
  --status-warning-text: #fbbf24;

  /* ── Status: Info ── */
  --status-info-bg: rgba(59, 130, 246, 0.15);
  --status-info-text: #93c5fd;

  /* ── Status: Purple ── */
  --status-purple-bg: rgba(139, 92, 246, 0.15);
  --status-purple-text: #c4b5fd;

  /* ── Components ── */
  --switch-on: #22c55e;
  --switch-off: #374151;
  --badge-bg: #1c2536;
  --badge-text: #99a3b8;
  --dirty-dot: #f97316;

  /* ── Nav ── */
  --nav-active-bg: #2563eb;
  --nav-active-text: #ffffff;
  --nav-hover-bg: #1c2536;
}
```

- [ ] **Step 2: Add global utility styles and button classes**

Append to `src/app.css` (after the scrollbar styles):

```css
/* ── Global form resets ── */
input::placeholder,
textarea::placeholder {
  color: var(--text-muted);
}

/* ── Reusable button classes ── */
.btn-primary {
  background-color: var(--accent-primary);
  color: var(--text-on-accent);
  padding: 0.25rem 0.75rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  line-height: 1rem;
  transition: background-color 150ms;
  cursor: pointer;
}
.btn-primary:hover:not(:disabled) {
  background-color: var(--accent-primary-hover);
}
.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-secondary {
  border: 1px solid var(--border-strong);
  color: var(--text-secondary);
  padding: 0.25rem 0.75rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  line-height: 1rem;
  transition: background-color 150ms;
  cursor: pointer;
  background: transparent;
}
.btn-secondary:hover:not(:disabled) {
  background-color: var(--bg-tertiary);
}
.btn-secondary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-success {
  background-color: var(--status-success-solid);
  color: var(--text-on-accent);
  padding: 0.25rem 0.75rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  line-height: 1rem;
  transition: background-color 150ms;
  cursor: pointer;
}
.btn-success:hover:not(:disabled) {
  background-color: var(--status-success-solid-hover);
}
.btn-success:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-danger-ghost {
  color: var(--text-muted);
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  line-height: 1rem;
  transition: all 150ms;
  cursor: pointer;
  background: transparent;
}
.btn-danger-ghost:hover {
  background-color: var(--status-error-bg);
  color: var(--status-error-text);
}

/* ── Form input base ── */
.input-base {
  width: 100%;
  border-radius: 0.25rem;
  border: 1px solid var(--border-strong);
  background-color: var(--bg-input);
  color: var(--text-primary);
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  outline: none;
  transition: border-color 150ms, box-shadow 150ms;
}
.input-base:focus {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px var(--focus-ring);
}

/* ── Badge base ── */
.badge {
  border-radius: 0.25rem;
  padding: 0.125rem 0.375rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
}
.badge-info {
  background-color: var(--status-info-bg);
  color: var(--status-info-text);
}
.badge-success {
  background-color: var(--status-success-bg);
  color: var(--status-success-text);
}
.badge-error {
  background-color: var(--status-error-bg);
  color: var(--status-error-text);
}
.badge-warning {
  background-color: var(--status-warning-bg);
  color: var(--status-warning-text);
}
.badge-purple {
  background-color: var(--status-purple-bg);
  color: var(--status-purple-text);
}
.badge-neutral {
  background-color: var(--badge-bg);
  color: var(--badge-text);
}

/* ── Card base ── */
.card {
  background-color: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 0.75rem 1rem;
  transition: border-color 150ms;
}
.card:hover {
  border-color: var(--border-strong);
}

/* ── Error/success container ── */
.alert-error {
  border: 1px solid var(--status-error-text);
  background-color: var(--status-error-bg);
  color: var(--status-error-text);
  border-radius: 0.25rem;
  padding: 0.5rem 1rem;
  font-size: 0.75rem;
}
.alert-success {
  border: 1px solid var(--status-success-text);
  background-color: var(--status-success-bg);
  color: var(--status-success-text);
  border-radius: 0.25rem;
  padding: 0.5rem 1rem;
  font-size: 0.75rem;
}

/* ── Toggle switch ── */
.toggle-track {
  position: relative;
  display: inline-flex;
  height: 1.25rem;
  width: 2.25rem;
  flex-shrink: 0;
  cursor: pointer;
  border-radius: 9999px;
  border: 2px solid transparent;
  transition: background-color 200ms;
}
.toggle-track[aria-checked="true"] {
  background-color: var(--switch-on);
}
.toggle-track[aria-checked="false"] {
  background-color: var(--switch-off);
}
.toggle-knob {
  pointer-events: none;
  display: inline-block;
  height: 1rem;
  width: 1rem;
  transform: translateX(0);
  border-radius: 9999px;
  background-color: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: transform 200ms;
}
.toggle-track[aria-checked="true"] .toggle-knob {
  transform: translateX(1rem);
}

/* ── Code/pre block ── */
.code-block {
  background-color: var(--bg-code);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 0.25rem;
  padding: 0.75rem;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
  font-size: 0.75rem;
  line-height: 1.625;
}
```

- [ ] **Step 3: Run svelte-check to verify no regressions**

```bash
cd /Users/eric.yao/workspace/darknight/dot-claude-gui && pnpm exec svelte-check 2>&1 | tail -3
```

Expected: same 0 errors / 12 warnings baseline.

- [ ] **Step 4: Commit**

```bash
git add src/app.css
git commit -m "feat(theme): comprehensive CSS variable system + utility classes

Add ~35 semantic design tokens for light and dark themes. Light theme
uses warm off-white with blue-gray surfaces; dark theme uses deep
navy-blacks. Add reusable CSS classes for buttons, badges, inputs,
cards, toggles, and alerts."
```

---

## Task 2: Shared Components

**Files:**
- Modify: `src/lib/components/shared/StringListEditor.svelte`
- Modify: `src/lib/components/shared/ScopeSelector.svelte`
- Modify: `src/lib/components/shared/DirtyDot.svelte`
- Modify: `src/lib/components/shared/Toast.svelte`

These are used across the entire app, so theming them first gives the widest impact.

- [ ] **Step 1: Theme StringListEditor.svelte**

Read `src/lib/components/shared/StringListEditor.svelte`. Apply these replacements:

| Find | Replace with |
|------|-------------|
| `class="text-xs font-medium text-gray-400 uppercase tracking-wide"` | `class="text-xs font-medium uppercase tracking-wide" style="color: var(--text-muted)"` |
| `class="flex-1 rounded bg-gray-800 px-2 py-1 text-xs text-gray-200 font-mono break-all"` | `class="flex-1 rounded px-2 py-1 text-xs font-mono break-all" style="background-color: var(--bg-tertiary); color: var(--text-primary)"` |
| `class="opacity-0 group-hover:opacity-100 transition-opacity rounded p-1 text-gray-500 hover:text-red-400 hover:bg-gray-700"` | `class="btn-danger-ghost opacity-0 group-hover:opacity-100 transition-opacity rounded p-1"` |
| The input element: replace all color classes (`border-gray-700 bg-gray-800 text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500`) | `class="input-base"` (with any non-color classes like sizing kept) |
| The add button: replace `bg-blue-600 hover:bg-blue-500 active:bg-blue-700 text-white` | `class="btn-primary"` |

- [ ] **Step 2: Theme ScopeSelector.svelte**

Read `src/lib/components/shared/ScopeSelector.svelte`. Apply:

| Find | Replace with |
|------|-------------|
| `text-gray-200` (display name) | `style="color: var(--text-primary)"` |
| `text-gray-400` (dropdown arrow) | `style="color: var(--text-muted)"` |
| `hover:bg-gray-800` (button hover) | use `--bg-card-hover` |
| `bg-gray-800` (dropdown bg) | `style="background-color: var(--bg-card)"` |
| `border-gray-700` (dropdown border) | `style="border-color: var(--border-strong)"` |
| `text-blue-400` (active scope) | `style="color: var(--accent-text)"` |
| `text-gray-300` (inactive scope) | `style="color: var(--text-secondary)"` |
| `hover:bg-gray-700` (item hover) | use `--bg-tertiary` |
| `text-red-400` (error) | `style="color: var(--status-error-text)"` |

- [ ] **Step 3: Theme DirtyDot.svelte**

Replace `bg-orange-500` with `style="background-color: var(--dirty-dot)"`.

- [ ] **Step 4: Verify Toast.svelte**

Read `src/lib/components/shared/Toast.svelte` — it already uses CSS variables. Verify the hex colors for status borders (#238636, #f85149, #d29922, #58a6ff) and replace them with CSS variable references:
- `#238636` → `var(--status-success-text)`
- `#f85149` → `var(--status-error-text)`
- `#d29922` → `var(--status-warning-text)`
- `#58a6ff` → `var(--status-info-text)`

- [ ] **Step 5: Visual verify + commit**

```bash
pnpm exec svelte-check 2>&1 | tail -3
git add src/lib/components/shared/
git commit -m "feat(theme): theme shared components with CSS variables"
```

---

## Task 3: Settings Module — Editors

**Files:**
- Modify: `src/lib/components/settings/GeneralEditor.svelte`
- Modify: `src/lib/components/settings/PermissionsEditor.svelte`
- Modify: `src/lib/components/settings/SandboxEditor.svelte`
- Modify: `src/lib/components/settings/StatusLineEditor.svelte`
- Modify: `src/lib/components/settings/EnvVarEditor.svelte`
- Modify: `src/lib/components/settings/HooksEditor.svelte`
- Modify: `src/lib/components/settings/JsonPreview.svelte`
- Modify: `src/lib/components/settings/SettingsEditor.svelte`

These files already have `dark:` variants for many classes. Replace both the light and dark classes with CSS variables.

- [ ] **Step 1: Theme GeneralEditor.svelte**

Read the file. Common patterns to replace:

| Pattern | Replacement |
|---------|------------|
| `class="block text-sm font-medium text-gray-700 dark:text-gray-300"` on labels | `class="block text-sm font-medium" style="color: var(--text-secondary)"` |
| `class="w-full rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"` on inputs/selects | `class="input-base"` |
| `class="h-4 w-4 rounded border-gray-300 dark:border-gray-600 text-blue-500 focus:ring-blue-500"` on checkboxes | `class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)"` |
| `class="text-sm text-gray-700 dark:text-gray-300"` on checkbox labels | `class="text-sm" style="color: var(--text-secondary)"` |
| Save button: `bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-40` | `class="btn-primary"` |
| Revert button: `border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-40` | `class="btn-secondary"` |
| `border-t border-gray-200 dark:border-gray-700` dividers | `style="border-color: var(--border-color)"` |

- [ ] **Step 2: Theme PermissionsEditor, SandboxEditor, StatusLineEditor**

Same label/input/button patterns as GeneralEditor. Read each file and apply the same mapping. These three share identical form element patterns.

- [ ] **Step 3: Theme EnvVarEditor.svelte**

Additional patterns beyond the standard:
| Pattern | Replacement |
|---------|------------|
| `<code>` key badges: `bg-gray-100 dark:bg-gray-800 ... text-gray-800 dark:text-gray-200 border border-gray-200 dark:border-gray-700` | `class="rounded px-2 py-1.5 text-xs font-mono" style="background-color: var(--bg-tertiary); color: var(--text-primary); border: 1px solid var(--border-color)"` |
| `text-gray-400 text-sm` equals sign | `style="color: var(--text-muted)"` |
| `text-red-400 ... hover:text-red-600` Remove button | `class="btn-danger-ghost"` + keep opacity transition |
| `text-gray-600 dark:text-gray-400` label | `style="color: var(--text-muted)"` |
| `text-red-500` validation error | `style="color: var(--status-error-text)"` |
| Dirty dot orange span in env row — keep as is (uses absolute positioning) but replace `bg-orange-500` → `style="background-color: var(--dirty-dot)"` |

- [ ] **Step 4: Theme HooksEditor.svelte**

Largest file. Additional patterns:
| Pattern | Replacement |
|---------|------------|
| `border border-gray-200 dark:border-gray-700 rounded-lg` (rule card) | `class="card"` |
| `text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase` (rule header) | `class="text-xs font-semibold uppercase tracking-wide" style="color: var(--text-muted)"` |
| `text-xs text-red-500 hover:text-red-700 dark:hover:text-red-400` (delete rule) | `class="btn-danger-ghost text-xs"` |
| `text-xs font-medium text-gray-600 dark:text-gray-400` (field label) | `class="text-xs font-medium" style="color: var(--text-muted)"` |
| `ml-3 pl-3 border-l-2 border-gray-200 dark:border-gray-600` (hook indent) | keep layout, change `border-gray-200 dark:border-gray-600` → `style="border-color: var(--border-color)"` |
| `accent-blue-500` (radio) | `style="accent-color: var(--accent-primary)"` |
| `text-xs text-gray-500 dark:text-gray-400` (field hint) | `class="text-xs" style="color: var(--text-muted)"` |
| `text-xs text-blue-500 hover:text-blue-700 dark:hover:text-blue-400` (+ Add Hook) | `class="text-xs" style="color: var(--accent-text)"` |
| `border-dashed border-gray-300 dark:border-gray-600 ... text-gray-600 dark:text-gray-400` (+ Add Rule) | `class="btn-secondary"` + add `border-style: dashed` |

- [ ] **Step 5: Theme JsonPreview.svelte and SettingsEditor.svelte**

Read both files. JsonPreview likely uses code block styling → use `.code-block` class. SettingsEditor is the wrapper → replace any hardcoded nav/tab colors.

- [ ] **Step 6: Commit**

```bash
pnpm exec svelte-check 2>&1 | tail -3
git add src/lib/components/settings/
git commit -m "feat(theme): theme settings editors with CSS variables"
```

---

## Task 4: Plugins Module

**Files:**
- Modify: `src/lib/components/plugins/InstalledPlugins.svelte`
- Modify: `src/lib/components/plugins/MarketplaceBrowser.svelte`
- Modify: `src/lib/components/plugins/MarketplaceManager.svelte`
- Modify: `src/lib/components/plugins/ProjectActivation.svelte`
- Modify: `src/lib/components/plugins/PluginsModule.svelte`

- [ ] **Step 1: Theme InstalledPlugins.svelte**

| Pattern | Replacement |
|---------|------------|
| `text-gray-500` (loading, group label) | `style="color: var(--text-muted)"` |
| `text-gray-600` (empty state) | `style="color: var(--text-muted)"` |
| Error container `border-red-800 bg-red-950 ... text-red-400` | `class="alert-error"` |
| Plugin card `border-gray-800 bg-gray-900 ... hover:border-gray-700` | `class="card"` |
| `text-gray-100` (plugin name) | `style="color: var(--text-primary)"` |
| `text-gray-500` (marketplace/version) | `style="color: var(--text-muted)"` |
| `text-gray-400` (description) | `style="color: var(--text-secondary)"` |
| `bg-red-900 ... text-red-300` (blocked badge) | `class="badge badge-error"` |
| Uninstall button | `class="btn-danger-ghost opacity-0 group-hover:opacity-100 transition-opacity"` |
| Toggle switch: replace `bg-green-600`/`bg-gray-700`/`bg-white` | Use `.toggle-track` + `.toggle-knob` classes |
| Output panel `border-gray-800 bg-gray-950` | `class="code-block"` with `max-height` kept |

- [ ] **Step 2: Theme MarketplaceBrowser.svelte**

| Pattern | Replacement |
|---------|------------|
| `text-gray-400` (label) | `style="color: var(--text-muted)"` |
| `text-gray-600` (empty state) | `style="color: var(--text-muted)"` |
| Select: `border-gray-700 bg-gray-800 text-gray-100 focus:border-blue-500` | `class="input-base"` |
| Plugin card: `border-gray-800 bg-gray-900 hover:border-gray-700` | `class="card"` |
| `text-gray-100` (name) | `style="color: var(--text-primary)"` |
| `text-gray-500` (version) | `style="color: var(--text-muted)"` |
| Category badge: `bg-blue-900 ... text-blue-300` (+ dark: variants) | `class="badge badge-info"` |
| Installed badge: `bg-green-100 ... text-green-700` (+ dark: variants) | `class="badge badge-success"` |
| `text-gray-400` (description) | `style="color: var(--text-secondary)"` |
| Install button: replace all `bg-blue-600 ... dark:bg-blue-500 dark:hover:bg-blue-400` | `class="btn-primary"` |
| Upgrade button: replace `bg-emerald-600 ... dark:bg-emerald-500` | `class="btn-success"` |
| Re-install button: replace `border-gray-300 ... dark:border-gray-600 dark:text-gray-400 dark:hover:bg-gray-800` | `class="btn-secondary"` |
| Disabled button `bg-gray-700 text-gray-400` | `class="btn-primary" disabled` |
| Version diff text `text-gray-500 dark:text-gray-400` | `style="color: var(--text-muted)"` |
| Output panel | `class="code-block"` |
| Error container | `class="alert-error"` |

- [ ] **Step 3: Theme MarketplaceManager.svelte**

| Pattern | Replacement |
|---------|------------|
| Form card: `border-gray-800 bg-gray-900` | `class="card"` |
| `text-gray-200` heading | `style="color: var(--text-primary)"` |
| Input fields | `class="input-base"` |
| Add button: `bg-blue-700 hover:bg-blue-600 text-white` | `class="btn-primary"` |
| Output panel: `border-gray-700 bg-gray-950` | `class="code-block"` |
| `text-gray-400` label | `style="color: var(--text-muted)"` |
| `text-gray-300` output text | `style="color: var(--text-secondary)"` |
| Remove button: `text-red-400 hover:text-red-300` | `class="btn-danger-ghost"` |
| Error container | `class="alert-error"` |

- [ ] **Step 4: Theme ProjectActivation.svelte**

| Pattern | Replacement |
|---------|------------|
| Table card: `bg-gray-900 border-gray-800` | `class="card"` (without padding, add separately) |
| `text-gray-200` headings | `style="color: var(--text-primary)"` |
| `text-gray-500` labels | `style="color: var(--text-muted)"` |
| `text-gray-600` version/marketplace | `style="color: var(--text-muted)"` |
| `bg-red-900/60 text-red-400` blocked badge | `class="badge badge-error"` |
| `hover:bg-gray-800/30` row hover | `style: background-color: var(--bg-card-hover)` |
| `divide-gray-800/50` row dividers | `style="border-color: var(--border-color)"` |
| Toggle switch | `.toggle-track` + `.toggle-knob` |
| Override buttons: green/red/gray variants | Use `style` with status CSS variables |

- [ ] **Step 5: Theme PluginsModule.svelte**

Read and replace any hardcoded colors (likely minimal — it's a wrapper).

- [ ] **Step 6: Commit**

```bash
pnpm exec svelte-check 2>&1 | tail -3
git add src/lib/components/plugins/
git commit -m "feat(theme): theme plugins module with CSS variables"
```

---

## Task 5: Skills Module

**Files:**
- Modify: `src/lib/components/skills/SkillList.svelte`
- Modify: `src/lib/components/skills/SkillPreview.svelte`
- Modify: `src/lib/components/skills/SkillsModule.svelte`

- [ ] **Step 1: Theme SkillList.svelte**

| Pattern | Replacement |
|---------|------------|
| Header bar: `border-gray-800` | `style="border-color: var(--border-color)"` |
| `text-gray-300` (title) | `style="color: var(--text-primary)"` |
| `text-gray-500` (count, group count) | `style="color: var(--text-muted)"` |
| Sort select: `bg-gray-800 text-gray-300` | `style="background-color: var(--bg-tertiary); color: var(--text-secondary)"` |
| `text-gray-500` (loading, group header) | `style="color: var(--text-muted)"` |
| `text-red-400` error | `style="color: var(--status-error-text)"` |
| Group header: `text-gray-500 hover:text-gray-300` | `style="color: var(--text-muted)"` with hover via JS or keep |
| `text-gray-600` (group count) | `style="color: var(--text-muted)"` |
| Selected item: `bg-gray-800 text-white` | `style="background-color: var(--accent-bg); color: var(--text-primary)"` |
| Unselected item: `text-gray-400 hover:bg-gray-800/50 hover:text-gray-200` | `style="color: var(--text-secondary)"` |
| `text-green-400` valid checkmark | `style="color: var(--status-success-text)"` |
| `text-red-400` invalid mark | `style="color: var(--status-error-text)"` |

- [ ] **Step 2: Theme SkillPreview.svelte**

Read and apply. Likely contains code block styling for SKILL.md content → use `.code-block`. Apply standard text/bg variable mappings.

- [ ] **Step 3: SkillsModule.svelte** — wrapper, likely minimal.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/skills/
git commit -m "feat(theme): theme skills module with CSS variables"
```

---

## Task 6: Memory Module

**Files:**
- Modify: `src/lib/components/memory/MemoryEditor.svelte`
- Modify: `src/lib/components/memory/MemoryList.svelte`
- Modify: `src/lib/components/memory/MemoryModule.svelte`

- [ ] **Step 1: Theme MemoryEditor.svelte**

| Pattern | Replacement |
|---------|------------|
| `text-gray-600` (placeholder text) | `style="color: var(--text-muted)"` |
| `border-gray-800` (header/footer border) | `style="border-color: var(--border-color)"` |
| `text-gray-100` heading | `style="color: var(--text-primary)"` |
| Memory type badges: `bg-blue-900 text-blue-300` | `class="badge badge-info"` |
| Memory type badges: `bg-purple-900 text-purple-300` | `class="badge badge-purple"` |
| Memory type badges: `bg-yellow-900 text-yellow-300` | `class="badge badge-warning"` |
| Memory type badges: `bg-gray-700 text-gray-300` | `class="badge badge-neutral"` |
| Unsaved badge: `bg-orange-900 text-orange-300` | `class="badge badge-warning"` |
| `text-gray-500` filename | `style="color: var(--text-muted)"` |
| `text-gray-400` description | `style="color: var(--text-secondary)"` |
| Save button: `bg-blue-600 text-white hover:bg-blue-500` | `class="btn-primary"` |
| Save disabled: `bg-gray-700 text-gray-500` | `class="btn-primary" disabled` |
| Delete button: `text-red-400 hover:bg-red-900/50 hover:text-red-300` | `class="btn-danger-ghost"` |
| Textarea: `border-gray-700 bg-gray-950 text-gray-200 focus:border-gray-600` | `class="code-block" style="resize: none"` + override to use `--bg-code` |
| `text-gray-500` loading | `style="color: var(--text-muted)"` |
| `text-red-400` error | `style="color: var(--status-error-text)"` |

- [ ] **Step 2: Theme MemoryList.svelte**

| Pattern | Replacement |
|---------|------------|
| `border-gray-800` borders | `style="border-color: var(--border-color)"` |
| `text-gray-600` empty state | `style="color: var(--text-muted)"` |
| `text-gray-300` dropdown text | `style="color: var(--text-secondary)"` |
| `bg-gray-800` dropdown bg | `style="background-color: var(--bg-tertiary)"` |
| Selected item: `bg-gray-800 text-white` | `style="background-color: var(--accent-bg); color: var(--text-primary)"` |
| Unselected item: `text-gray-400 hover:bg-gray-800/50 hover:text-gray-200` | `style="color: var(--text-secondary)"` |
| Memory type badges (same as editor) | `.badge .badge-info` etc. |
| Dirty dot: `bg-orange-500` | `style="background-color: var(--dirty-dot)"` |
| `text-gray-500` sub text | `style="color: var(--text-muted)"` |

- [ ] **Step 3: MemoryModule.svelte** — wrapper, likely minimal.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/memory/
git commit -m "feat(theme): theme memory module with CSS variables"
```

---

## Task 7: CLAUDE.md Module

**Files:**
- Modify: `src/lib/components/claudemd/ClaudeMdEditor.svelte`
- Modify: `src/lib/components/claudemd/ClaudeMdList.svelte`
- Modify: `src/lib/components/claudemd/ClaudeMdModule.svelte`

Mirror Memory module patterns exactly (same editor + list structure).

- [ ] **Step 1: Theme ClaudeMdEditor.svelte** — same pattern as MemoryEditor
- [ ] **Step 2: Theme ClaudeMdList.svelte** — same pattern as MemoryList
- [ ] **Step 3: ClaudeMdModule.svelte** — wrapper
- [ ] **Step 4: Commit**

```bash
git add src/lib/components/claudemd/
git commit -m "feat(theme): theme claudemd module with CSS variables"
```

---

## Task 8: MCP Module

**Files:**
- Modify: `src/lib/components/mcp/McpServerEditor.svelte`
- Modify: `src/lib/components/mcp/McpServerList.svelte`
- Modify: `src/lib/components/mcp/McpModule.svelte`

- [ ] **Step 1: Theme McpServerEditor.svelte**

Key patterns:
| Pattern | Replacement |
|---------|------------|
| Error/success containers | `.alert-error` / `.alert-success` |
| Input fields with all the color classes | `.input-base` |
| Radio buttons: `accent-blue-500` | `style="accent-color: var(--accent-primary)"` |
| Submit button | `.btn-primary` |
| Env add/remove buttons | `.btn-secondary` / `.btn-danger-ghost` |
| Label text colors | `style="color: var(--text-muted)"` or `var(--text-secondary)` |

- [ ] **Step 2: Theme McpServerList.svelte**

| Pattern | Replacement |
|---------|------------|
| Error container | `.alert-error` |
| Loading spinner: `border-gray-600 border-t-blue-400` | `style="border-color: var(--border-strong); border-top-color: var(--accent-primary)"` |
| Server cards: `border-gray-800 bg-gray-900 hover:border-gray-700` | `.card` |
| Transport badges: `bg-blue-900 text-blue-300` etc. | `.badge .badge-info` / `.badge-success` / `.badge-warning` |
| Status dots: `bg-green-500` / `bg-red-500` / `bg-yellow-500` | `style="background-color: var(--status-success-text)"` etc. |
| Remove button | `.btn-danger-ghost` |

- [ ] **Step 3: McpModule.svelte** — wrapper
- [ ] **Step 4: Commit**

```bash
git add src/lib/components/mcp/
git commit -m "feat(theme): theme mcp module with CSS variables"
```

---

## Task 9: Remaining Modules

**Files:**
- Modify: `src/lib/components/effective/EffectiveConfigView.svelte`
- Modify: `src/lib/components/launcher/LauncherView.svelte`
- Modify: `src/lib/components/appsettings/AppSettingsView.svelte`

- [ ] **Step 1: Theme EffectiveConfigView.svelte**

This file partially uses `dark:` variants on source badges. Replace ALL color classes (including the `dark:` variants) with CSS variables:
| Pattern | Replacement |
|---------|------------|
| Section cards: `border-gray-700 bg-gray-900` | `.card` |
| `hover:bg-gray-800/50` | `var(--bg-card-hover)` |
| Source badges: `bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300` | `.badge .badge-info` |
| Source badges: `bg-green-100 ... dark:bg-green-900` | `.badge .badge-success` |
| Source badges: `bg-yellow-100 ... dark:bg-yellow-900` | `.badge .badge-warning` |
| Source badges: `bg-red-100 ... dark:bg-red-900` | `.badge .badge-error` |
| Code blocks: `bg-gray-950 text-gray-300` | `.code-block` |

- [ ] **Step 2: Theme LauncherView.svelte**

65 hardcoded instances. Key patterns:
| Pattern | Replacement |
|---------|------------|
| Cards: `border-gray-700 bg-gray-900` | `.card` |
| Config boxes: `bg-gray-800` | `style="background-color: var(--bg-tertiary)"` |
| Labels/text: `text-gray-200/300/400/500/600` | Map to `--text-primary`/`--text-secondary`/`--text-muted` |
| Input fields | `.input-base` |
| Checkboxes: `text-blue-500 focus:ring-blue-500` | `style="accent-color: var(--accent-primary)"` |
| Add env button: `bg-gray-700 hover:bg-gray-600` | `.btn-secondary` |
| Remove button: `text-red-400 hover:bg-red-900/30` | `.btn-danger-ghost` |
| Success message: `text-green-300 border-green-700 bg-green-900/20` | `.alert-success` |
| Error message: `text-red-300 border-red-700 bg-red-900/20` | `.alert-error` |
| Launch button: `bg-blue-600 hover:bg-blue-500 text-white` | `.btn-primary` |

- [ ] **Step 3: Theme AppSettingsView.svelte**

Likely minimal — check for hardcoded colors and replace. It mostly uses `bg-gray-800` for inputs.

- [ ] **Step 4: Commit**

```bash
pnpm exec svelte-check 2>&1 | tail -3
git add src/lib/components/effective/ src/lib/components/launcher/ src/lib/components/appsettings/
git commit -m "feat(theme): theme effective config, launcher, app settings"
```

---

## Task 10: App.svelte Refinement + Sub-panel Lists in App.svelte

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Audit App.svelte for remaining hardcoded colors**

App.svelte already uses CSS variables for the main framework (sidebar, sub-panel, detail). But the inline sub-panel sections (Settings, Plugins, MCP, AppSettings sub-navigation lists) may still have hardcoded colors like:
- `bg-gray-800 text-white` (selected item)
- `text-gray-400 hover:bg-gray-800/50 hover:text-gray-200` (unselected item)
- `text-gray-600` (description text)

Replace these with CSS variable inline styles matching the SkillList pattern.

- [ ] **Step 2: Commit**

```bash
git add src/App.svelte
git commit -m "feat(theme): theme App.svelte sub-panel lists"
```

---

## Task 11: Final Verification

- [ ] **Step 1: Run svelte-check**

```bash
pnpm exec svelte-check 2>&1 | tail -5
```

Expected: 0 errors (same or fewer warnings).

- [ ] **Step 2: Run cargo test**

```bash
cargo test --workspace 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 3: Grep for remaining hardcoded gray classes**

```bash
grep -rn "bg-gray-\|text-gray-\|border-gray-" src/lib/components/ src/App.svelte | grep -v node_modules | wc -l
```

Expected: 0 or near-0 (a few may remain in transition/animation utility classes which don't need theming).

- [ ] **Step 4: Visual verification in both themes**

Start dev server (`pnpm tauri dev`) and test:
1. Switch to Light theme (App Settings → Appearance → 浅色) — all modules should be readable with warm off-white backgrounds
2. Switch to Dark theme — all modules should have deep navy-black backgrounds
3. Verify every module: Settings, Plugins, Skills, Memory, CLAUDE.md, MCP, Effective Config, Launcher, App Settings
4. Check interactive states: hover, focus, selected items, disabled buttons
5. Check status colors: error alerts, success badges, warning badges

- [ ] **Step 5: Final commit tag**

```bash
git tag v1.0.0-alpha.2
```
