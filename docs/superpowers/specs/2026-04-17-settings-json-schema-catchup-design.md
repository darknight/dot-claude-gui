# settings.json Schema Catchup — Design

**Date**: 2026-04-17
**Scope**: 补齐 `~/.claude/settings.json` 所有未建模字段（~40 个），并兑现 release notes 2.1.110–2.1.112 里属于 settings 层的新字段（`tui`、`effortLevel.xhigh`）。
**Branch**: `feat/settings-schema-catchup`（可调）

---

## 1. 背景

Claude Code 最近三个版本（2.1.110/.111/.112）引入若干新设置字段。排查时发现：

1. **字段跨三个存储层**。`~/.claude/settings.json`（本项目管理）、`~/.claude.json`（Claude Code OAuth + UI state，本项目未管）、环境变量（EnvVarEditor 已支持）。
2. **本仓库 `Settings` struct 仅建模 15 个字段**，对比 Claude Code `cli.js@2.1.112` 内嵌 zod schema 抽出的 ~55 个顶层 settings 字段，长期靠 `extra: HashMap<String, Value>` 兜底透传，没有 GUI 编辑入口。
3. **`schemastore.org/claude-code-settings.json` 落后于 Claude Code 实际版本**（已确认未收录 2.1.110+ 新增字段），不能作为唯一 source of truth。

本 spec 解决 settings.json 层的技术债。`~/.claude.json` 管理由另一份 spec 覆盖。

---

## 2. 目标与非目标

### 目标

- 把 `Settings` struct 字段从 15 扩展到 ~55，所有可枚举顶层字段在 Rust/TS 里显式声明
- GUI 可编辑字段相应扩展，tab 数从 6 增加到 10
- release notes 2.1.110+ 里属于 settings 层的 `tui`、`effortLevel`（含 `xhigh`）可在 GUI 直接编辑
- 建立 schema 漂移检测机制：从本机 `@anthropic-ai/claude-code/cli.js` 抽取 zod schema，与项目维护的 snapshot 对比，新字段能被主动发现
- 每个里程碑独立可合入、独立可验收，不出现"大 PR"

### 非目标

- 不管理 `~/.claude.json`（theme / autoScrollEnabled / 推送通知 / Remote Control 等）——另一份 spec
- 不做 env 变量预设下拉（EnvVarEditor 现状够用）
- 不做 Claude Code 启动参数面板
- 不与 Plan D Account/Workspace 集成
- 不为 Tier 3 长尾字段（`spinnerVerbs` 等）做语义化专门表单
- 不引入 ts-rs 或 codegen 管线
- 不引入第三语言 i18n

---

## 3. 成功标准

1. 从 `cli.js@2.1.112` 抽出的所有顶层 settings 字段，要么在 Rust/TS 里有显式字段，要么在明确的「子对象走 Value」跳过列表里
2. 用户在 GUI 可编辑字段从 15 个扩大到 ~55 个
3. `pnpm check:schema` 可在本地运行，发现本机 Claude Code 较 snapshot 新增/删除/变更字段
4. Claude Code 2.1.110+ 新字段 `tui`、`effortLevel.xhigh` 在 GUI 下拉可切换，写入文件后 Claude Code 启动无 schema 报错

---

## 4. 里程碑拆解

每个里程碑独立可合入、独立可验收。依赖关系：M1 → M2 → { M3, M4, M5, M6, M7 并行 } → M8 → M9。

| # | 主题 | 涉及字段 | 验收标准 |
|---|------|---------|---------|
| **M1** | Schema 抽取工具 & 基线快照 | — | `scripts/extract-claude-schema.ts` 能从本机 cli.js 抽字段表到 `docs/claude-schema-snapshot.json`；`pnpm check:schema` 对比当前版本输出 0 diff；故意增删快照后能正确报 diff；CI workflow 每周一 cron 跑一次，diff 非空则标红 |
| **M2** | release notes 2.1.110+ 核心 | `tui`, `effortLevel`（含 `xhigh`） | 两字段在 General tab 下拉可切换；写入 `"fullscreen"` / `"xhigh"` 后 Claude Code 启动无 schema 报错；`cargo test -p claude-types` roundtrip + 类型化断言通过 |
| **M3** | Runtime tab | `model`, `outputStyle`, `fastMode`, `fastModePerSessionOptIn`, `availableModels`, `autoCompactWindow`, `showClearContextOnPlanAccept`, `promptSuggestionEnabled` | 新 Runtime tab 出现；每字段可编辑、回显、持久化；JsonPreview 正确 |
| **M4** | General 扩展 | `autoMemoryEnabled`, `includeGitInstructions`, `respectGitignore`, `cleanupPeriodDays`, `claudeMdExcludes`, `plansDirectory`, `syntaxHighlightingDisabled` | 扩展现有 GeneralEditor，不新开 tab；每字段 roundtrip |
| **M5** | MCP tab | `allowedMcpServers`, `enabledMcpjsonServers`, `disabledMcpjsonServers`, `enableAllProjectMcpServers`, `allowManagedMcpServersOnly` + 既有 `deniedMcpServers` 迁入 | MCP tab 出现；allowed/denied 列表 CRUD；jsonServers 启用/禁用；策略开关可切换 |
| **M6** | Plugins & Marketplace tab | `extraKnownMarketplaces`, `strictKnownMarketplaces`, `blockedMarketplaces`, `skippedMarketplaces`, `skippedPlugins`, `pluginConfigs`, `pluginTrustMessage`, `skillOverrides` | 市场/插件名单 CRUD；`pluginConfigs` 走内嵌 JSON 编辑 |
| **M7** | Hooks Policy 扩展 | `disableAllHooks`, `allowedHttpHookUrls`, `httpHookAllowedEnvVars`, `allowManagedHooksOnly`, `allowManagedPermissionRulesOnly`, `disableSkillShellExecution` | 扩展现有 HooksEditor（同 tab 下新增小节），所有策略字段可编辑 |
| **M8** | Advanced JSON tab | Tier 3 长尾 ~25 个 + 子对象 Value 字段 | Advanced tab 左侧 key picker（从 snapshot 读取 + describe tooltip），右侧按类型选对应编辑器；任何已知字段可增删改；类型化断言：已知字段不落入 `extra` |
| **M9** | i18n 补全 + 端到端回归 | — | 所有新字段 zh-CN + en-US 双语；fixtures 补齐；完整 fixture → 10 tab 回显 → 随机编辑 → 保存 → 重载一致；`cargo test --workspace` + `pnpm build` 全绿 |

**Tier 3 长尾明细**（M8 覆盖）：
- 认证：`apiKeyHelper`、`awsCredentialExport`、`awsAuthRefresh`、`gcpAuthRefresh`、`forceLoginMethod`、`forceLoginOrgUUID`、`otelHeadersHelper`
- 体验：`prefersReducedMotion`、`companyAnnouncements`、`feedbackSurveyRate`、`terminalTitleFromRename`、`awaySummaryEnabled`、`showThinkingSummaries`
- 其他：`advisorModel`、`agent`、`autoDreamEnabled`、`autoMemoryDirectory`、`skillListingBudgetFraction`、`skillListingMaxDescChars`、`skipWebFetchPreflight`、`forceRemoteSettingsRefresh`
- 子对象（以 `Value` 存，编辑入口进 Advanced tab）：`attribution`、`autoMode`、`fileSuggestion`、`worktree`、`subagentStatusLine`、`spinnerVerbs`、`spinnerTipsOverride`、`remote`
  - `pluginConfigs` 也用 `Value` 存，但 UI 入口在 M6 的 Plugins & Marketplace tab，不在 Advanced

---

## 5. Rust / TS 类型建模

**顶层字段**：全部显式声明在 `Settings` struct，每个 `Option<T>` + `skip_serializing_if = "Option::is_none"`，与既有风格一致。

**枚举字段**（`tui`、`effortLevel`、`autoUpdatesChannel`、`forceLoginMethod` 等）：使用 `Option<String>`，不用 Rust enum。理由：forward-compat，Claude Code 若新增枚举值也不会反序列化失败；UI 下拉组件维护合法值白名单。

**子对象策略**：
- **有独立 UI 的子对象**（已有的 `Permissions`、`Hooks`、`StatusLine`、`Sandbox`，新增 `AllowedMcpServer`、`Marketplace*` 等）：独立 struct，各带 `extra: HashMap<String, Value>` 兜底
- **Rust 端以 `Value` 存的子对象**：不造 struct，UI 上按所属 tab 呈现 JSON 编辑器。
  - 进 Advanced JSON tab 的（M8）：`attribution`、`autoMode`、`fileSuggestion`、`worktree`、`subagentStatusLine`、`spinnerVerbs`、`spinnerTipsOverride`、`remote`
  - 进 Plugins & Marketplace tab 的（M6）：`pluginConfigs`（因和插件语义耦合，虽然 Rust 端也是 `Value`，UI 放在 M6 的 tab 内嵌 JSON 编辑器里而非 Advanced）

**`extra: HashMap<String, Value>`**：保留，作为 Claude Code 未来新增字段被 GUI 吞掉前的最后兜底。M1 的 schema drift 检测负责提醒"新字段应该建模"。

**TS 类型**：`src/lib/api/types.ts` 手动同步 Rust struct（本项目已是这个模式）。不引入 ts-rs。

---

## 6. 前端组件与路由

### Settings 导航（最终 10 tab）

```
General                 ← 已有，M4 扩展
Env                     ← 已有，不改
Permissions             ← 已有，不改
Hooks                   ← 已有，M7 扩展（加 Policy 小节）
Sandbox                 ← 已有，不改
Status Line             ← 已有，不改
── 新增 ──
Runtime                 ← M3
MCP                     ← M5
Plugins & Marketplace   ← M6
Advanced                ← M8
```

### 新增组件

```
src/lib/components/settings/
  RuntimeEditor.svelte              ← M3
  McpPolicyEditor.svelte            ← M5
  PluginsMarketplaceEditor.svelte   ← M6
  AdvancedJsonEditor.svelte         ← M8
  sub/
    McpServerForm.svelte            ← M5，allowed/denied 共用
    MarketplaceEntryForm.svelte     ← M6
    JsonValueEditor.svelte          ← M8
    SchemaKeyPicker.svelte          ← M8
```

`SettingsEditor.svelte` 维持现有 `{#if activeSection === "..."}` 直接状态比较路由（CLAUDE.md Gotcha #4）。

### 数据流

继续走单例 `configStore`：所有编辑器订阅 `configStore.activeSettings`、改动调 `configStore.markDirty()` + `configStore.save(patch)`。Advanced JSON 写入未知字段时顺其自然走 `extra`。每个 tab 底部保留 `JsonPreview` 便于调试。

### Advanced tab 细节（M8）

- **左栏**：列出 schema snapshot 所有字段，按类别分组，搜索框过滤。每项显示字段名、类型、一句 describe、当前值是否被设置（✓ 标记）
- **右栏**：根据字段类型选编辑器
  - `boolean` → checkbox
  - `enum` → select（从 snapshot 的 `enumValues` 渲染）
  - `string` / `number` → input
  - `array` / `object` → JSON textarea（`JSON.parse` 校验）
- "重置为默认"按钮 = 从 settings 中删除该 key

---

## 7. Schema 抽取脚本 & 漂移检测

### 脚本：`scripts/extract-claude-schema.ts`

**查找 cli.js 的顺序**：
1. `CLAUDE_CODE_CLI_JS` 环境变量（显式指定）
2. `$(pnpm root -g)/@anthropic-ai/claude-code/cli.js`
3. `$(npm root -g)/@anthropic-ai/claude-code/cli.js`
4. `~/.bun/install/global/node_modules/@anthropic-ai/claude-code/cli.js`

全部失败时报错并提示安装。

**抽取 regex**（基于已验证的 2.1.112 pattern）：
- 顶层 zod：`[a-zA-Z_]{2,40}:y\.(string|boolean|number|enum|array|object|record|union|literal)\(` + 后续 `.optional()` / `.describe("...")` 片段
- `/config` 描述表：`[a-zA-Z_\.]+:\{source:"(settings|global)",type:"[^"]+",description:"[^"]*"`（顺便记录 `source:"global"` 的字段为下一个 spec 用）

解析失败的字段不吃异常，写进 `warnings` 数组让维护者决定是否升级 pattern。

### Snapshot：`docs/claude-schema-snapshot.json`

```jsonc
{
  "claudeCodeVersion": "2.1.112",
  "extractedAt": "2026-04-17T...",
  "settingsFields": [
    { "name": "tui", "type": "enum", "enumValues": ["default","fullscreen"], "optional": true, "describe": "Terminal UI renderer…" },
    { "name": "effortLevel", "type": "enum", "enumValues": ["low","medium","high","xhigh"], "optional": true, "describe": "Persisted effort level…" }
    // …
  ],
  "globalFields": [
    { "name": "theme", "source": "global", "type": "string", "describe": "Color theme for the UI" }
    // …给下个 spec 用
  ],
  "warnings": []
}
```

### 命令

- `pnpm check:schema` — 抽本机 Claude Code，对比 snapshot，diff 非空 → 退出码 1 + 打印变更
- `pnpm check:schema --update` — 覆盖 snapshot（用于维护者确认新字段后刷新基线）

### CI

`.github/workflows/schema-drift.yml`：
- 每周一 cron + 可手动触发
- Runner 上 `npm i -g @anthropic-ai/claude-code@latest`
- 跑 `pnpm check:schema`
- diff 非空 → workflow 标红（**不自动开 issue、不 fail 常规 PR CI**）

### 本机开发者工作流（写入 `CLAUDE.md` 或 `scripts/README.md`）

1. 本地 `claude update` 或 `npm i -g @anthropic-ai/claude-code@latest`
2. 跑 `pnpm check:schema`，若有 diff：
   - 新字段 → 评估；小字段直接补 M3/M4/M8；涉及新子系统则开 spec
   - 枚举值变更 → 更新对应下拉选项
   - 类型变更 → 处理破坏性变更
3. 确认后 `pnpm check:schema --update` 刷新快照并提交

---

## 8. 测试与验证

### Rust 端

1. **现有 roundtrip 测试**：`tests/fixtures/settings-full.json` 每里程碑补齐新字段
2. **Tier 1/2 类型化断言**（每里程碑新增）：断言新建模字段不落入 `extra`
   ```rust
   let s: Settings = serde_json::from_str(r#"{"tui":"fullscreen"}"#).unwrap();
   assert_eq!(s.tui.as_deref(), Some("fullscreen"));
   assert!(!s.extra.contains_key("tui"));
   ```
3. **unknown-fields 兜底测试**：已有 `roundtrip_preserves_unknown_fields`，保留
4. **schema snapshot 对齐测试**（M1 新增）：读 `docs/claude-schema-snapshot.json`，断言 snapshot 每个字段在 `Settings` struct 里有同名字段或在明确的「子对象走 Value」跳过列表里

### 前端

项目无前端测试套件（CLAUDE.md 明确），本 spec 不引入。代替：**每里程碑手动冒烟**：
- 加载全字段 fixture → tab 下字段回显正确
- 编辑 + 保存 + 重载 → 一致
- `tauri dev` 控制台无 Svelte 报错（CLAUDE.md Gotcha #5）

### 端到端验证（M9）

- 完整 fixture 包含所有 ~55 字段
- 10 个 tab 逐个回显
- 随机改 5-10 字段，保存，`diff` 文件前后只有改动 key 变化
- 用修改后的 settings.json 启动 `claude` CLI，确认无 schema validation 报错

### 回归清单（M9 收尾）

- 既有 6 tab 功能不回退（Permissions、Hooks、Sandbox 的复杂列表编辑）
- Env tab CRUD 正常
- Effective Config view 仍能 merge 所有层
- Watcher 在外部修改 settings.json 时仍能触发 `config-changed` 事件
- i18n 切换 zh-CN ↔ en-US 所有新字段都有翻译

---

## 9. i18n 策略

**原则**：
- 每字段 zh-CN + en-US 两套 label + 一句 tooltip
- label 人工翻译
- Tooltip **默认复用 Claude Code `.describe()` 英文字符串**作为 en-US；zh-CN tooltip 保留原意即可
- Schema snapshot 抽取的 `describe` 字段自动填入 en-US tooltip 默认值

**组织**：
- `src/lib/i18n.ts` 维持现有 flat key 结构
- 新 key 约定：`settings.fields.<camelCaseFieldName>.label` / `.tooltip`
- 每里程碑同步补齐自身字段的翻译；M9 只做扫尾（漏翻 + 拼写检查）

**Tab 名称**：
- Runtime → 运行时
- MCP → MCP
- Plugins & Marketplace → 插件与市场
- Advanced → 高级

**Advanced tab 特殊**：字段清单 schema 驱动；未翻译字段 fallback 显示英文 key + describe，tooltip 提示"未翻译"。

---

## 10. 非目标回顾 / 开放问题 / 风险

### 开放问题（不阻塞 spec）

- 分支名：默认 `feat/settings-schema-catchup`
- Advanced tab 的"隐藏已处理字段"开关是否默认开启
- `pluginConfigs` 先内嵌 JSON 编辑，不做键值 CRUD
- schema drift CI cron 频率（每周一）

### 风险

- **cli.js 压缩器换了**（esbuild → terser 等）→ regex 失效 → 缓解：多 pattern + `warnings` 数组、手动 fallback
- **本机没装 Claude Code**：`check:schema` 报错并提示安装路径
- **release notes 提到的字段实际在 `~/.claude.json`**（如 `theme`）：本 spec 不管，snapshot 会记录到 `globalFields` 区给下个 spec 用

---

## 11. 参考

- Claude Code 内嵌 zod schema：`@anthropic-ai/claude-code@2.1.112/cli.js`（单文件 13MB minified）
- Schemastore schema（落后于 Claude Code，仅参考）：`https://json.schemastore.org/claude-code-settings.json`
- 上一份管理 global config 的初步计划（等待下一份 spec）：`/Users/eric.yao/.ccs/instances/me/plans/claude-code-release-humming-bumblebee.md`
- 相关现有设计：
  - `docs/superpowers/specs/2026-03-30-dot-claude-gui-design.md`（项目基础设计）
  - `docs/superpowers/specs/2026-04-10-plan-d-account-workspace-design.md`（Plan D，未来集成，本 spec 不涉及）
  - `docs/superpowers/specs/2026-04-16-i18n-design.md`（i18n 基础）
