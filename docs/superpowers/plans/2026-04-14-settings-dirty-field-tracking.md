# Plan: Settings 编辑器 — 字段级修改标记

## Context

切换 scope 或编辑设置后，UI 仅在工具栏显示全局的 "Unsaved changes" 提示，但没有标记具体哪些字段被修改了。用户无法一眼看出哪些配置发生了变化，体验不友好。

## 需求

- 每个被修改的字段旁边应有视觉指示（如左侧彩色竖线、圆点、或背景高亮）
- 未修改的字段保持正常样式
- Revert 后所有标记清除

## 涉及的编辑器组件

| 组件 | 路径 |
|------|------|
| GeneralEditor | `src/lib/components/settings/GeneralEditor.svelte` |
| PermissionsEditor | `src/lib/components/settings/PermissionsEditor.svelte` |
| HooksEditor | `src/lib/components/settings/HooksEditor.svelte` |
| SandboxEditor | `src/lib/components/settings/SandboxEditor.svelte` |
| EnvVarEditor | `src/lib/components/settings/EnvVarEditor.svelte` |
| StatusLineEditor | `src/lib/components/settings/StatusLineEditor.svelte` |

## 实现思路

每个编辑器中，用 `$derived` 对比当前本地值与 `configStore.activeSettings` 中的存储值：

```typescript
const languageDirty = $derived(language !== (settings.language ?? ""));
const thinkingDirty = $derived(alwaysThinkingEnabled !== (settings.alwaysThinkingEnabled ?? false));
```

在模板中，为修改过的字段添加左侧指示线：

```svelte
<div class="space-y-1" class:border-l-2 class:border-l-blue-500 class:pl-2={languageDirty}>
```

## 优先级

低 — 不影响功能，属于 UX 增强。可在 Phase 7.2 或独立迭代中实现。
