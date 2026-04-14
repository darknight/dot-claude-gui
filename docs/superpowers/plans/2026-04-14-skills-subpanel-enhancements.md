# Plan: Skills 子面板增强 — 数量显示、排序、分组

## Context

Skills 模块的子面板（中间栏）目前只有一个平面列表，缺少实用功能。需要添加：
1. 显示技能总数
2. 按名称/添加时间排序（正序/倒序）
3. 按插件来源分组

**分支**: `feature/phase-7.1-dedaemonization`（worktree 在 `.worktrees/phase-7-1-dedaemon/`）

---

## 现状分析

**SkillInfo 字段**（Rust + TypeScript 一致）：
- `id`, `name`, `description`, `source`, `path`, `valid`, `validationError`
- **没有时间戳字段** — 需要新增

**source 字段格式**：
- 用户技能：`"user"`
- 插件技能：`"plugin:scope@marketplace_id"` （如 `"plugin:user@claude-plugins-official"`）

---

## 实现方案

### Step 1: 后端 — 给 SkillInfo 添加 `modifiedAt` 时间戳

**文件**：
- `crates/claude-types/src/skills.rs` — `SkillInfo` 结构体添加 `modified_at: Option<i64>`（Unix 秒）
- `src-tauri/src/commands/skills.rs` — `scan_skills_dir` 中读取 SKILL.md 的文件修改时间 (`std::fs::metadata → modified()`)
- `src/lib/api/types.ts` — TypeScript `SkillInfo` 添加 `modifiedAt?: number`

实现细节：
```rust
// 在 scan_skills_dir 中，push SkillInfo 前获取 mtime
let modified_at = std::fs::metadata(&skill_md_path)
    .and_then(|m| m.modified())
    .ok()
    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|d| d.as_secs() as i64);
```

### Step 2: 前端 — Skills 子面板从 App.svelte 提取为独立组件

当前 Skills 子面板的代码内联在 App.svelte 的 `{:else if activeNav === "K"}` 分支中（约 30 行）。排序/分组逻辑会使其膨胀，应提取为 `src/lib/components/skills/SkillsList.svelte`。

**参考模式**：Memory 模块已经将子面板提取为 `MemoryList.svelte`（App.svelte 中直接 `<MemoryList />`）。

### Step 3: 前端 — SkillsList.svelte 组件实现

**文件**: `src/lib/components/skills/SkillsList.svelte`

**结构**：
```
┌─────────────────────────┐
│ 技能 (32)               │  ← 标题 + 数量
│ [排序 ▾] [分组 ▾]       │  ← 工具栏：排序下拉 + 分组开关
├─────────────────────────┤
│ ▸ superpowers (12)      │  ← 分组模式：可折叠分组头
│   brainstorming         │
│   debugging             │
│   ...                   │
│ ▸ user (3)              │
│   my-skill              │
│   ...                   │
├─────────────────────────┤
│ （非分组模式时为平面列表）│
└─────────────────────────┘
```

**排序选项**（下拉菜单）：
- 名称 A→Z（默认）
- 名称 Z→A
- 最近修改优先
- 最早修改优先

**分组逻辑**：
- 解析 `source` 字段：`"user"` → 分组名 "用户技能"，`"plugin:user@claude-plugins-official"` → 提取 marketplace 前面的插件包名（从 `installed_plugins.json` 的 key 中获取，如 `superpowers`）
- 实际实现：source 格式为 `plugin:scope@marketplace`，需要从 path 字段提取插件名（path 包含如 `.../cache/claude-plugins-official/superpowers/5.0.7/skills/...`）
- 更好的方案：在后端 `scan_skills_dir` 中直接传入插件名作为 source 的一部分，或新增 `pluginName` 字段

**决策**：在 SkillInfo 中新增 `pluginName: Option<String>` 字段，后端填充（对用户技能为 None，对插件技能为插件包名如 "superpowers"）。这比前端解析 path 更可靠。

### Step 4: 前端 — 排序和分组逻辑

在 `SkillsList.svelte` 中用 `$derived` 计算排序/分组后的列表：

```typescript
let sortBy = $state<"name-asc" | "name-desc" | "time-desc" | "time-asc">("name-asc");
let groupByPlugin = $state(false);

const sortedSkills = $derived(/* 排序逻辑 */);
const groupedSkills = $derived(/* 分组逻辑：Map<string, SkillInfo[]> */);
```

### Step 5: App.svelte — 替换内联代码

将 Skills 子面板内联代码替换为：
```svelte
{:else if activeNav === "K"}
  <SkillsList />
```

---

## 需要修改的文件

| 文件 | 改动 |
|------|------|
| `crates/claude-types/src/skills.rs` | SkillInfo 添加 `modified_at: Option<i64>`, `plugin_name: Option<String>` |
| `src-tauri/src/commands/skills.rs` | `scan_skills_dir` 读取 mtime, `list_skills_logic` 传入 plugin_name |
| `src/lib/api/types.ts` | SkillInfo 添加 `modifiedAt?: number`, `pluginName?: string` |
| `src/lib/components/skills/SkillsList.svelte` | **新建** — 子面板组件（数量、排序、分组） |
| `src/App.svelte` | Skills 子面板替换为 `<SkillsList />` |

---

## 验证

1. `cargo test --workspace` — 确认后端类型改动不破坏测试
2. `pnpm svelte-check` — 确认 TypeScript 无报错
3. UI 验证（在 Tauri app 中）：
   - 技能数量正确显示
   - 排序：按名称正/倒序切换，列表顺序正确变化
   - 排序：按修改时间正/倒序切换
   - 分组：开启后按插件名分组，每组有标题和数量
   - 分组：用户技能单独一组
   - 点击技能仍能正常加载详情
