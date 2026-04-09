# Phase 6: Toast Notifications, Skill Content Preview, CLAUDE.md Editor

## Problem

Three gaps in the current GUI:

1. **No operation feedback** — saves, deletes, connection changes produce no visible confirmation. Users can't tell if an action succeeded or failed without checking the console.
2. **Skill preview is a placeholder** — `SkillPreview.svelte:66-77` shows "Content preview requires daemon file-read support (not yet available)" instead of actual SKILL.md content.
3. **No CLAUDE.md management** — CLAUDE.md is Claude Code's most important instruction file, but the GUI has no way to view or edit it. Users must use a text editor.

## Scope

- **Toast notification system** — global toast component + store, integrated into core stores
- **Skill content preview** — new daemon endpoint + frontend update to display SKILL.md content
- **CLAUDE.md editor** — new navigation module (sidebar → sub-panel → detail panel) for editing `~/.claude/CLAUDE.md` (global) and `<project>/.claude/CLAUDE.md` (project-level)
- **Out of scope**: `~/.claude/rules/*.md`, `AGENTS.md`, `GEMINI.md`, markdown rendering, rich text editing

## Design

### 1. Toast Notification System

Pure frontend feature. No backend changes.

#### Toast Store (`src/lib/stores/toast.svelte.ts`)

```typescript
type ToastType = "success" | "error" | "warning" | "info";

interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration: number;
}

class ToastStore {
  toasts = $state<Toast[]>([]);

  show(message: string, type: ToastType = "info", duration = 4000): void;
  success(message: string, duration = 4000): void;
  error(message: string, duration = 6000): void;
  warning(message: string, duration = 5000): void;
  info(message: string, duration = 4000): void;
  dismiss(id: string): void;
  reset(): void;
}
```

- IDs via `crypto.randomUUID()`
- Auto-dismiss via `setTimeout`; error gets longer display (6s)
- No limit on concurrent toasts; natural stacking handles it

#### Toast Component (`src/lib/components/shared/Toast.svelte`)

- Position: `fixed bottom-4 right-4 z-50`
- Each toast: dark card (`var(--bg-secondary)`) with colored left border, icon, message, dismiss button
- Colors: success=#238636, error=#f85149, info=#58a6ff, warning=#d29922
- Icons: Heroicons outline (check-circle, exclamation-circle, information-circle, exclamation-triangle)
- Animation: Svelte `transition:fly={{ x: 100, duration: 300 }}`
- Rendered at root level in `App.svelte`, after the main layout `</div>`

#### Integration Points (initial)

| Store | Success | Error |
|-------|---------|-------|
| `config.svelte.ts` | save → "Settings saved" | save → error message |
| `memory.svelte.ts` | save → "File saved", delete → "File deleted" | save/delete → error message |
| `connection.svelte.ts` | connect → "Connected to daemon" | connect → error message |
| `claudemd.svelte.ts` (new) | save/delete/create → success message | all → error message |

Other stores (plugins, mcp) can be integrated incrementally later.

### 2. Skill Content Preview

Small change spanning backend and frontend.

#### Backend

**New type** in `crates/claude-types/src/skills.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContentResponse {
    pub id: String,
    pub content: String,
}
```

**New handler** in `crates/claude-daemon/src/api/skills.rs`:

`GET /api/v1/skills/{id}/content`

Logic:
1. Scan user skills dir (`{claude_home}/skills/`) and plugin skills dirs (same logic as `list_skills`)
2. Find the first skill whose directory name matches `id`
3. Read its `SKILL.md` content via `std::fs::read_to_string`
4. Return `SkillContentResponse { id, content }`
5. If not found, return 404 with `SKILL_NOT_FOUND` error code

Extract a `find_skill_path(claude_home, id) -> Option<PathBuf>` helper from the existing scanning code to avoid duplication.

#### Frontend

**API client** (`src/lib/api/client.ts`):
```typescript
getSkillContent(id: string): Promise<SkillContentResponse>
```

**Skills store** (`src/lib/stores/skills.svelte.ts`):
- Add `skillContent = $state<string | null>(null)` and `contentLoading = $state(false)`
- Add `loadSkillContent(id: string)` method
- Modify `selectSkill(id)` to call `loadSkillContent(id)` automatically
- Update `reset()` to clear content state

**SkillPreview component** (`src/lib/components/skills/SkillPreview.svelte`):

Replace lines 66-77 (the placeholder block):
- When `skillsStore.contentLoading`: show "Loading..."
- When `skillsStore.skillContent` exists: display in existing `<pre>` element with same styling
- On load error: show toast notification

Content is rendered as plain text in `<pre>` (no markdown rendering).

### 3. CLAUDE.md Editor

New navigation module following the Memory module pattern exactly.

#### Backend Types (`crates/claude-types/src/claudemd.rs`)

```rust
pub struct ClaudeMdFile {
    pub id: String,           // "global" or "project:<project_id>"
    pub scope: String,        // "global" | "project"
    pub filename: String,     // always "CLAUDE.md"
    pub path: String,         // full filesystem path
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub exists: bool,         // false = can be created
}

pub struct ClaudeMdFileDetail {
    pub id: String,
    pub scope: String,
    pub filename: String,
    pub path: String,
    pub content: String,
    pub project_id: Option<String>,
}

pub struct UpdateClaudeMdRequest {
    pub content: String,
}
```

Register module: `pub mod claudemd;` in `crates/claude-types/src/lib.rs`.

#### Backend API (`crates/claude-daemon/src/api/claudemd.rs`)

**Endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/claudemd` | List all CLAUDE.md files |
| GET | `/api/v1/claudemd/{id}` | Read file content |
| PUT | `/api/v1/claudemd/{id}` | Update file content |
| DELETE | `/api/v1/claudemd/{id}` | Delete file |

**`GET /api/v1/claudemd` — List files:**

Scans two sources:
1. `{claude_home}/CLAUDE.md` → id="global", scope="global", exists=file_exists
2. For each registered project (from `state.inner.projects`): `{project_path}/.claude/CLAUDE.md` → id="project:{project_id}", scope="project", exists=file_exists

Returns all entries, including non-existent files (with `exists: false`), so the frontend can offer "create" for projects without a CLAUDE.md.

**`GET /api/v1/claudemd/{id}` — Read content:**

ID decoding:
- `id = "global"` → `{claude_home}/CLAUDE.md`
- `id = "project:{project_id}"` → look up project path from registered projects → `{project_path}/.claude/CLAUDE.md`

Returns 404 if file doesn't exist.

**`PUT /api/v1/claudemd/{id}` — Update/Create:**

Same path resolution. Uses `claude_config::write::atomic_write`. Creates parent directory (`.claude/`) if it doesn't exist. This endpoint handles both update and create — PUT is idempotent: if the file doesn't exist it creates it, if it exists it overwrites. The frontend sends the same request in both cases; the "create" flow in the UI is just selecting a non-existent project entry and saving.

**`DELETE /api/v1/claudemd/{id}` — Delete:**

Same path resolution. Returns 404 if file doesn't exist. Returns 204 on success.

**Security:** The handler validates that `id` follows the expected format (`"global"` or `"project:<registered_project_id>"`). Project IDs are verified against the registered projects list — arbitrary paths are rejected.

Register module: `pub mod claudemd;` in `crates/claude-daemon/src/api/mod.rs`.

Register routes in `crates/claude-daemon/src/server.rs`:
```rust
.route("/api/v1/claudemd", get(list_claudemd_files))
.route("/api/v1/claudemd/{id}", get(get_claudemd_file).put(put_claudemd_file).delete(delete_claudemd_file))
```

#### Frontend Types (`src/lib/api/types.ts`)

```typescript
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

#### Frontend API Client (`src/lib/api/client.ts`)

```typescript
listClaudeMdFiles(): Promise<ClaudeMdFile[]>
getClaudeMdFile(id: string): Promise<ClaudeMdFileDetail>
updateClaudeMdFile(id: string, content: string): Promise<void>
deleteClaudeMdFile(id: string): Promise<void>
```

#### Frontend Store (`src/lib/stores/claudemd.svelte.ts`)

Follows `memory.svelte.ts` pattern:

```typescript
class ClaudeMdStore {
  files = $state<ClaudeMdFile[]>([]);
  activeFile = $state<ClaudeMdFileDetail | null>(null);
  loading = $state(false);
  saving = $state(false);
  error = $state<string>("");

  async loadFiles(): Promise<void>;
  async loadFile(id: string): Promise<void>;
  async saveFile(id: string, content: string): Promise<void>;
  async deleteFile(id: string): Promise<void>;
  selectFile(id: string): void;
  reset(): void;
}
```

Save/delete operations call `toastStore.success()` or `toastStore.error()`.

#### Frontend Components

**`src/lib/components/claudemd/ClaudeMdList.svelte`** (~100 lines)

Sub-panel list component:
- Groups files by scope with section headers: "全局" and "项目"
- Global file: always shown, click to select
- Project files: show project name, blue text if exists, gray italic "点击创建" if not
- Active item highlighted with `bg-gray-800`
- Pattern: follows `MemoryList.svelte`

**`src/lib/components/claudemd/ClaudeMdEditor.svelte`** (~130 lines)

Detail panel editor:
- Header: filename + scope badge (blue "全局" or purple "项目:name") + dirty indicator (orange "unsaved") + file path
- Buttons: Save (blue, disabled when clean) + Delete (red outline)
- Body: `<textarea>` with `bind:value={localContent}`, monospace font
- Dirty tracking: `localContent !== originalContent` (identical to `MemoryEditor.svelte`)
- Loading state when `claudeMdStore.loading`
- Error bar at bottom when `claudeMdStore.error`
- For non-existent files (creating): starts with empty textarea, Save creates the file

**`src/lib/components/claudemd/ClaudeMdModule.svelte`** (~8 lines)

Simple wrapper rendering `<ClaudeMdEditor />`.

#### App.svelte Integration

1. **New nav button** in `navButtons` array (between Memory "M" and MCP "C"):
   ```
   { id: "D", label: "指令", icon: "<document-text heroicon path>" }
   ```

2. **Helper function**: `isClaudeMdModule()` returning `activeNav === "D"`

3. **Sub-panel**: render `<ClaudeMdList />` when `isClaudeMdModule()`

4. **Detail panel**: render `<ClaudeMdModule />` when `isClaudeMdModule()`

5. **Data loading** in `$effect` (line 148): add `claudeMdStore.loadFiles()` when connection becomes active

6. **Store reset** in `connection.svelte.ts`: add `claudeMdStore.reset()` to `resetAllStores()`

## Implementation Order

1. **Toast system** (Step 1) — creates the notification infrastructure used by Steps 2 and 3
2. **Skill content preview** (Step 2) — smallest scope, good warm-up for backend pattern
3. **CLAUDE.md editor** (Step 3) — largest scope, depends on Toast for save feedback

## Files Changed

### New Files (8)

| File | Purpose |
|------|---------|
| `src/lib/stores/toast.svelte.ts` | Toast notification store |
| `src/lib/components/shared/Toast.svelte` | Toast UI component |
| `crates/claude-types/src/claudemd.rs` | CLAUDE.md type definitions |
| `crates/claude-daemon/src/api/claudemd.rs` | CLAUDE.md API handlers |
| `src/lib/stores/claudemd.svelte.ts` | CLAUDE.md frontend store |
| `src/lib/components/claudemd/ClaudeMdList.svelte` | File list sub-panel |
| `src/lib/components/claudemd/ClaudeMdEditor.svelte` | File editor detail panel |
| `src/lib/components/claudemd/ClaudeMdModule.svelte` | Module wrapper |

### Modified Files (13)

| File | Change |
|------|--------|
| `src/App.svelte` | Toast rendering + CLAUDE.md nav/panels + data loading |
| `src/lib/api/types.ts` | SkillContentResponse + ClaudeMd types |
| `src/lib/api/client.ts` | Skill content + CLAUDE.md API methods |
| `src/lib/stores/skills.svelte.ts` | Content state, loading, auto-load on select |
| `src/lib/stores/config.svelte.ts` | Toast integration on save |
| `src/lib/stores/memory.svelte.ts` | Toast integration on save/delete |
| `src/lib/stores/connection.svelte.ts` | Toast integration on connect + claudemd reset |
| `src/lib/components/skills/SkillPreview.svelte` | Replace placeholder with actual content |
| `crates/claude-types/src/lib.rs` | `pub mod claudemd` |
| `crates/claude-types/src/skills.rs` | SkillContentResponse type |
| `crates/claude-daemon/src/api/mod.rs` | `pub mod claudemd` |
| `crates/claude-daemon/src/api/skills.rs` | `get_skill_content` handler + `find_skill_path` helper |
| `crates/claude-daemon/src/server.rs` | New route registrations |

## Verification

### Per-step checks
1. `cargo test --workspace` — all Rust tests pass
2. `cargo build -p claude-daemon` — daemon compiles
3. `pnpm build` — frontend compiles

### End-to-end
1. Start daemon: `cargo run -p claude-daemon -- --port 7890`
2. Start app: `pnpm tauri dev`
3. **Toast**: Save settings → green toast appears bottom-right; disconnect daemon → error toast
4. **Skill Preview**: Click any skill → SKILL.md full content displayed instead of placeholder
5. **CLAUDE.md**: "指令" appears in sidebar → lists global + project files → edit and save → toast confirms
