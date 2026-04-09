# Phase 6: Toast, Skill Preview, CLAUDE.md Editor — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add global toast notifications, replace the skill content placeholder with real file content, and create a CLAUDE.md editor navigation module.

**Architecture:** Three independent features sharing the same codebase patterns. Toast is pure frontend (store + component). Skill preview adds one backend endpoint and updates an existing component. CLAUDE.md editor is a full new module (Rust types + API handler + frontend store + 3 components) following the Memory module pattern.

**Tech Stack:** Svelte 5 (runes), TypeScript, Rust/axum, Tailwind CSS 4, CSS variables

---

## File Structure

### New Files

| File | Responsibility |
|------|----------------|
| `src/lib/stores/toast.svelte.ts` | Toast notification queue, auto-dismiss timers |
| `src/lib/components/shared/Toast.svelte` | Fixed-position toast stack UI, animations |
| `crates/claude-types/src/claudemd.rs` | ClaudeMdFile, ClaudeMdFileDetail, UpdateClaudeMdRequest types |
| `crates/claude-daemon/src/api/claudemd.rs` | CRUD handlers for CLAUDE.md files |
| `src/lib/stores/claudemd.svelte.ts` | CLAUDE.md file list, active file, save/delete |
| `src/lib/components/claudemd/ClaudeMdList.svelte` | Sub-panel file list grouped by scope |
| `src/lib/components/claudemd/ClaudeMdEditor.svelte` | Detail panel textarea editor with dirty tracking |
| `src/lib/components/claudemd/ClaudeMdModule.svelte` | Module wrapper |

### Modified Files

| File | Change |
|------|--------|
| `src/App.svelte` | Render Toast, add CLAUDE.md nav + panels + data loading |
| `src/lib/api/types.ts` | Add SkillContentResponse, ClaudeMdFile, ClaudeMdFileDetail types |
| `src/lib/api/client.ts` | Add getSkillContent, CLAUDE.md CRUD methods |
| `src/lib/stores/skills.svelte.ts` | Add skillContent, contentLoading, loadSkillContent |
| `src/lib/stores/config.svelte.ts` | Add toastStore calls in save |
| `src/lib/stores/memory.svelte.ts` | Add toastStore calls in save/delete |
| `src/lib/stores/connection.svelte.ts` | Add toastStore calls in connect, add claudeMdStore.reset() |
| `src/lib/components/skills/SkillPreview.svelte` | Replace placeholder with real content |
| `crates/claude-types/src/lib.rs` | Add `pub mod claudemd` |
| `crates/claude-types/src/skills.rs` | Add SkillContentResponse |
| `crates/claude-daemon/src/api/mod.rs` | Add `pub mod claudemd` |
| `crates/claude-daemon/src/api/skills.rs` | Add get_skill_content handler, extract find_skill_path |
| `crates/claude-daemon/src/server.rs` | Register new routes |

---

## Task 1: Toast Store

**Files:**
- Create: `src/lib/stores/toast.svelte.ts`

- [ ] **Step 1: Create the toast store**

```typescript
// src/lib/stores/toast.svelte.ts

type ToastType = "success" | "error" | "warning" | "info";

interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration: number;
}

class ToastStore {
  toasts = $state<Toast[]>([]);

  show(message: string, type: ToastType = "info", duration = 4000): void {
    const id = crypto.randomUUID();
    this.toasts = [...this.toasts, { id, type, message, duration }];
    if (duration > 0) {
      setTimeout(() => this.dismiss(id), duration);
    }
  }

  success(message: string, duration = 4000): void {
    this.show(message, "success", duration);
  }

  error(message: string, duration = 6000): void {
    this.show(message, "error", duration);
  }

  warning(message: string, duration = 5000): void {
    this.show(message, "warning", duration);
  }

  info(message: string, duration = 4000): void {
    this.show(message, "info", duration);
  }

  dismiss(id: string): void {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  reset(): void {
    this.toasts = [];
  }
}

export const toastStore = new ToastStore();
```

- [ ] **Step 2: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds with no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/toast.svelte.ts
git commit -m "feat(toast): add toast notification store"
```

---

## Task 2: Toast Component

**Files:**
- Create: `src/lib/components/shared/Toast.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create the Toast component**

```svelte
<!-- src/lib/components/shared/Toast.svelte -->
<script lang="ts">
  import { fly } from "svelte/transition";
  import { toastStore } from "$lib/stores/toast.svelte";

  const iconPaths: Record<string, string> = {
    success:
      "M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
    error:
      "M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z",
    warning:
      "M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.008v.008H12v-.008Z",
    info: "M11.25 11.25l.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z",
  };

  const borderColors: Record<string, string> = {
    success: "#238636",
    error: "#f85149",
    warning: "#d29922",
    info: "#58a6ff",
  };

  const iconColors: Record<string, string> = {
    success: "#3fb950",
    error: "#f85149",
    warning: "#d29922",
    info: "#58a6ff",
  };
</script>

{#if toastStore.toasts.length > 0}
  <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2" style="max-width: 360px;">
    {#each toastStore.toasts as toast (toast.id)}
      <div
        class="flex items-center gap-3 rounded-lg px-4 py-3 shadow-lg"
        style="background-color: var(--bg-secondary); border: 1px solid {borderColors[toast.type]};"
        transition:fly={{ x: 100, duration: 300 }}
      >
        <svg
          class="h-5 w-5 flex-shrink-0"
          fill="none"
          stroke={iconColors[toast.type]}
          viewBox="0 0 24 24"
          stroke-width="1.5"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d={iconPaths[toast.type]}
          />
        </svg>
        <span class="flex-1 text-sm" style="color: var(--text-primary);">
          {toast.message}
        </span>
        <button
          class="flex-shrink-0 text-lg leading-none opacity-50 transition-opacity hover:opacity-100"
          style="color: var(--text-secondary);"
          onclick={() => toastStore.dismiss(toast.id)}
        >
          &times;
        </button>
      </div>
    {/each}
  </div>
{/if}
```

- [ ] **Step 2: Add Toast to App.svelte**

In `src/App.svelte`, add the import at the top (after the existing component imports around line 24):

```typescript
import Toast from "$lib/components/shared/Toast.svelte";
```

Add `<Toast />` right before the closing `</div>` of the root container (before line 485):

```svelte
    {/if}
  </div>
  <Toast />
</div>
```

Note: `<Toast />` must be OUTSIDE the `{#if connectionStore.status ...}` block and OUTSIDE the body `<div>`, but INSIDE the root `<div>`. This ensures toasts render above all panels regardless of connection state.

- [ ] **Step 3: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/shared/Toast.svelte src/App.svelte
git commit -m "feat(toast): add Toast component with fly animation"
```

---

## Task 3: Toast Integration into Existing Stores

**Files:**
- Modify: `src/lib/stores/config.svelte.ts`
- Modify: `src/lib/stores/memory.svelte.ts`
- Modify: `src/lib/stores/connection.svelte.ts`

- [ ] **Step 1: Integrate toast into config store**

In `src/lib/stores/config.svelte.ts`, add import at line 1:

```typescript
import { toastStore } from "./toast.svelte";
```

In the `save` method, add `toastStore.success` after `this.isDirty = false;` (line 70) and `toastStore.error` in the catch block (line 72):

Replace lines 69-73:
```typescript
      this.isDirty = false;
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save";
      throw e;
```

With:
```typescript
      this.isDirty = false;
      toastStore.success("Settings saved");
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save";
      toastStore.error(this.error);
      throw e;
```

- [ ] **Step 2: Integrate toast into memory store**

In `src/lib/stores/memory.svelte.ts`, add import at line 1:

```typescript
import { toastStore } from "./toast.svelte";
```

In `saveFile`, after `this.activeFile = { ...this.activeFile, content };` (line 64), add:

```typescript
      toastStore.success("File saved");
```

In `saveFile` catch block (line 67), add after the error assignment:

```typescript
      toastStore.error(this.error);
```

In `deleteFile`, after `this.activeFile = null;` (line 81), add:

```typescript
      toastStore.success("File deleted");
```

In `deleteFile` catch block (line 83), add after the error assignment:

```typescript
      toastStore.error(this.error);
```

- [ ] **Step 3: Integrate toast into connection store**

In `src/lib/stores/connection.svelte.ts`, add import at line 1:

```typescript
import { toastStore } from "./toast.svelte";
```

In `connect`, after `this.status = "connected";` (line 43), add:

```typescript
      toastStore.info("Connected to daemon");
```

In `connect` catch block, after `this.error = ...` (line 46), add:

```typescript
      toastStore.error("Connection failed: " + this.error);
```

- [ ] **Step 4: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds with no errors.

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/config.svelte.ts src/lib/stores/memory.svelte.ts src/lib/stores/connection.svelte.ts
git commit -m "feat(toast): integrate toast notifications into config, memory, and connection stores"
```

---

## Task 4: Skill Content — Backend Type and Handler

**Files:**
- Modify: `crates/claude-types/src/skills.rs`
- Modify: `crates/claude-daemon/src/api/skills.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Add SkillContentResponse type**

In `crates/claude-types/src/skills.rs`, append after the `SkillInfo` struct (after line 20):

```rust
/// Response for GET /api/v1/skills/{id}/content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContentResponse {
    pub id: String,
    pub content: String,
}
```

- [ ] **Step 2: Extract find_skill_path helper and add get_skill_content handler**

In `crates/claude-daemon/src/api/skills.rs`, add imports at the top. Replace lines 1-8:

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::{Extension, Json, extract::Path as AxumPath, http::StatusCode};
use claude_types::{
    api::ErrorResponse,
    plugins::InstalledPluginsFile,
    skills::{SkillContentResponse, SkillInfo},
};

use crate::state::AppState;
```

Add this helper function after the `scan_skills_dir` function (after line 174), before `list_skills`:

```rust
/// Find the filesystem path of a skill's SKILL.md file by its ID.
///
/// Searches user skills first, then plugin skills. Returns the first match.
fn find_skill_path(claude_home: &Path, skill_id: &str) -> Option<PathBuf> {
    // 1. Check user skills
    let user_skill = claude_home.join("skills").join(skill_id).join("SKILL.md");
    if user_skill.exists() {
        return Some(user_skill);
    }

    // 2. Check plugin skills
    let plugins_dir = claude_home.join("plugins");
    let installed = read_installed_plugins(&plugins_dir);

    for (_marketplace_id, plugins) in &installed.plugins {
        for plugin in plugins {
            let plugin_skill = std::path::PathBuf::from(&plugin.install_path)
                .join("skills")
                .join(skill_id)
                .join("SKILL.md");
            if plugin_skill.exists() {
                return Some(plugin_skill);
            }
        }
    }

    None
}
```

Add the handler function after `list_skills` (after line 205):

```rust
// ---------------------------------------------------------------------------
// GET /api/v1/skills/{id}/content
// ---------------------------------------------------------------------------

pub async fn get_skill_content(
    Extension(state): Extension<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<SkillContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let claude_home = &state.inner.claude_home;

    let skill_path = find_skill_path(claude_home, &id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "SKILL_NOT_FOUND".to_string(),
                message: format!("Skill '{}' not found", id),
                validation_errors: vec![],
            }),
        )
    })?;

    let content = std::fs::read_to_string(&skill_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "READ_ERROR".to_string(),
                message: format!("Failed to read skill file: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(Json(SkillContentResponse { id, content }))
}
```

- [ ] **Step 3: Register the route**

In `crates/claude-daemon/src/server.rs`, update the skills import at line 26. Change:

```rust
        skills::list_skills,
```

To:

```rust
        skills::{get_skill_content, list_skills},
```

Add the new route after line 77 (`.route("/api/v1/skills", get(list_skills))`):

```rust
        .route("/api/v1/skills/{id}/content", get(get_skill_content))
```

- [ ] **Step 4: Verify Rust compiles and tests pass**

Run: `cargo build -p claude-daemon && cargo test --workspace`
Expected: Build succeeds, all existing tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/claude-types/src/skills.rs crates/claude-daemon/src/api/skills.rs crates/claude-daemon/src/server.rs
git commit -m "feat(skills): add GET /api/v1/skills/{id}/content endpoint"
```

---

## Task 5: Skill Content — Backend Test

**Files:**
- Modify: `crates/claude-daemon/tests/api_test.rs`

- [ ] **Step 1: Write the test**

Append to `crates/claude-daemon/tests/api_test.rs`:

```rust
// ---------------------------------------------------------------------------
// Skill content endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_skill_content_returns_file() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    // Create a skill directory with SKILL.md
    let skills_dir = dir.path().join("skills").join("my-test-skill");
    std::fs::create_dir_all(&skills_dir).unwrap();
    std::fs::write(
        skills_dir.join("SKILL.md"),
        "---\nname: my-test-skill\ndescription: A test skill\n---\n\n# My Test Skill\n\nThis is the content.",
    )
    .unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{port}/api/v1/skills/my-test-skill/content"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "my-test-skill");
    assert!(body["content"].as_str().unwrap().contains("# My Test Skill"));
}

#[tokio::test]
async fn get_skill_content_not_found() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{port}/api/v1/skills/nonexistent-skill/content"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test --workspace`
Expected: All tests pass, including the two new ones.

- [ ] **Step 3: Commit**

```bash
git add crates/claude-daemon/tests/api_test.rs
git commit -m "test(skills): add tests for skill content endpoint"
```

---

## Task 6: Skill Content — Frontend

**Files:**
- Modify: `src/lib/api/types.ts`
- Modify: `src/lib/api/client.ts`
- Modify: `src/lib/stores/skills.svelte.ts`
- Modify: `src/lib/components/skills/SkillPreview.svelte`

- [ ] **Step 1: Add SkillContentResponse type**

In `src/lib/api/types.ts`, add after the `SkillInfo` interface (after line 249):

```typescript
export interface SkillContentResponse {
  id: string;
  content: string;
}
```

- [ ] **Step 2: Add API client method**

In `src/lib/api/client.ts`, add the import for the new type. Update line 18 to include:

```typescript
  SkillContentResponse,
```

Add the method after `listSkills()` (after line 221):

```typescript
  getSkillContent(id: string): Promise<SkillContentResponse> {
    return this.fetch<SkillContentResponse>(
      `/api/v1/skills/${encodeURIComponent(id)}/content`
    );
  }
```

- [ ] **Step 3: Update skills store**

Replace the entire contents of `src/lib/stores/skills.svelte.ts` with:

```typescript
import { connectionStore } from "./connection.svelte";
import { toastStore } from "./toast.svelte";
import type { SkillInfo } from "$lib/api/types";

class SkillsStore {
  skills = $state<SkillInfo[]>([]);
  selectedSkillId = $state<string | null>(null);
  skillContent = $state<string | null>(null);
  contentLoading = $state(false);
  loading = $state(false);
  error = $state<string>("");

  get selectedSkill(): SkillInfo | undefined {
    return this.skills.find((s) => s.id === this.selectedSkillId);
  }

  async loadSkills() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    try {
      this.skills = await client.listSkills();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load skills";
    } finally {
      this.loading = false;
    }
  }

  selectSkill(id: string | null) {
    this.selectedSkillId = id;
    this.skillContent = null;
    if (id) {
      void this.loadSkillContent(id);
    }
  }

  async loadSkillContent(id: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.contentLoading = true;
    try {
      const res = await client.getSkillContent(id);
      this.skillContent = res.content;
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Failed to load skill content";
      toastStore.error(msg);
      this.skillContent = null;
    } finally {
      this.contentLoading = false;
    }
  }

  reset(): void {
    this.skills = [];
    this.selectedSkillId = null;
    this.skillContent = null;
    this.contentLoading = false;
    this.loading = false;
    this.error = "";
  }
}

export const skillsStore = new SkillsStore();
```

- [ ] **Step 4: Update SkillPreview component**

In `src/lib/components/skills/SkillPreview.svelte`, replace lines 66-77 (the SKILL.md placeholder block):

```svelte
      <!-- SKILL.md content placeholder -->
      <div>
        <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">
          SKILL.md
        </h3>
        <div class="rounded-lg border border-gray-800 bg-gray-950">
          <pre class="overflow-auto p-4 text-xs leading-relaxed text-gray-300 whitespace-pre-wrap">{skill.path}</pre>
          <p class="border-t border-gray-800 px-4 py-2 text-xs text-gray-600">
            Content preview requires daemon file-read support (not yet available).
          </p>
        </div>
      </div>
```

With:

```svelte
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
```

- [ ] **Step 5: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds with no errors.

- [ ] **Step 6: Commit**

```bash
git add src/lib/api/types.ts src/lib/api/client.ts src/lib/stores/skills.svelte.ts src/lib/components/skills/SkillPreview.svelte
git commit -m "feat(skills): display actual SKILL.md content in preview"
```

---

## Task 7: CLAUDE.md — Backend Types

**Files:**
- Create: `crates/claude-types/src/claudemd.rs`
- Modify: `crates/claude-types/src/lib.rs`

- [ ] **Step 1: Create the types module**

```rust
// crates/claude-types/src/claudemd.rs

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CLAUDE.md API types
// ---------------------------------------------------------------------------

/// A CLAUDE.md file entry (returned by list endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeMdFile {
    pub id: String,
    pub scope: String,
    pub filename: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    pub exists: bool,
}

/// Full content of a CLAUDE.md file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeMdFileDetail {
    pub id: String,
    pub scope: String,
    pub filename: String,
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

/// Request body for PUT /api/v1/claudemd/{id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClaudeMdRequest {
    pub content: String,
}
```

- [ ] **Step 2: Register the module**

In `crates/claude-types/src/lib.rs`, add after `pub mod api;` (line 1):

```rust
pub mod claudemd;
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cargo build -p claude-types`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add crates/claude-types/src/claudemd.rs crates/claude-types/src/lib.rs
git commit -m "feat(types): add CLAUDE.md API types"
```

---

## Task 8: CLAUDE.md — Backend API Handler

**Files:**
- Create: `crates/claude-daemon/src/api/claudemd.rs`
- Modify: `crates/claude-daemon/src/api/mod.rs`
- Modify: `crates/claude-daemon/src/server.rs`

- [ ] **Step 1: Create the API handler**

```rust
// crates/claude-daemon/src/api/claudemd.rs

use std::path::PathBuf;

use axum::{Extension, Json, extract::Path, http::StatusCode};
use claude_types::{
    api::ErrorResponse,
    claudemd::{ClaudeMdFile, ClaudeMdFileDetail, UpdateClaudeMdRequest},
};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// ID → filesystem path resolution
// ---------------------------------------------------------------------------

/// Resolve a CLAUDE.md id to a filesystem path.
///
/// Valid formats:
/// - `"global"` → `{claude_home}/CLAUDE.md`
/// - `"project:{project_id}"` → `{project_path}/.claude/CLAUDE.md`
async fn resolve_claudemd_path(
    state: &AppState,
    id: &str,
) -> Result<PathBuf, (StatusCode, Json<ErrorResponse>)> {
    if id == "global" {
        return Ok(state.inner.claude_home.join("CLAUDE.md"));
    }

    if let Some(project_id) = id.strip_prefix("project:") {
        let projects = state.inner.projects.read().await;
        let project = projects.iter().find(|p| p.id == project_id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "PROJECT_NOT_FOUND".to_string(),
                    message: format!("Project '{}' not found", project_id),
                    validation_errors: vec![],
                }),
            )
        })?;
        return Ok(project.path.join(".claude").join("CLAUDE.md"));
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: "INVALID_ID".to_string(),
            message: format!("Invalid CLAUDE.md id format: '{}'", id),
            validation_errors: vec![],
        }),
    ))
}

// ---------------------------------------------------------------------------
// GET /api/v1/claudemd
// ---------------------------------------------------------------------------

pub async fn list_claudemd_files(
    Extension(state): Extension<AppState>,
) -> Json<Vec<ClaudeMdFile>> {
    let mut result = Vec::new();

    // 1. Global CLAUDE.md
    let global_path = state.inner.claude_home.join("CLAUDE.md");
    result.push(ClaudeMdFile {
        id: "global".to_string(),
        scope: "global".to_string(),
        filename: "CLAUDE.md".to_string(),
        path: global_path.to_string_lossy().into_owned(),
        project_id: None,
        project_name: None,
        exists: global_path.exists(),
    });

    // 2. Per-project CLAUDE.md
    let projects = state.inner.projects.read().await;
    for project in projects.iter() {
        let project_path = project.path.join(".claude").join("CLAUDE.md");
        let id = format!("project:{}", project.id);
        result.push(ClaudeMdFile {
            id,
            scope: "project".to_string(),
            filename: "CLAUDE.md".to_string(),
            path: project_path.to_string_lossy().into_owned(),
            project_id: Some(project.id.clone()),
            project_name: Some(project.name.clone()),
            exists: project_path.exists(),
        });
    }

    Json(result)
}

// ---------------------------------------------------------------------------
// GET /api/v1/claudemd/{id}
// ---------------------------------------------------------------------------

pub async fn get_claudemd_file(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClaudeMdFileDetail>, (StatusCode, Json<ErrorResponse>)> {
    let file_path = resolve_claudemd_path(&state, &id).await?;

    let content = std::fs::read_to_string(&file_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: format!("CLAUDE.md not found at '{}'", file_path.display()),
                validation_errors: vec![],
            }),
        )
    })?;

    let scope = if id == "global" { "global" } else { "project" };
    let project_id = id.strip_prefix("project:").map(|s| s.to_string());

    Ok(Json(ClaudeMdFileDetail {
        id,
        scope: scope.to_string(),
        filename: "CLAUDE.md".to_string(),
        path: file_path.to_string_lossy().into_owned(),
        content,
        project_id,
    }))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/claudemd/{id}
// ---------------------------------------------------------------------------

pub async fn put_claudemd_file(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateClaudeMdRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let file_path = resolve_claudemd_path(&state, &id).await?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "DIR_ERROR".to_string(),
                    message: format!("Failed to create directory: {}", e),
                    validation_errors: vec![],
                }),
            )
        })?;
    }

    claude_config::write::atomic_write(&file_path, body.content.as_bytes()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "WRITE_ERROR".to_string(),
                message: format!("Failed to write CLAUDE.md: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/claudemd/{id}
// ---------------------------------------------------------------------------

pub async fn delete_claudemd_file(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let file_path = resolve_claudemd_path(&state, &id).await?;

    std::fs::remove_file(&file_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: format!("CLAUDE.md not found at '{}'", file_path.display()),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
```

- [ ] **Step 2: Register the module**

In `crates/claude-daemon/src/api/mod.rs`, add after line 1 (`pub mod config;`):

```rust
pub mod claudemd;
```

- [ ] **Step 3: Register the routes**

In `crates/claude-daemon/src/server.rs`, add the import. After line 15 (`launcher::launch_claude,`):

```rust
        claudemd::{
            delete_claudemd_file, get_claudemd_file, list_claudemd_files, put_claudemd_file,
        },
```

Add routes after the skills route (after line 77, `.route("/api/v1/skills", get(list_skills))`). If task 4 was already applied, add after the skills content route:

```rust
        // CLAUDE.md routes
        .route("/api/v1/claudemd", get(list_claudemd_files))
        .route(
            "/api/v1/claudemd/{id}",
            get(get_claudemd_file).put(put_claudemd_file).delete(delete_claudemd_file),
        )
```

- [ ] **Step 4: Verify Rust compiles and tests pass**

Run: `cargo build -p claude-daemon && cargo test --workspace`
Expected: Build succeeds, all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/claude-daemon/src/api/claudemd.rs crates/claude-daemon/src/api/mod.rs crates/claude-daemon/src/server.rs
git commit -m "feat(claudemd): add CLAUDE.md CRUD API endpoints"
```

---

## Task 9: CLAUDE.md — Backend Tests

**Files:**
- Modify: `crates/claude-daemon/tests/api_test.rs`

- [ ] **Step 1: Write the tests**

Append to `crates/claude-daemon/tests/api_test.rs`:

```rust
// ---------------------------------------------------------------------------
// CLAUDE.md endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_claudemd_includes_global() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/claudemd"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    // Should always include the global entry (even if file doesn't exist)
    assert!(body.iter().any(|f| f["id"] == "global"));
    // Global file doesn't exist in temp dir
    let global = body.iter().find(|f| f["id"] == "global").unwrap();
    assert_eq!(global["exists"], false);
}

#[tokio::test]
async fn claudemd_crud_lifecycle() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}/api/v1/claudemd");

    // GET global — should 404 since file doesn't exist yet
    let resp = client
        .get(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    // PUT global — create the file
    let resp = client
        .put(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(r#"{"content":"# Test CLAUDE.md\n\nHello world."}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify file exists on disk
    assert!(dir.path().join("CLAUDE.md").exists());

    // GET global — should succeed
    let resp = client
        .get(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["content"].as_str().unwrap().contains("Test CLAUDE.md"));

    // DELETE global
    let resp = client
        .delete(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Verify file is gone
    assert!(!dir.path().join("CLAUDE.md").exists());
}

#[tokio::test]
async fn claudemd_invalid_id_returns_400() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{port}/api/v1/claudemd/invalid-format"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test --workspace`
Expected: All tests pass, including the three new ones.

- [ ] **Step 3: Commit**

```bash
git add crates/claude-daemon/tests/api_test.rs
git commit -m "test(claudemd): add tests for CLAUDE.md CRUD endpoints"
```

---

## Task 10: CLAUDE.md — Frontend Types and API Client

**Files:**
- Modify: `src/lib/api/types.ts`
- Modify: `src/lib/api/client.ts`

- [ ] **Step 1: Add TypeScript types**

In `src/lib/api/types.ts`, add after the `SkillContentResponse` interface (added in Task 6):

```typescript
// ---------------------------------------------------------------------------
// CLAUDE.md
// ---------------------------------------------------------------------------

export interface ClaudeMdFile {
  id: string;
  scope: "global" | "project";
  filename: string;
  path: string;
  projectId?: string;
  projectName?: string;
  exists: boolean;
}

export interface ClaudeMdFileDetail {
  id: string;
  scope: "global" | "project";
  filename: string;
  path: string;
  content: string;
  projectId?: string;
}
```

- [ ] **Step 2: Add API client methods**

In `src/lib/api/client.ts`, add to the imports (around line 1-19):

```typescript
  ClaudeMdFile,
  ClaudeMdFileDetail,
```

Add the methods after the Skills section (after `getSkillContent`, added in Task 6):

```typescript
  // -------------------------------------------------------------------------
  // CLAUDE.md endpoints
  // -------------------------------------------------------------------------

  listClaudeMdFiles(): Promise<ClaudeMdFile[]> {
    return this.fetch<ClaudeMdFile[]>("/api/v1/claudemd");
  }

  getClaudeMdFile(id: string): Promise<ClaudeMdFileDetail> {
    return this.fetch<ClaudeMdFileDetail>(
      `/api/v1/claudemd/${encodeURIComponent(id)}`
    );
  }

  async updateClaudeMdFile(id: string, content: string): Promise<void> {
    await this.fetch(
      `/api/v1/claudemd/${encodeURIComponent(id)}`,
      {
        method: "PUT",
        body: JSON.stringify({ content }),
      }
    );
  }

  async deleteClaudeMdFile(id: string): Promise<void> {
    await this.fetch(
      `/api/v1/claudemd/${encodeURIComponent(id)}`,
      { method: "DELETE" }
    );
  }
```

- [ ] **Step 3: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/lib/api/types.ts src/lib/api/client.ts
git commit -m "feat(claudemd): add CLAUDE.md TypeScript types and API client methods"
```

---

## Task 11: CLAUDE.md — Frontend Store

**Files:**
- Create: `src/lib/stores/claudemd.svelte.ts`

- [ ] **Step 1: Create the store**

```typescript
// src/lib/stores/claudemd.svelte.ts

import { connectionStore } from "./connection.svelte";
import { toastStore } from "./toast.svelte";
import type { ClaudeMdFile, ClaudeMdFileDetail } from "$lib/api/types";

class ClaudeMdStore {
  files = $state<ClaudeMdFile[]>([]);
  activeFile = $state<ClaudeMdFileDetail | null>(null);
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  async loadFiles() {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.files = await client.listClaudeMdFiles();
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load CLAUDE.md files";
    } finally {
      this.loading = false;
    }
  }

  async loadFile(id: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.loading = true;
    this.error = "";
    try {
      this.activeFile = await client.getClaudeMdFile(id);
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to load CLAUDE.md";
      this.activeFile = null;
    } finally {
      this.loading = false;
    }
  }

  async saveFile(id: string, content: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.saving = true;
    this.error = "";
    try {
      await client.updateClaudeMdFile(id, content);
      if (this.activeFile && this.activeFile.id === id) {
        this.activeFile = { ...this.activeFile, content };
      }
      // Update exists flag in file list
      this.files = this.files.map((f) =>
        f.id === id ? { ...f, exists: true } : f
      );
      toastStore.success("CLAUDE.md saved");
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to save CLAUDE.md";
      toastStore.error(this.error);
    } finally {
      this.saving = false;
    }
  }

  async deleteFile(id: string) {
    const client = connectionStore.client;
    if (!client) return;
    this.error = "";
    try {
      await client.deleteClaudeMdFile(id);
      this.files = this.files.map((f) =>
        f.id === id ? { ...f, exists: false } : f
      );
      if (this.activeFile?.id === id) {
        this.activeFile = null;
      }
      toastStore.success("CLAUDE.md deleted");
    } catch (e) {
      this.error = e instanceof Error ? e.message : "Failed to delete CLAUDE.md";
      toastStore.error(this.error);
    }
  }

  selectFile(id: string) {
    const file = this.files.find((f) => f.id === id);
    if (file && file.exists) {
      void this.loadFile(id);
    } else {
      // For non-existent files, set up a blank editor for creation
      this.activeFile = {
        id,
        scope: file?.scope ?? "project",
        filename: "CLAUDE.md",
        path: file?.path ?? "",
        content: "",
        projectId: file?.projectId,
      };
    }
  }

  reset(): void {
    this.files = [];
    this.activeFile = null;
    this.loading = false;
    this.saving = false;
    this.error = "";
  }
}

export const claudeMdStore = new ClaudeMdStore();
```

- [ ] **Step 2: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/claudemd.svelte.ts
git commit -m "feat(claudemd): add CLAUDE.md frontend store"
```

---

## Task 12: CLAUDE.md — Frontend Components

**Files:**
- Create: `src/lib/components/claudemd/ClaudeMdList.svelte`
- Create: `src/lib/components/claudemd/ClaudeMdEditor.svelte`
- Create: `src/lib/components/claudemd/ClaudeMdModule.svelte`

- [ ] **Step 1: Create ClaudeMdList component**

```svelte
<!-- src/lib/components/claudemd/ClaudeMdList.svelte -->
<script lang="ts">
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";

  function scopeBadgeClass(scope: string): string {
    return scope === "global"
      ? "bg-blue-900 text-blue-300"
      : "bg-purple-900 text-purple-300";
  }
</script>

<div class="flex flex-col overflow-hidden h-full">
  <ul class="flex-1 overflow-y-auto py-2">
    {#if claudeMdStore.loading && claudeMdStore.files.length === 0}
      <li class="px-4 py-2 text-xs text-gray-500">Loading...</li>
    {:else if claudeMdStore.files.length === 0}
      <li class="px-4 py-2 text-xs text-gray-600">No CLAUDE.md files found</li>
    {:else}
      <!-- Global section -->
      <li class="px-4 pt-2 pb-1">
        <span class="text-xs font-semibold uppercase tracking-wider text-gray-600">Global</span>
      </li>
      {#each claudeMdStore.files.filter((f) => f.scope === "global") as file (file.id)}
        <li>
          <button
            class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors
              {claudeMdStore.activeFile?.id === file.id
              ? 'bg-gray-800 text-white'
              : file.exists
                ? 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'
                : 'text-gray-600 italic hover:bg-gray-800/50 hover:text-gray-400'}"
            onclick={() => claudeMdStore.selectFile(file.id)}
          >
            <span class="truncate">{file.exists ? "CLAUDE.md" : "CLAUDE.md (create)"}</span>
            <span class="ml-auto flex-shrink-0 rounded px-1.5 py-0.5 text-xs font-medium {scopeBadgeClass(file.scope)}">
              {file.scope === "global" ? "全局" : file.projectName ?? "project"}
            </span>
          </button>
        </li>
      {/each}

      <!-- Project section -->
      {@const projectFiles = claudeMdStore.files.filter((f) => f.scope === "project")}
      {#if projectFiles.length > 0}
        <li class="px-4 pt-4 pb-1">
          <span class="text-xs font-semibold uppercase tracking-wider text-gray-600">Projects</span>
        </li>
        {#each projectFiles as file (file.id)}
          <li>
            <button
              class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm transition-colors
                {claudeMdStore.activeFile?.id === file.id
                ? 'bg-gray-800 text-white'
                : file.exists
                  ? 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'
                  : 'text-gray-600 italic hover:bg-gray-800/50 hover:text-gray-400'}"
              onclick={() => claudeMdStore.selectFile(file.id)}
            >
              <span class="truncate">{file.projectName ?? file.projectId}</span>
              {#if !file.exists}
                <span class="ml-auto flex-shrink-0 text-xs text-gray-600">click to create</span>
              {/if}
            </button>
          </li>
        {/each}
      {/if}
    {/if}
  </ul>

  {#if claudeMdStore.error}
    <div class="px-4 py-2 text-xs text-red-400 border-t border-gray-800">
      {claudeMdStore.error}
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Create ClaudeMdEditor component**

```svelte
<!-- src/lib/components/claudemd/ClaudeMdEditor.svelte -->
<script lang="ts">
  import { claudeMdStore } from "$lib/stores/claudemd.svelte";

  let localContent = $state("");
  let originalContent = $state("");

  $effect(() => {
    const file = claudeMdStore.activeFile;
    if (file) {
      localContent = file.content;
      originalContent = file.content;
    } else {
      localContent = "";
      originalContent = "";
    }
  });

  let isDirty = $derived(localContent !== originalContent);

  async function handleSave() {
    const file = claudeMdStore.activeFile;
    if (!file) return;
    await claudeMdStore.saveFile(file.id, localContent);
    originalContent = localContent;
  }

  async function handleDelete() {
    const file = claudeMdStore.activeFile;
    if (!file) return;
    if (!confirm("Are you sure you want to delete this CLAUDE.md?")) return;
    await claudeMdStore.deleteFile(file.id);
  }

  function scopeBadgeClass(scope: string): string {
    return scope === "global"
      ? "bg-blue-900 text-blue-300"
      : "bg-purple-900 text-purple-300";
  }

  function scopeLabel(scope: string, projectName?: string): string {
    if (scope === "global") return "全局";
    return projectName ?? "project";
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if !claudeMdStore.activeFile}
    <div class="flex flex-1 items-center justify-center">
      <p class="text-sm text-gray-600">Select a CLAUDE.md file to view and edit</p>
    </div>
  {:else}
    {@const file = claudeMdStore.activeFile}
    <!-- Header -->
    <div class="border-b border-gray-800 px-6 py-4">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="flex items-center gap-2">
            <h2 class="truncate text-sm font-semibold text-gray-100">
              {file.filename}
            </h2>
            <span class="flex-shrink-0 rounded px-1.5 py-0.5 text-xs font-medium {scopeBadgeClass(file.scope)}">
              {scopeLabel(file.scope, file.projectId)}
            </span>
            {#if isDirty}
              <span class="flex-shrink-0 rounded bg-orange-900 px-1.5 py-0.5 text-xs font-medium text-orange-300">
                unsaved
              </span>
            {/if}
          </div>
          <p class="mt-0.5 font-mono text-xs text-gray-500">{file.path}</p>
        </div>

        <div class="flex flex-shrink-0 items-center gap-2">
          <button
            class="rounded px-3 py-1.5 text-xs font-medium transition-colors
              {isDirty && !claudeMdStore.saving
              ? 'bg-blue-600 text-white hover:bg-blue-500'
              : 'cursor-not-allowed bg-gray-700 text-gray-500'}"
            disabled={!isDirty || claudeMdStore.saving}
            onclick={handleSave}
          >
            {claudeMdStore.saving ? "Saving..." : "Save"}
          </button>
          {#if file.scope !== "global" || originalContent !== ""}
            <button
              class="rounded px-3 py-1.5 text-xs font-medium text-red-400 transition-colors hover:bg-red-900/50 hover:text-red-300"
              onclick={handleDelete}
            >
              Delete
            </button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Editor -->
    <div class="flex flex-1 flex-col overflow-hidden p-4">
      {#if claudeMdStore.loading}
        <div class="flex flex-1 items-center justify-center">
          <p class="text-sm text-gray-500">Loading...</p>
        </div>
      {:else}
        <textarea
          class="flex-1 resize-none rounded border border-gray-700 bg-gray-950 p-3 font-mono text-xs text-gray-200 leading-relaxed focus:border-gray-600 focus:outline-none"
          bind:value={localContent}
          spellcheck={false}
        ></textarea>
      {/if}
    </div>

    {#if claudeMdStore.error}
      <div class="border-t border-gray-800 px-6 py-2 text-xs text-red-400">
        {claudeMdStore.error}
      </div>
    {/if}
  {/if}
</div>
```

- [ ] **Step 3: Create ClaudeMdModule component**

```svelte
<!-- src/lib/components/claudemd/ClaudeMdModule.svelte -->
<script lang="ts">
  import ClaudeMdEditor from "./ClaudeMdEditor.svelte";
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  <ClaudeMdEditor />
</div>
```

- [ ] **Step 4: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/claudemd/ClaudeMdList.svelte src/lib/components/claudemd/ClaudeMdEditor.svelte src/lib/components/claudemd/ClaudeMdModule.svelte
git commit -m "feat(claudemd): add ClaudeMdList, ClaudeMdEditor, and ClaudeMdModule components"
```

---

## Task 13: CLAUDE.md — App.svelte Integration

**Files:**
- Modify: `src/App.svelte`
- Modify: `src/lib/stores/connection.svelte.ts`

- [ ] **Step 1: Add imports to App.svelte**

In `src/App.svelte`, add these imports after the existing component imports (around line 24):

```typescript
import ClaudeMdList from "$lib/components/claudemd/ClaudeMdList.svelte";
import ClaudeMdModule from "$lib/components/claudemd/ClaudeMdModule.svelte";
import { claudeMdStore } from "$lib/stores/claudemd.svelte";
```

- [ ] **Step 2: Add nav button**

In the `navButtons` array, add between the Memory entry (`id: "M"`) and the MCP entry (`id: "C"`). After line 69 (the Memory button), add:

```typescript
    { id: "D", label: "指令", icon: "M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" },
```

Note: this is the Heroicons "document-text" icon — same icon as Memory but semantically appropriate for instructions. If you want a different icon, the Heroicons "pencil-square" path is: `"m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10"`. Use whichever you prefer — the plan uses document-text to match the mockup.

- [ ] **Step 3: Add helper function**

After `isLauncherModule()` (around line 198), add:

```typescript
  function isClaudeMdModule(): boolean {
    return activeNav === "D";
  }
```

- [ ] **Step 4: Add sub-panel rendering**

In the sub-panel section of `App.svelte`, add a new `{:else if}` block. After the Memory sub-panel block (`{:else if isMemoryModule()}` around line 358-360) and before the MCP block (`{:else if isMcpModule()}`), add:

```svelte
        {:else if isClaudeMdModule()}
          <ClaudeMdList />
```

- [ ] **Step 5: Add detail panel rendering**

In the detail panel section, add a new block. After the Memory detail block (`{:else if isMemoryModule()}` around line 459-461) and before the MCP block (`{:else if isMcpModule()}`), add:

```svelte
          {:else if isClaudeMdModule()}
            <ClaudeMdModule />
```

- [ ] **Step 6: Add data loading on connect**

In the `$effect` block that loads data on connection (around line 148-166), add `claudeMdStore.loadFiles()` after `mcpStore.loadServers();`:

```typescript
      claudeMdStore.loadFiles();
```

- [ ] **Step 7: Add store reset**

In `src/lib/stores/connection.svelte.ts`, add import at line 1:

```typescript
import { claudeMdStore } from "./claudemd.svelte";
```

In `resetAllStores()`, add after `mcpStore.reset();` (line 64):

```typescript
    claudeMdStore.reset();
```

- [ ] **Step 8: Verify frontend compiles**

Run: `pnpm build`
Expected: Build succeeds.

- [ ] **Step 9: Commit**

```bash
git add src/App.svelte src/lib/stores/connection.svelte.ts
git commit -m "feat(claudemd): integrate CLAUDE.md editor into App navigation and layout"
```

---

## Task 14: End-to-End Verification

- [ ] **Step 1: Build daemon**

Run: `cargo build -p claude-daemon`
Expected: Build succeeds.

- [ ] **Step 2: Copy daemon sidecar**

Run: `cp target/debug/claude-daemon src-tauri/binaries/claude-daemon-aarch64-apple-darwin`

- [ ] **Step 3: Run all Rust tests**

Run: `cargo test --workspace`
Expected: All tests pass.

- [ ] **Step 4: Build frontend**

Run: `pnpm build`
Expected: Build succeeds with no errors.

- [ ] **Step 5: Manual verification with `pnpm tauri dev`**

Start: `pnpm tauri dev`

Verify:
1. **Toast**: Save any setting → green "Settings saved" toast appears bottom-right, auto-dismisses after 4s
2. **Toast**: Click × button on a toast → dismisses immediately
3. **Skill Preview**: Click any skill in sidebar → SKILL.md content appears instead of placeholder text
4. **CLAUDE.md**: "指令" navigation item appears in sidebar between "记忆" and "MCP"
5. **CLAUDE.md**: Click "指令" → sub-panel shows global CLAUDE.md + registered projects
6. **CLAUDE.md**: Select global → editor loads content (or blank for new)
7. **CLAUDE.md**: Edit content → "unsaved" badge appears, Save button activates
8. **CLAUDE.md**: Click Save → green toast, dirty state clears
9. **CLAUDE.md**: Click Delete → confirmation dialog → file removed, list updates
