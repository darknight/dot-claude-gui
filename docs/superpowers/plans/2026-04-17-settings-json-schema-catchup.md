# settings.json Schema Catchup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 `~/.claude/settings.json` 的 Rust Settings struct 从 15 字段扩展到 ~55，引入 schema 漂移检测，并新增 Runtime / MCP / Plugins & Marketplace / Advanced 四个 UI tab，兑现 Claude Code 2.1.110+ 的 `tui`、`effortLevel.xhigh` 等新字段。

**Architecture:** 所有工作在单一分支 `feat/settings-schema-catchup` 上逐里程碑完成。Rust 端扩展 `crates/claude-types/src/settings.rs`，TS 端同步 `src/lib/api/types.ts`，UI 侧在 `src/lib/components/settings/` 新增组件、在 `src/App.svelte:95` 扩展 `settingsSections`。Schema 抽取脚本从本机 `@anthropic-ai/claude-code/cli.js` 内嵌 zod 定义提取字段清单并与快照对比。

**Tech Stack:** Tauri 2.0 · Svelte 5 runes · Rust workspace (claude-types + claude-config) · pnpm · cargo test · TypeScript (tsx 运行脚本) · Node built-ins only（脚本无新增依赖）

---

## 共用上下文

### 参考文件

- Spec：`docs/superpowers/specs/2026-04-17-settings-json-schema-catchup-design.md`
- Rust Settings 现状：`crates/claude-types/src/settings.rs:11-63`
- TS Settings 现状：`src/lib/api/types.ts:10-28`
- SettingsEditor 路由：`src/lib/components/settings/SettingsEditor.svelte:30-44`
- Sub-panel 导航：`src/App.svelte:95-102`
- 典型表单编辑器：`src/lib/components/settings/GeneralEditor.svelte`
- Config store：`src/lib/stores/config.svelte.ts`
- i18n：`src/lib/i18n.ts` · 语言文件 `src/lib/locales/*.json`
- Fixture：`tests/fixtures/settings-full.json`

### 关键约束（来自 CLAUDE.md）

- **Svelte 5 HMR 不刷新响应图**：新增 `$state` / `$effect` / `$derived` 时必须重启 `pnpm tauri dev`
- **`onDestroy` 必须同步调用**：不能放在 `await` 后的 callback；异步清理用 `onMount(() => { ...; return () => {...} })` 返回函数
- **`{#each}` key 全局唯一**：复合 key `(item.id + ':' + item.source)`
- **`{#if}` 用直接状态比较**：`{#if activeNav === "S"}` 而非 helper function
- **调试 UI 先开 Tauri DevTools**：`Cmd+Option+I`
- **Schema source of truth**：Claude Code `cli.js` 内嵌 zod schema（schemastore 落后）
- **`config-changed` events**：`source: "file-watcher"`（非 `"user"`），过滤时不要过严

### TDD 节奏（每个功能任务）

1. 写 fail 的测试
2. 运行测试确认失败（`cargo test -p claude-types <test_name>` 或其他）
3. 写最小实现
4. 运行测试确认通过
5. 提交

### Rust 字段添加统一模式

在 `Settings` struct 追加字段：
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub <field_name>: Option<T>,
```
位置：放在 `extra` 字段之前（`extra` 必须是最后一个 flatten 字段）。

### 提交消息约定

- 功能提交：`feat(settings): add <field>` / `feat(settings-ui): add <tab> tab`
- 类型提交：`feat(claude-types): add <field>`
- 测试提交：`test(claude-types): assert <field> typed`
- 文档提交：`docs(schema): regenerate snapshot for <version>`
- 工具提交：`chore(scripts): add extract-claude-schema`

### 里程碑终点动作

每个里程碑完成后：
1. 跑 `cargo test --workspace` → 全绿
2. 跑 `pnpm build` → 无 TS 错误
3. （UI 里程碑）启动 `pnpm tauri dev` 手动冒烟
4. 在分支上打 tag：`git tag milestone/M<n>-<slug>`
5. 进入下一个里程碑

### M0 — 准备分支（在开始 M1 前执行一次）

- [ ] **步骤 1：确认在 main 分支且状态干净**

```bash
git status
```
Expected: `On branch main ... nothing to commit, working tree clean`（或只有未推送到远程的 commit）

- [ ] **步骤 2：拉取最新 main**

```bash
git pull --ff-only origin main
```

- [ ] **步骤 3：创建并切换到功能分支**

```bash
git checkout -b feat/settings-schema-catchup
```
Expected: `Switched to a new branch 'feat/settings-schema-catchup'`

- [ ] **步骤 4：推送分支占位**

```bash
git push -u origin feat/settings-schema-catchup
```

---

## Milestone 1 — Schema 抽取工具与快照

**目标**：建立 `scripts/extract-claude-schema.ts`，从本机 Claude Code 的 `cli.js` 抽出 settings / global 字段定义，写入 `docs/claude-schema-snapshot.json`；`pnpm check:schema` 能发现漂移；GitHub Actions 每周跑一次并在 diff 时标红。

**Files:**
- Create: `scripts/extract-claude-schema.ts`
- Create: `scripts/check-claude-schema.ts`
- Create: `scripts/README.md`
- Create: `docs/claude-schema-snapshot.json`
- Create: `.github/workflows/schema-drift.yml`
- Modify: `package.json`（添加 `check:schema`、`extract:schema` scripts）

### Task 1：Schema 抽取脚本骨架

- [ ] **步骤 1：创建 `scripts/extract-claude-schema.ts` 骨架**

```typescript
#!/usr/bin/env tsx
// scripts/extract-claude-schema.ts
//
// 从本机 @anthropic-ai/claude-code 的 cli.js 抽取 settings/global 字段定义。
// 输出 JSON 到 stdout（或 --out <path>）。
//
// 用法:
//   pnpm extract:schema                          # 默认查找 npm/pnpm/bun 全局
//   CLAUDE_CODE_CLI_JS=/path/to/cli.js pnpm extract:schema
//   pnpm extract:schema --out docs/claude-schema-snapshot.json

import { readFileSync, existsSync, writeFileSync } from "node:fs";
import { execSync } from "node:child_process";
import { homedir } from "node:os";
import { join } from "node:path";

interface SchemaField {
  name: string;
  type: string;
  optional?: boolean;
  enumValues?: string[];
  describe?: string;
}

interface GlobalField {
  name: string;
  source: "settings" | "global";
  type: string;
  describe?: string;
  options?: string[];
}

interface SchemaSnapshot {
  claudeCodeVersion: string;
  extractedAt: string;
  settingsFields: SchemaField[];
  globalFields: GlobalField[];
  warnings: string[];
}

function findCliJs(): string {
  const envPath = process.env.CLAUDE_CODE_CLI_JS;
  if (envPath && existsSync(envPath)) return envPath;

  const candidates: string[] = [];
  for (const cmd of ["pnpm root -g", "npm root -g"]) {
    try {
      const root = execSync(cmd, { encoding: "utf8" }).trim();
      candidates.push(join(root, "@anthropic-ai/claude-code/cli.js"));
    } catch {
      /* ignore */
    }
  }
  candidates.push(
    join(homedir(), ".bun/install/global/node_modules/@anthropic-ai/claude-code/cli.js")
  );

  for (const c of candidates) {
    if (existsSync(c)) return c;
  }

  throw new Error(
    `Could not find @anthropic-ai/claude-code/cli.js. Tried:\n  ${candidates.join("\n  ")}\n` +
      `Install it via 'npm i -g @anthropic-ai/claude-code' or set CLAUDE_CODE_CLI_JS env var.`
  );
}

function readVersion(cliJsPath: string): string {
  // package.json sits next to cli.js
  const pkgPath = cliJsPath.replace(/cli\.js$/, "package.json");
  if (!existsSync(pkgPath)) return "unknown";
  const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
  return pkg.version ?? "unknown";
}

function extractSettingsFields(source: string, warnings: string[]): SchemaField[] {
  const fields: SchemaField[] = [];
  // Matches: foo:y.string().optional().describe("…")
  //          foo:y.enum(["a","b"]).optional()
  //          foo:y.boolean().optional().describe("…")
  //          foo:y.number().int().optional()
  //          foo:y.array(...)
  //          foo:y.object({...}).optional()
  const re = /([a-zA-Z_][a-zA-Z0-9_]{1,40}):y\.(string|boolean|number|enum|array|object|record|union|literal|any)\(/g;
  const seen = new Set<string>();

  for (const m of source.matchAll(re)) {
    const name = m[1];
    const type = m[2];
    if (seen.has(name)) continue;
    seen.add(name);

    const tail = source.slice(m.index! + m[0].length, m.index! + m[0].length + 400);
    const optional = /\)\.optional\(\)/.test(tail);

    let enumValues: string[] | undefined;
    if (type === "enum") {
      const enumMatch = source.slice(m.index!, m.index! + 400).match(/y\.enum\(\[([^\]]+)\]/);
      if (enumMatch) {
        enumValues = enumMatch[1]
          .split(",")
          .map((s) => s.trim().replace(/^"|"$/g, ""));
      } else {
        warnings.push(`Could not extract enum values for ${name}`);
      }
    }

    const descMatch = tail.match(/\.describe\(["'`]([^"'`]+)["'`]\)/);
    const describe = descMatch?.[1];

    fields.push({ name, type, optional, enumValues, describe });
  }

  return fields;
}

function extractGlobalFields(source: string, warnings: string[]): GlobalField[] {
  const fields: GlobalField[] = [];
  // Matches: foo:{source:"global",type:"boolean",description:"…"}
  //          foo:{source:"settings",type:"string",description:"…",options:[…]}
  const re =
    /([a-zA-Z_][a-zA-Z0-9_.]{1,40}):\{source:"(settings|global)",type:"([a-zA-Z_]+)",description:([^}]{0,500})/g;

  for (const m of source.matchAll(re)) {
    const name = m[1];
    const src = m[2] as "settings" | "global";
    const type = m[3];
    const descPayload = m[4];

    let describe: string | undefined;
    const descMatch = descPayload.match(/^["'`]([^"'`]+)["'`]/);
    if (descMatch) describe = descMatch[1];
    else warnings.push(`Could not parse description for ${name}`);

    fields.push({ name, source: src, type, describe });
  }

  return fields;
}

function main() {
  const outArg = process.argv.indexOf("--out");
  const outPath = outArg >= 0 ? process.argv[outArg + 1] : undefined;

  const cliJsPath = findCliJs();
  const version = readVersion(cliJsPath);
  const source = readFileSync(cliJsPath, "utf8");

  const warnings: string[] = [];
  const settingsFields = extractSettingsFields(source, warnings).sort((a, b) =>
    a.name.localeCompare(b.name)
  );
  const globalFields = extractGlobalFields(source, warnings).sort((a, b) =>
    a.name.localeCompare(b.name)
  );

  const snapshot: SchemaSnapshot = {
    claudeCodeVersion: version,
    extractedAt: new Date().toISOString(),
    settingsFields,
    globalFields,
    warnings,
  };

  const json = JSON.stringify(snapshot, null, 2) + "\n";
  if (outPath) {
    writeFileSync(outPath, json);
    process.stderr.write(
      `Wrote ${settingsFields.length} settings fields and ${globalFields.length} global fields to ${outPath}\n`
    );
  } else {
    process.stdout.write(json);
  }
}

main();
```

- [ ] **步骤 2：装 tsx 作为 devDependency**

```bash
pnpm add -Dw tsx
```

- [ ] **步骤 3：往 package.json 加 scripts**

在 `package.json` 的 `"scripts"` 对象里追加：
```json
"extract:schema": "tsx scripts/extract-claude-schema.ts --out docs/claude-schema-snapshot.json",
"check:schema": "tsx scripts/check-claude-schema.ts"
```

- [ ] **步骤 4：跑一次抽取，观察输出是否合理**

```bash
pnpm extract:schema
```
Expected stderr: `Wrote <N> settings fields and <M> global fields to docs/claude-schema-snapshot.json`，`N ≥ 40`、`M ≥ 10`。

若找不到 cli.js：
```bash
npm i -g @anthropic-ai/claude-code@latest
pnpm extract:schema
```

- [ ] **步骤 5：手工审查 `docs/claude-schema-snapshot.json`**

打开文件，确认：
- `claudeCodeVersion` 是当前版本（如 `"2.1.112"`）
- `settingsFields` 数组含 `tui`（`type:"enum"`, `enumValues:["default","fullscreen"]`）
- `settingsFields` 数组含 `effortLevel`（`type:"enum"`, `enumValues:["low","medium","high","xhigh"]`）
- `globalFields` 数组含 `theme`、`autoScrollEnabled`、`agentPushNotifEnabled`、`remoteControlAtStartup`
- `warnings` 为空数组（若不为空，手工审查并在 Task 3 修正 regex）

**注意**：当前 regex 会同时捕获 sub-object 内部字段（如 `matcher`、`padding`、`allow`、`deny`、`command`、`headers` 等）。这些字段不是顶层 Settings key，Task 5 的 snapshot 对齐测试必须要么把它们列进 `skipped`，要么在这里先细化 extractor 只保留顶层字段。推荐做法：先跑一次 `pnpm extract:schema`，扫描 `settingsFields` 找到所有明显内部字段（如 `matcher`、`padding`、`allow`、`deny`、`ask`、`command`、`url`、`method`、`headers`、`timeout`、`allowedEnvVars`、`type`、`source`、`repo`、`enabled`、`pluginsMarketplace`.`source` 等），把它们一次性加到 Task 5 `skipped` 列表。若发现数量超过 ~40 个，改 extractor 跟踪 zod 嵌套深度（见 Task 1a 下方）。

- [ ] **步骤 6：提交初次快照与脚本**

```bash
git add scripts/extract-claude-schema.ts package.json pnpm-lock.yaml docs/claude-schema-snapshot.json
git commit -m "chore(scripts): add extract-claude-schema + initial snapshot"
```

### Task 1a（可选，若步骤 5 发现 sub-object 字段污染严重则做）：细化 extractor 只保留顶层

- [ ] **步骤 1：在 `extractSettingsFields` 里加嵌套深度追踪**

替换函数体为：
```typescript
function extractSettingsFields(source: string, warnings: string[]): SchemaField[] {
  // 先找"疑似顶层 settings 对象"的起点：通过 'tui' 或 'effortLevel' 这样的确定顶层 marker 锚定
  const anchorRe = /y\.object\(\{[^{}]{0,2000}?tui:y\.enum/;
  const anchorMatch = source.match(anchorRe);
  if (!anchorMatch) {
    warnings.push("Could not find settings schema anchor (tui enum); falling back to whole-file scan");
    return extractSettingsFieldsWholeFile(source, warnings);
  }

  // 从 anchor 向前定位到对应的 y.object({，然后跟踪 {} 深度，只在深度 0 时抽字段
  const anchorIdx = anchorMatch.index!;
  // 找最近的 y.object({ 之前的位置
  let objectStart = source.lastIndexOf("y.object({", anchorIdx);
  if (objectStart < 0) objectStart = anchorIdx;

  // 扫描对象体字段（只在深度 1 时）
  const fields: SchemaField[] = [];
  const seen = new Set<string>();
  let depth = 0;
  let i = objectStart + "y.object({".length;
  depth = 1;

  // 抓取顶层字段的正则（在扫描时逐字段匹配而非全文一次扫完）
  const topRe = /([a-zA-Z_][a-zA-Z0-9_]{1,40}):y\.(string|boolean|number|enum|array|object|record|union|literal|any)\(/g;
  while (i < source.length && depth > 0) {
    const ch = source[i];
    if (ch === "{") depth++;
    else if (ch === "}") depth--;
    else if (depth === 1) {
      topRe.lastIndex = i;
      const m = topRe.exec(source);
      if (m && m.index === i) {
        const name = m[1];
        const type = m[2];
        if (!seen.has(name)) {
          seen.add(name);
          const tail = source.slice(m.index + m[0].length, m.index + m[0].length + 400);
          const optional = /\)\.optional\(\)/.test(tail);
          let enumValues: string[] | undefined;
          if (type === "enum") {
            const em = source.slice(m.index, m.index + 400).match(/y\.enum\(\[([^\]]+)\]/);
            if (em) enumValues = em[1].split(",").map((s) => s.trim().replace(/^"|"$/g, ""));
          }
          const dm = tail.match(/\.describe\(["'`]([^"'`]+)["'`]\)/);
          fields.push({ name, type, optional, enumValues, describe: dm?.[1] });
        }
        i = m.index + m[0].length;
        continue;
      }
    }
    i++;
  }

  return fields;
}

function extractSettingsFieldsWholeFile(source: string, warnings: string[]): SchemaField[] {
  // 原 regex 实现，作 fallback
  // ...（保留原实现）
  return [];
}
```

> 说明：anchor 策略依赖 `tui` 字段存在。若某天 `tui` 改名，anchor 找不到 → 打 warning + fallback 到 whole-file 扫描，仍可跑但可能含 sub-object 字段。把 fallback 实现成原 regex 逻辑。

- [ ] **步骤 2：重新抽取并确认顶层字段数量在合理区间（~50-60，不含 sub-object 内部字段）**

```bash
pnpm extract:schema
```
人工审查 `settingsFields` 长度：应 < 70。若含 `matcher` / `padding` / `allow` / `deny` / `command` 等明显 sub-object 字段，说明 anchor 没命中或 depth 追踪漏算，继续调试。

- [ ] **步骤 3：提交**

```bash
git add scripts/extract-claude-schema.ts docs/claude-schema-snapshot.json
git commit -m "chore(scripts): scope schema extraction to top-level settings"
```

### Task 2：Check 脚本（漂移检测）

- [ ] **步骤 1：创建 `scripts/check-claude-schema.ts`**

```typescript
#!/usr/bin/env tsx
// scripts/check-claude-schema.ts
//
// 抽当前本机 Claude Code schema，对比 docs/claude-schema-snapshot.json。
// 有 diff → 退出码 1，并打印变更清单。

import { readFileSync, existsSync, writeFileSync } from "node:fs";
import { execSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";

const SNAPSHOT_PATH = "docs/claude-schema-snapshot.json";

interface Field {
  name: string;
  type: string;
  enumValues?: string[];
  describe?: string;
}

interface Snapshot {
  claudeCodeVersion: string;
  settingsFields: Field[];
  globalFields: (Field & { source: string })[];
  warnings: string[];
}

function loadSnapshot(path: string): Snapshot {
  if (!existsSync(path)) {
    throw new Error(
      `Snapshot not found at ${path}. Run 'pnpm extract:schema' to create baseline.`
    );
  }
  return JSON.parse(readFileSync(path, "utf8"));
}

function extractFresh(): Snapshot {
  const tmpOut = join(tmpdir(), `claude-schema-${Date.now()}.json`);
  execSync(`tsx scripts/extract-claude-schema.ts --out ${tmpOut}`, {
    stdio: ["ignore", "ignore", "inherit"],
  });
  return JSON.parse(readFileSync(tmpOut, "utf8"));
}

function diffFields(prev: Field[], next: Field[], label: string): string[] {
  const lines: string[] = [];
  const prevMap = new Map(prev.map((f) => [f.name, f]));
  const nextMap = new Map(next.map((f) => [f.name, f]));

  for (const [name, f] of nextMap) {
    const before = prevMap.get(name);
    if (!before) {
      lines.push(
        `+ [${label}] ${name}: ${f.type}${
          f.enumValues ? ` enum=${JSON.stringify(f.enumValues)}` : ""
        }`
      );
    } else {
      if (before.type !== f.type) {
        lines.push(`~ [${label}] ${name} type: ${before.type} → ${f.type}`);
      }
      const beforeEnum = JSON.stringify(before.enumValues ?? []);
      const afterEnum = JSON.stringify(f.enumValues ?? []);
      if (beforeEnum !== afterEnum) {
        lines.push(`~ [${label}] ${name} enum: ${beforeEnum} → ${afterEnum}`);
      }
    }
  }
  for (const [name] of prevMap) {
    if (!nextMap.has(name)) {
      lines.push(`- [${label}] ${name} (removed)`);
    }
  }
  return lines;
}

function main() {
  const isUpdate = process.argv.includes("--update");
  const prev = isUpdate ? null : loadSnapshot(SNAPSHOT_PATH);
  const next = extractFresh();

  if (isUpdate) {
    writeFileSync(SNAPSHOT_PATH, JSON.stringify(next, null, 2) + "\n");
    console.log(`Updated snapshot. Version: ${next.claudeCodeVersion}`);
    return;
  }

  const diffs = [
    ...diffFields(prev!.settingsFields, next.settingsFields, "settings"),
    ...diffFields(
      prev!.globalFields.map((f) => ({ ...f })),
      next.globalFields.map((f) => ({ ...f })),
      "global"
    ),
  ];

  if (next.warnings.length > 0) {
    console.warn("Warnings while extracting:");
    for (const w of next.warnings) console.warn("  " + w);
  }

  if (diffs.length === 0) {
    console.log(
      `✓ Schema unchanged vs snapshot. Version: ${prev!.claudeCodeVersion} → ${next.claudeCodeVersion}`
    );
    process.exit(0);
  }

  console.log(`Schema drift detected (${prev!.claudeCodeVersion} → ${next.claudeCodeVersion}):`);
  for (const d of diffs) console.log("  " + d);
  console.log("");
  console.log("To accept these changes, run: pnpm check:schema --update");
  process.exit(1);
}

main();
```

- [ ] **步骤 2：验证 0-diff 场景**

```bash
pnpm check:schema
```
Expected:
```
✓ Schema unchanged vs snapshot. Version: 2.1.112 → 2.1.112
```
退出码 0。

- [ ] **步骤 3：人为破坏快照验证 diff 能报出来**

手工在 `docs/claude-schema-snapshot.json` 的 `settingsFields` 数组**第一项**删除（保留文件结构）。跑：
```bash
pnpm check:schema
```
Expected：非 0 退出码，输出类似 `- [settings] <fieldname> (removed)`。

- [ ] **步骤 4：还原快照**

```bash
pnpm extract:schema
```
再跑 `pnpm check:schema` 应回到 ✓。

- [ ] **步骤 5：验证 `--update` 模式**

```bash
pnpm check:schema --update
```
Expected：`Updated snapshot. Version: <version>`，文件被刷新。

- [ ] **步骤 6：提交 check 脚本**

```bash
git add scripts/check-claude-schema.ts package.json
git commit -m "chore(scripts): add check-claude-schema drift detector"
```

### Task 3：脚本 README

- [ ] **步骤 1：创建 `scripts/README.md`**

```markdown
# scripts/

内部脚本。运行方式：`pnpm <script-name>`。

## Schema 漂移工具

`extract-claude-schema.ts` / `check-claude-schema.ts` 用于跟踪 Claude Code
`settings.json` schema 随版本变化。

### 常用命令

```bash
pnpm extract:schema          # 抽取当前本机 @anthropic-ai/claude-code 的字段清单
                             # 输出到 docs/claude-schema-snapshot.json
pnpm check:schema            # 对比本机 schema 与快照，diff 非空则退出码 1
pnpm check:schema --update   # 抽取并直接覆盖快照（用于接受新字段后刷基线）
```

### 工作流

1. 运行 `claude update` 或 `npm i -g @anthropic-ai/claude-code@latest` 升级本机 Claude Code
2. 跑 `pnpm check:schema`。若有 diff：
   - 新字段 → 根据重要程度决定直接补进 Settings struct（M2/M3/M4/M8 的 tab），或开新 spec
   - 枚举值变更 → 更新对应下拉组件的合法值白名单
   - 字段类型变更 → 处理破坏性变更（可能影响已序列化 settings.json）
3. 补完字段建模后 `pnpm check:schema --update` 刷新快照并提交

### 查找 cli.js

默认顺序：
1. `CLAUDE_CODE_CLI_JS` 环境变量
2. `$(pnpm root -g)/@anthropic-ai/claude-code/cli.js`
3. `$(npm root -g)/@anthropic-ai/claude-code/cli.js`
4. `~/.bun/install/global/node_modules/@anthropic-ai/claude-code/cli.js`

若全部失败，脚本会报错并提示安装。可以显式指定：
```bash
CLAUDE_CODE_CLI_JS=/path/to/cli.js pnpm extract:schema
```

### Regex 维护

脚本靠 regex 从 minified `cli.js` 抽 zod 字段。如果某字段没被抽到但实际存在，
`snapshot.warnings` 会列出。此时：
1. 手工 grep 该字段在 cli.js 里的实际 pattern
2. 升级 `extractSettingsFields` / `extractGlobalFields` 的 regex
3. 重跑 `pnpm extract:schema` 确认字段进入
```

- [ ] **步骤 2：提交**

```bash
git add scripts/README.md
git commit -m "docs(scripts): add schema tooling README"
```

### Task 4：GitHub Actions Workflow

- [ ] **步骤 1：创建 `.github/workflows/schema-drift.yml`**

```yaml
name: Claude schema drift

on:
  schedule:
    # 每周一 UTC 02:00 跑一次
    - cron: "0 2 * * 1"
  workflow_dispatch: {}

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: 20

      - uses: pnpm/action-setup@v4
        with:
          version: 9
          run_install: false

      - name: Install deps
        run: pnpm install --frozen-lockfile

      - name: Install latest Claude Code
        run: npm i -g @anthropic-ai/claude-code@latest

      - name: Check schema drift
        run: pnpm check:schema
```

- [ ] **步骤 2：提交**

```bash
git add .github/workflows/schema-drift.yml
git commit -m "ci: add weekly Claude schema drift workflow"
```

### Task 5：Rust snapshot 对齐测试

- [ ] **步骤 1：写 fail 测试，断言快照的 settingsFields 在 Rust struct 里都有同名字段或在跳过列表**

编辑 `crates/claude-types/src/settings.rs`，在文件末尾 `mod tests` 内部末尾追加：
```rust
    #[test]
    fn settings_struct_matches_schema_snapshot() {
        // 读 docs/claude-schema-snapshot.json 的 settingsFields，
        // 断言每个字段在 Settings 里有同名字段或在跳过列表里。
        let snapshot_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/claude-schema-snapshot.json");
        let snapshot_raw = std::fs::read_to_string(&snapshot_path)
            .expect("schema snapshot should exist — run `pnpm extract:schema`");
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot_raw).expect("snapshot should be valid JSON");

        let fields = snapshot["settingsFields"]
            .as_array()
            .expect("settingsFields should be an array")
            .iter()
            .map(|f| f["name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();

        // 当前（M1 结束时）已建模的顶层字段：
        let modeled: &[&str] = &[
            "env",
            "includeCoAuthoredBy",
            "permissions",
            "hooks",
            "deniedMcpServers",
            "statusLine",
            "enabledPlugins",
            "extraKnownMarketplaces",
            "language",
            "alwaysThinkingEnabled",
            "autoUpdatesChannel",
            "minimumVersion",
            "skipDangerousModePermissionPrompt",
            "sandbox",
            "modelOverrides",
        ];

        // 在后续里程碑中添加字段时，从 `skipped` 列表移除并加到 `modeled`。
        // `skipped` 当前包含所有未建模字段，包括 M2-M8 将要处理的。
        // M9 完成后，`skipped` 应仅剩真正走 `extra` 兜底的非顶层字段（理论上空）。
        let skipped: &[&str] = &[
            "$schema",
            // M2: tui, effortLevel
            "tui", "effortLevel",
            // M3: Runtime tab
            "model", "outputStyle", "fastMode", "fastModePerSessionOptIn",
            "availableModels", "autoCompactWindow", "showClearContextOnPlanAccept",
            "promptSuggestionEnabled",
            // M4: General extension
            "autoMemoryEnabled", "includeGitInstructions", "respectGitignore",
            "cleanupPeriodDays", "claudeMdExcludes", "plansDirectory",
            "syntaxHighlightingDisabled",
            // M5: MCP tab
            "allowedMcpServers", "enabledMcpjsonServers", "disabledMcpjsonServers",
            "enableAllProjectMcpServers", "allowManagedMcpServersOnly",
            // M6: Plugins & Marketplace
            "strictKnownMarketplaces", "blockedMarketplaces", "skippedMarketplaces",
            "skippedPlugins", "pluginConfigs", "pluginTrustMessage", "skillOverrides",
            // M7: Hooks Policy
            "disableAllHooks", "allowedHttpHookUrls", "httpHookAllowedEnvVars",
            "allowManagedHooksOnly", "allowManagedPermissionRulesOnly",
            "disableSkillShellExecution",
            // M8: Advanced (Tier 3 long tail)
            "apiKeyHelper", "awsCredentialExport", "awsAuthRefresh",
            "gcpAuthRefresh", "forceLoginMethod", "forceLoginOrgUUID",
            "otelHeadersHelper", "prefersReducedMotion", "companyAnnouncements",
            "feedbackSurveyRate", "terminalTitleFromRename", "awaySummaryEnabled",
            "showThinkingSummaries", "advisorModel", "agent", "autoDreamEnabled",
            "autoMemoryDirectory", "skillListingBudgetFraction",
            "skillListingMaxDescChars", "skipWebFetchPreflight",
            "forceRemoteSettingsRefresh",
            // M8: sub-objects stored as serde_json::Value
            "attribution", "autoMode", "fileSuggestion", "worktree",
            "subagentStatusLine", "spinnerVerbs", "spinnerTipsOverride", "remote",
            // M6: pluginConfigs (sub-object stored as Value, UI in M6)
            // (already listed above)
            // Defaults / additions that may appear and are not actionable in this plan:
            "disableBypassPermissionsMode", "disableDeepLinkRegistration",
            "additionalDirectories", "symlinkDirectories", "channelsEnabled",
            "allowedChannelPlugins", "voice",
        ];

        let missing: Vec<&String> = fields
            .iter()
            .filter(|f| !modeled.contains(&f.as_str()) && !skipped.contains(&f.as_str()))
            .collect();

        assert!(
            missing.is_empty(),
            "Settings struct missing fields from snapshot: {:?}",
            missing
        );
    }
```

- [ ] **步骤 2：运行测试确认通过（因跳过列表已包含所有未建模字段）**

```bash
cargo test -p claude-types settings_struct_matches_schema_snapshot
```
Expected: `test result: ok. 1 passed`

- [ ] **步骤 3：人为破坏：在快照 settingsFields 里加一个假字段 `fakeField`**

手工编辑 `docs/claude-schema-snapshot.json`，在 `settingsFields` 末尾加一项 `{"name":"fakeField","type":"string"}`。

再跑测试：
```bash
cargo test -p claude-types settings_struct_matches_schema_snapshot
```
Expected: 失败，输出 `Settings struct missing fields from snapshot: ["fakeField"]`

- [ ] **步骤 4：还原快照**

```bash
pnpm extract:schema
cargo test -p claude-types settings_struct_matches_schema_snapshot
```
Expected：PASS

- [ ] **步骤 5：提交**

```bash
git add crates/claude-types/src/settings.rs
git commit -m "test(claude-types): assert Settings matches schema snapshot"
```

### Task 6：M1 里程碑收尾

- [ ] **步骤 1：跑所有 Rust 测试**

```bash
cargo test --workspace
```
Expected：全绿

- [ ] **步骤 2：跑前端构建**

```bash
pnpm build
```
Expected：无错误（schema 工具改动不应影响前端）

- [ ] **步骤 3：推送分支、打里程碑 tag**

```bash
git push
git tag milestone/M1-schema-tooling
git push --tags
```

---

## Milestone 2 — `tui` 与 `effortLevel`（release notes 2.1.110+）

**目标**：Rust Settings 新增 `tui`、`effort_level` 两个顶层字段；TS Settings 同步；GeneralEditor 添加下拉控件；zh-CN / en-US / ja-JP 三语 i18n；端到端验证。

**Files:**
- Modify: `crates/claude-types/src/settings.rs`（加两字段 + 类型化测试）
- Modify: `src/lib/api/types.ts`（加两字段到 Settings interface）
- Modify: `src/lib/components/settings/GeneralEditor.svelte`（加两个下拉）
- Modify: `src/lib/locales/zh-CN.json` · `en-US.json` · `ja-JP.json`
- Modify: `tests/fixtures/settings-full.json`（加两字段覆盖）
- Modify: `crates/claude-types/src/settings.rs`（更新 `skipped` 列表，移除 `tui` / `effortLevel`）

### Task 7：加 Rust 字段 + 类型化断言

- [ ] **步骤 1：写 fail 测试**

在 `crates/claude-types/src/settings.rs` 的 `mod tests` 内部末尾追加：
```rust
    #[test]
    fn parse_tui_field_is_typed() {
        let s: Settings =
            serde_json::from_str(r#"{"tui":"fullscreen"}"#).expect("should parse");
        assert_eq!(s.tui.as_deref(), Some("fullscreen"));
        assert!(!s.extra.contains_key("tui"));
    }

    #[test]
    fn parse_effort_level_is_typed() {
        let s: Settings =
            serde_json::from_str(r#"{"effortLevel":"xhigh"}"#).expect("should parse");
        assert_eq!(s.effort_level.as_deref(), Some("xhigh"));
        assert!(!s.extra.contains_key("effortLevel"));
    }
```

- [ ] **步骤 2：运行测试确认失败**

```bash
cargo test -p claude-types parse_tui_field_is_typed
```
Expected：编译失败 `no field 'tui' on type 'Settings'`

- [ ] **步骤 3：加字段到 Settings struct**

在 `crates/claude-types/src/settings.rs:58` 的 `model_overrides` 之后、`extra` 之前插入：
```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_level: Option<String>,
```

- [ ] **步骤 4：运行测试确认通过**

```bash
cargo test -p claude-types parse_tui_field_is_typed parse_effort_level_is_typed
```
Expected：两个测试 PASS

- [ ] **步骤 5：更新 snapshot 对齐测试的跳过列表**

在步骤 2.5 `settings_struct_matches_schema_snapshot` 测试里，从 `skipped` 列表移除 `"tui", "effortLevel",`（它们现在是 modeled）。在 `modeled` 列表里追加 `"tui", "effortLevel",`。

```bash
cargo test -p claude-types settings_struct_matches_schema_snapshot
```
Expected：PASS

- [ ] **步骤 6：提交**

```bash
git add crates/claude-types/src/settings.rs
git commit -m "feat(claude-types): add tui and effortLevel fields"
```

### Task 8：TS 类型同步

- [ ] **步骤 1：在 `src/lib/api/types.ts:10-28` 的 `Settings` interface 追加字段**

在 `modelOverrides?: Record<string, unknown>;` 之后、`[key: string]: unknown;` 之前插入：
```typescript
  tui?: "default" | "fullscreen";
  effortLevel?: "low" | "medium" | "high" | "xhigh";
```

- [ ] **步骤 2：验证 TS 编译**

```bash
pnpm build
```
Expected：无 TS 错误

- [ ] **步骤 3：提交**

```bash
git add src/lib/api/types.ts
git commit -m "feat(types): add tui and effortLevel to Settings interface"
```

### Task 9：i18n 键

- [ ] **步骤 1：在 `src/lib/locales/zh-CN.json` 追加键**

```json
  "settings.fields.tui.label": "TUI 渲染器",
  "settings.fields.tui.tooltip": "终端 UI 渲染器：fullscreen 使用无闪烁 alt-screen 渲染，default 走经典渲染",
  "settings.fields.effortLevel.label": "Effort 等级",
  "settings.fields.effortLevel.tooltip": "持久化的 effort 等级（适配模型时生效）。xhigh 仅 Opus 4.7 专属，其他模型 fallback 到 high",
```

- [ ] **步骤 2：在 `src/lib/locales/en-US.json` 追加键**

```json
  "settings.fields.tui.label": "TUI Renderer",
  "settings.fields.tui.tooltip": "Terminal UI renderer: 'fullscreen' for flicker-free alt-screen rendering, 'default' for the classic renderer",
  "settings.fields.effortLevel.label": "Effort Level",
  "settings.fields.effortLevel.tooltip": "Persisted effort level for supported models. 'xhigh' is Opus 4.7 only; other models fall back to 'high'",
```

- [ ] **步骤 3：在 `src/lib/locales/ja-JP.json` 追加键**

```json
  "settings.fields.tui.label": "TUI レンダラー",
  "settings.fields.tui.tooltip": "ターミナル UI レンダラー：fullscreen はちらつきのない alt-screen 描画、default は従来の描画",
  "settings.fields.effortLevel.label": "Effort レベル",
  "settings.fields.effortLevel.tooltip": "対応モデルで永続化される effort レベル。xhigh は Opus 4.7 専用、他モデルは high にフォールバック",
```

- [ ] **步骤 4：验证 pnpm build 无 i18n 错误**

```bash
pnpm build
```
Expected：无错误

- [ ] **步骤 5：提交**

```bash
git add src/lib/locales/zh-CN.json src/lib/locales/en-US.json src/lib/locales/ja-JP.json
git commit -m "i18n: add tui and effortLevel labels"
```

### Task 10：GeneralEditor 加控件

- [ ] **步骤 1：在 `src/lib/components/settings/GeneralEditor.svelte` 的 `<script>` 节追加状态**

在现有的 `let skipDangerousModePermissionPrompt = $state(...)` 后、`$effect` 之前插入：
```typescript
  let tui = $state((settings.tui as string) ?? "");
  let effortLevel = $state((settings.effortLevel as string) ?? "");
```

在现有的 `$effect(() => {...})` 里追加两行赋值：
```typescript
    tui = (settings.tui as string) ?? "";
    effortLevel = (settings.effortLevel as string) ?? "";
```

- [ ] **步骤 2：追加 dirty derived**

在现有 `const skipDangerousDirty = $derived(...)` 之后：
```typescript
  const tuiDirty = $derived(tui !== ((settings.tui as string) ?? ""));
  const effortLevelDirty = $derived(
    effortLevel !== ((settings.effortLevel as string) ?? ""),
  );
```

- [ ] **步骤 3：在 `previewData` 和 `save()` 里追加**

```typescript
  const previewData = $derived({
    language: language || undefined,
    alwaysThinkingEnabled,
    autoUpdatesChannel,
    minimumVersion: minimumVersion || undefined,
    includeCoAuthoredBy,
    skipDangerousModePermissionPrompt,
    tui: tui || undefined,
    effortLevel: effortLevel || undefined,
  });

  function save() {
    configStore.save({
      language: language || undefined,
      alwaysThinkingEnabled,
      autoUpdatesChannel,
      minimumVersion: minimumVersion || undefined,
      includeCoAuthoredBy,
      skipDangerousModePermissionPrompt,
      tui: (tui || undefined) as "default" | "fullscreen" | undefined,
      effortLevel: (effortLevel || undefined) as
        | "low"
        | "medium"
        | "high"
        | "xhigh"
        | undefined,
    });
  }
```

- [ ] **步骤 4：在模板（template 部分）的 "Skip Dangerous Mode Permission Prompt" checkbox 之后、Save/Revert 按钮之前插入两个下拉**

```svelte
  <!-- TUI Renderer -->
  <div class="space-y-1">
    <label
      for="tui"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
      title={t("settings.fields.tui.tooltip")}
    >
      {t("settings.fields.tui.label")}
      <DirtyDot dirty={tuiDirty} />
    </label>
    <select
      id="tui"
      bind:value={tui}
      onchange={() => configStore.markDirty()}
      class="input-base"
    >
      <option value="">(unset)</option>
      <option value="default">default</option>
      <option value="fullscreen">fullscreen</option>
    </select>
  </div>

  <!-- Effort Level -->
  <div class="space-y-1">
    <label
      for="effortLevel"
      class="block text-sm font-medium" style="color: var(--text-secondary)"
      title={t("settings.fields.effortLevel.tooltip")}
    >
      {t("settings.fields.effortLevel.label")}
      <DirtyDot dirty={effortLevelDirty} />
    </label>
    <select
      id="effortLevel"
      bind:value={effortLevel}
      onchange={() => configStore.markDirty()}
      class="input-base"
    >
      <option value="">(unset)</option>
      <option value="low">low</option>
      <option value="medium">medium</option>
      <option value="high">high</option>
      <option value="xhigh">xhigh (Opus 4.7)</option>
    </select>
  </div>
```

- [ ] **步骤 5：重启 dev server（HMR 对新 $state 不可靠）**

如果已有 `pnpm tauri dev` 在运行，Ctrl+C 终止并重启：
```bash
pnpm tauri dev
```

- [ ] **步骤 6：手动冒烟**

1. 应用启动后打开 Settings → General
2. 向下滚动找到 "TUI Renderer" 和 "Effort Level"
3. 把 TUI 选成 `fullscreen`，把 Effort 选成 `xhigh`
4. 点 Save
5. Tauri DevTools Console：无报错
6. 终端打开 `~/.claude/settings.json`（或当前 scope 对应文件）：应含 `"tui": "fullscreen"`、`"effortLevel": "xhigh"`
7. 刷新应用（Cmd+R），字段应保持 `fullscreen` / `xhigh`

- [ ] **步骤 7：提交**

```bash
git add src/lib/components/settings/GeneralEditor.svelte
git commit -m "feat(settings-ui): add tui and effortLevel selectors in General"
```

### Task 11：fixture 覆盖 + 里程碑收尾

- [ ] **步骤 1：更新 `tests/fixtures/settings-full.json`，在根对象内（`modelOverrides` 后）追加**

```json
  "tui": "fullscreen",
  "effortLevel": "xhigh",
```

- [ ] **步骤 2：重跑 roundtrip**

```bash
cargo test -p claude-types parse_full_settings
```
Expected：PASS（fixture 的新字段被显式 struct 接收，不落入 extra）

- [ ] **步骤 3：最终验证**

```bash
cargo test --workspace
pnpm build
```
Expected：全绿

- [ ] **步骤 4：手动端到端验证用真实 Claude Code**

退出 `pnpm tauri dev`，启动真实 Claude Code CLI：
```bash
claude --help
```
Expected：不报 schema validation 错误（只要没有 "Unknown field" 之类的抱怨即可）。

- [ ] **步骤 5：打里程碑 tag**

```bash
git add tests/fixtures/settings-full.json
git commit -m "test: cover tui and effortLevel in settings fixture"
git push
git tag milestone/M2-tui-effortlevel
git push --tags
```

---

## Milestone 3 — Runtime tab

**目标**：新增 Runtime tab，容纳 8 个运行时相关字段。

**新字段**：`model`、`outputStyle`、`fastMode`、`fastModePerSessionOptIn`、`availableModels`、`autoCompactWindow`、`showClearContextOnPlanAccept`、`promptSuggestionEnabled`

**Files:**
- Modify: `crates/claude-types/src/settings.rs`
- Modify: `src/lib/api/types.ts`
- Create: `src/lib/components/settings/RuntimeEditor.svelte`
- Modify: `src/lib/components/settings/SettingsEditor.svelte`
- Modify: `src/App.svelte`（扩展 `settingsSections` 加 `"runtime"` 项）
- Modify: `src/lib/locales/{zh-CN,en-US,ja-JP}.json`（加 `settings.runtime` + 8 字段 label/tooltip + tab label）
- Modify: `tests/fixtures/settings-full.json`
- Modify: `crates/claude-types/src/settings.rs`（snapshot test 的 modeled/skipped 列表）

### Task 12：Rust 字段添加（含类型化测试）

- [ ] **步骤 1：写 fail 测试**

在 `crates/claude-types/src/settings.rs` 的 `mod tests` 内末尾追加：
```rust
    #[test]
    fn parse_runtime_fields_are_typed() {
        let json = r#"{
            "model": "opus",
            "outputStyle": "default",
            "fastMode": true,
            "fastModePerSessionOptIn": false,
            "availableModels": ["sonnet", "opus"],
            "autoCompactWindow": 200000,
            "showClearContextOnPlanAccept": true,
            "promptSuggestionEnabled": false
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(s.model.as_deref(), Some("opus"));
        assert_eq!(s.output_style.as_deref(), Some("default"));
        assert_eq!(s.fast_mode, Some(true));
        assert_eq!(s.fast_mode_per_session_opt_in, Some(false));
        assert_eq!(
            s.available_models.as_ref().map(Vec::as_slice),
            Some(&["sonnet".to_string(), "opus".to_string()][..])
        );
        assert_eq!(s.auto_compact_window, Some(200000));
        assert_eq!(s.show_clear_context_on_plan_accept, Some(true));
        assert_eq!(s.prompt_suggestion_enabled, Some(false));
        // 未知字段兜底 map 不含这些 key
        for k in [
            "model",
            "outputStyle",
            "fastMode",
            "fastModePerSessionOptIn",
            "availableModels",
            "autoCompactWindow",
            "showClearContextOnPlanAccept",
            "promptSuggestionEnabled",
        ] {
            assert!(!s.extra.contains_key(k), "{} should be typed, not in extra", k);
        }
    }
```

- [ ] **步骤 2：运行测试确认失败**

```bash
cargo test -p claude-types parse_runtime_fields_are_typed
```
Expected：编译失败 `no field 'model' on type 'Settings'`

- [ ] **步骤 3：加字段到 `Settings` struct（在 `tui` / `effort_level` 之后、`extra` 之前）**

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_mode: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_mode_per_session_opt_in: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_models: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_compact_window: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_clear_context_on_plan_accept: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_suggestion_enabled: Option<bool>,
```

- [ ] **步骤 4：更新 snapshot 测试的 modeled/skipped 列表**

在 `settings_struct_matches_schema_snapshot` 测试里：
- `modeled` 列表追加：`"model", "outputStyle", "fastMode", "fastModePerSessionOptIn", "availableModels", "autoCompactWindow", "showClearContextOnPlanAccept", "promptSuggestionEnabled",`
- `skipped` 列表移除这 8 个名字

- [ ] **步骤 5：跑测试**

```bash
cargo test -p claude-types
```
Expected：全绿（新增测试 + snapshot 对齐测试都 PASS）

- [ ] **步骤 6：提交**

```bash
git add crates/claude-types/src/settings.rs
git commit -m "feat(claude-types): add 8 runtime fields"
```

### Task 13：TS 类型同步

- [ ] **步骤 1：在 `src/lib/api/types.ts` 的 `Settings` interface 里 `effortLevel?` 之后、`[key: string]: unknown;` 之前追加**

```typescript
  model?: string;
  outputStyle?: string;
  fastMode?: boolean;
  fastModePerSessionOptIn?: boolean;
  availableModels?: string[];
  autoCompactWindow?: number;
  showClearContextOnPlanAccept?: boolean;
  promptSuggestionEnabled?: boolean;
```

- [ ] **步骤 2：验证**

```bash
pnpm build
```
Expected：无错误

- [ ] **步骤 3：提交**

```bash
git add src/lib/api/types.ts
git commit -m "feat(types): add 8 runtime fields to Settings interface"
```

### Task 14：i18n 键（zh-CN / en-US / ja-JP）

- [ ] **步骤 1：在 `src/lib/locales/zh-CN.json` 追加**

```json
  "settings.runtime": "运行时",
  "settings.fields.model.label": "模型",
  "settings.fields.model.tooltip": "覆盖默认模型（如 opus / sonnet / haiku / 自定义模型 ID）",
  "settings.fields.outputStyle.label": "输出风格",
  "settings.fields.outputStyle.tooltip": "通过 /output-style 命令写入的输出风格名",
  "settings.fields.fastMode.label": "Fast Mode",
  "settings.fields.fastMode.tooltip": "启用后使用 Fast Mode（Opus 4.6 加速输出）",
  "settings.fields.fastModePerSessionOptIn.label": "Fast Mode 按会话",
  "settings.fields.fastModePerSessionOptIn.tooltip": "为 true 时 Fast Mode 不跨会话持久化，每个新会话默认关闭",
  "settings.fields.availableModels.label": "可用模型列表",
  "settings.fields.availableModels.tooltip": "允许在模型选择器中显示的模型列表",
  "settings.fields.autoCompactWindow.label": "Auto Compact 阈值",
  "settings.fields.autoCompactWindow.tooltip": "自动压缩（compact）触发的上下文 token 阈值（100k-1M）",
  "settings.fields.showClearContextOnPlanAccept.label": "Plan 接受时提示清理上下文",
  "settings.fields.showClearContextOnPlanAccept.tooltip": "为 true 时，plan 批准对话框提供 clear context 选项",
  "settings.fields.promptSuggestionEnabled.label": "启用 Prompt 建议",
  "settings.fields.promptSuggestionEnabled.tooltip": "为 false 禁用提示建议；默认或 true 启用",
```

- [ ] **步骤 2：同样在 `en-US.json` 追加对应英文**

```json
  "settings.runtime": "Runtime",
  "settings.fields.model.label": "Model",
  "settings.fields.model.tooltip": "Override the default model (e.g. opus / sonnet / haiku / custom model id)",
  "settings.fields.outputStyle.label": "Output Style",
  "settings.fields.outputStyle.tooltip": "Output style name set via /output-style",
  "settings.fields.fastMode.label": "Fast Mode",
  "settings.fields.fastMode.tooltip": "Enable Fast Mode (Opus 4.6 accelerated output)",
  "settings.fields.fastModePerSessionOptIn.label": "Fast Mode Per Session",
  "settings.fields.fastModePerSessionOptIn.tooltip": "When true, Fast Mode does not persist across sessions; each session starts with Fast Mode off",
  "settings.fields.availableModels.label": "Available Models",
  "settings.fields.availableModels.tooltip": "Models allowed in the model picker",
  "settings.fields.autoCompactWindow.label": "Auto Compact Window",
  "settings.fields.autoCompactWindow.tooltip": "Context token threshold that triggers auto-compact (100k–1M)",
  "settings.fields.showClearContextOnPlanAccept.label": "Show Clear Context on Plan Accept",
  "settings.fields.showClearContextOnPlanAccept.tooltip": "When true, the plan-approval dialog offers a 'clear context' option",
  "settings.fields.promptSuggestionEnabled.label": "Prompt Suggestion Enabled",
  "settings.fields.promptSuggestionEnabled.tooltip": "When false, prompt suggestions are disabled; when absent or true, they are enabled",
```

- [ ] **步骤 3：在 `ja-JP.json` 追加对应日文翻译**

```json
  "settings.runtime": "ランタイム",
  "settings.fields.model.label": "モデル",
  "settings.fields.model.tooltip": "既定モデルを上書き（例: opus / sonnet / haiku / カスタムモデル ID）",
  "settings.fields.outputStyle.label": "出力スタイル",
  "settings.fields.outputStyle.tooltip": "/output-style コマンドで設定される出力スタイル名",
  "settings.fields.fastMode.label": "Fast Mode",
  "settings.fields.fastMode.tooltip": "Fast Mode を有効化（Opus 4.6 高速出力）",
  "settings.fields.fastModePerSessionOptIn.label": "Fast Mode（セッション毎）",
  "settings.fields.fastModePerSessionOptIn.tooltip": "true の場合、Fast Mode はセッションをまたがず、新規セッションでは既定オフ",
  "settings.fields.availableModels.label": "利用可能モデル",
  "settings.fields.availableModels.tooltip": "モデル選択に表示されるモデルリスト",
  "settings.fields.autoCompactWindow.label": "自動コンパクト閾値",
  "settings.fields.autoCompactWindow.tooltip": "自動コンパクトが発動するトークン閾値（100k–1M）",
  "settings.fields.showClearContextOnPlanAccept.label": "プラン承認時にコンテキスト消去を表示",
  "settings.fields.showClearContextOnPlanAccept.tooltip": "true の場合、プラン承認ダイアログで 'clear context' オプションを表示",
  "settings.fields.promptSuggestionEnabled.label": "プロンプト提案を有効化",
  "settings.fields.promptSuggestionEnabled.tooltip": "false でプロンプト提案を無効化、未指定または true で有効化",
```

- [ ] **步骤 4：跑 build 验证**

```bash
pnpm build
```
Expected：无错误

- [ ] **步骤 5：提交**

```bash
git add src/lib/locales/
git commit -m "i18n: add Runtime tab and 8 runtime field keys"
```

### Task 15：RuntimeEditor.svelte 组件

- [ ] **步骤 1：创建 `src/lib/components/settings/RuntimeEditor.svelte`**

```svelte
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";

  const settings = $derived(configStore.activeSettings);

  let model = $state((settings.model as string) ?? "");
  let outputStyle = $state((settings.outputStyle as string) ?? "");
  let fastMode = $state((settings.fastMode as boolean) ?? false);
  let fastModePerSessionOptIn = $state(
    (settings.fastModePerSessionOptIn as boolean) ?? false,
  );
  let availableModelsText = $state(
    ((settings.availableModels as string[]) ?? []).join("\n"),
  );
  let autoCompactWindow = $state((settings.autoCompactWindow as number | undefined) ?? "");
  let showClearContextOnPlanAccept = $state(
    (settings.showClearContextOnPlanAccept as boolean) ?? false,
  );
  let promptSuggestionEnabled = $state(
    (settings.promptSuggestionEnabled as boolean) ?? false,
  );

  $effect(() => {
    model = (settings.model as string) ?? "";
    outputStyle = (settings.outputStyle as string) ?? "";
    fastMode = (settings.fastMode as boolean) ?? false;
    fastModePerSessionOptIn = (settings.fastModePerSessionOptIn as boolean) ?? false;
    availableModelsText = ((settings.availableModels as string[]) ?? []).join("\n");
    autoCompactWindow = (settings.autoCompactWindow as number | undefined) ?? "";
    showClearContextOnPlanAccept =
      (settings.showClearContextOnPlanAccept as boolean) ?? false;
    promptSuggestionEnabled =
      (settings.promptSuggestionEnabled as boolean) ?? false;
  });

  const modelDirty = $derived(model !== ((settings.model as string) ?? ""));
  const outputStyleDirty = $derived(
    outputStyle !== ((settings.outputStyle as string) ?? ""),
  );
  const fastModeDirty = $derived(
    fastMode !== ((settings.fastMode as boolean) ?? false),
  );
  const fastModePerSessionDirty = $derived(
    fastModePerSessionOptIn !==
      ((settings.fastModePerSessionOptIn as boolean) ?? false),
  );
  const availableModelsDirty = $derived(
    availableModelsText !==
      ((settings.availableModels as string[]) ?? []).join("\n"),
  );
  const autoCompactWindowDirty = $derived(
    String(autoCompactWindow) !==
      String((settings.autoCompactWindow as number | undefined) ?? ""),
  );
  const showClearCtxDirty = $derived(
    showClearContextOnPlanAccept !==
      ((settings.showClearContextOnPlanAccept as boolean) ?? false),
  );
  const promptSuggestionDirty = $derived(
    promptSuggestionEnabled !==
      ((settings.promptSuggestionEnabled as boolean) ?? false),
  );

  function parseAvailableModels(): string[] | undefined {
    const lines = availableModelsText
      .split("\n")
      .map((s) => s.trim())
      .filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  function parsedWindow(): number | undefined {
    const n = Number(autoCompactWindow);
    return Number.isFinite(n) && n > 0 ? n : undefined;
  }

  const previewData = $derived({
    model: model || undefined,
    outputStyle: outputStyle || undefined,
    fastMode,
    fastModePerSessionOptIn,
    availableModels: parseAvailableModels(),
    autoCompactWindow: parsedWindow(),
    showClearContextOnPlanAccept,
    promptSuggestionEnabled,
  });

  function save() {
    configStore.save({
      model: model || undefined,
      outputStyle: outputStyle || undefined,
      fastMode,
      fastModePerSessionOptIn,
      availableModels: parseAvailableModels(),
      autoCompactWindow: parsedWindow(),
      showClearContextOnPlanAccept,
      promptSuggestionEnabled,
    });
  }
</script>

<div class="space-y-5 max-w-xl">
  <div class="space-y-1">
    <label for="model" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.model.tooltip")}>
      {t("settings.fields.model.label")}
      <DirtyDot dirty={modelDirty} />
    </label>
    <input id="model" type="text" bind:value={model}
           oninput={() => configStore.markDirty()}
           placeholder="opus"
           class="input-base" />
  </div>

  <div class="space-y-1">
    <label for="outputStyle" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.outputStyle.tooltip")}>
      {t("settings.fields.outputStyle.label")}
      <DirtyDot dirty={outputStyleDirty} />
    </label>
    <input id="outputStyle" type="text" bind:value={outputStyle}
           oninput={() => configStore.markDirty()}
           class="input-base" />
  </div>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={fastMode}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.fastMode.tooltip")}>
      {t("settings.fields.fastMode.label")}
      <DirtyDot dirty={fastModeDirty} />
    </span>
  </label>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={fastModePerSessionOptIn}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.fastModePerSessionOptIn.tooltip")}>
      {t("settings.fields.fastModePerSessionOptIn.label")}
      <DirtyDot dirty={fastModePerSessionDirty} />
    </span>
  </label>

  <div class="space-y-1">
    <label for="availableModels" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.availableModels.tooltip")}>
      {t("settings.fields.availableModels.label")}
      <DirtyDot dirty={availableModelsDirty} />
    </label>
    <textarea id="availableModels" rows="4"
              bind:value={availableModelsText}
              oninput={() => configStore.markDirty()}
              placeholder={"opus\nsonnet\nhaiku"}
              class="input-base font-mono text-xs"></textarea>
    <p class="text-xs" style="color: var(--text-muted)">one model id per line</p>
  </div>

  <div class="space-y-1">
    <label for="autoCompactWindow" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.autoCompactWindow.tooltip")}>
      {t("settings.fields.autoCompactWindow.label")}
      <DirtyDot dirty={autoCompactWindowDirty} />
    </label>
    <input id="autoCompactWindow" type="number"
           bind:value={autoCompactWindow}
           oninput={() => configStore.markDirty()}
           min="100000" max="1000000"
           placeholder="200000"
           class="input-base" />
  </div>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={showClearContextOnPlanAccept}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.showClearContextOnPlanAccept.tooltip")}>
      {t("settings.fields.showClearContextOnPlanAccept.label")}
      <DirtyDot dirty={showClearCtxDirty} />
    </span>
  </label>

  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={promptSuggestionEnabled}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.promptSuggestionEnabled.tooltip")}>
      {t("settings.fields.promptSuggestionEnabled.label")}
      <DirtyDot dirty={promptSuggestionDirty} />
    </span>
  </label>

  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button type="button" onclick={save}
            disabled={!configStore.isDirty || configStore.saving}
            class="btn-primary text-sm px-4 py-2">
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button type="button" onclick={() => configStore.revert()}
            disabled={!configStore.isDirty}
            class="btn-secondary text-sm px-4 py-2">
      {t("common.revert")}
    </button>
  </div>

  <JsonPreview data={previewData} title={t("settings.runtime") + " JSON"} />
</div>
```

- [ ] **步骤 2：提交**

```bash
git add src/lib/components/settings/RuntimeEditor.svelte
git commit -m "feat(settings-ui): add RuntimeEditor component"
```

### Task 16：注册 Runtime tab

- [ ] **步骤 1：修改 `src/lib/components/settings/SettingsEditor.svelte`**

在 `import EnvVarEditor ...` 之后追加：
```typescript
  import RuntimeEditor from "./RuntimeEditor.svelte";
```

在模板的 `{:else if activeSection === "statusline"}` 块之后插入：
```svelte
  {:else if activeSection === "runtime"}
    <RuntimeEditor />
```

- [ ] **步骤 2：修改 `src/App.svelte:95-102` 的 `settingsSections`**

在 `{ id: "statusline", labelKey: "settings.statusLine" },` 之后追加：
```typescript
    { id: "runtime", labelKey: "settings.runtime" },
```

- [ ] **步骤 3：重启 dev server**

```bash
# 若 pnpm tauri dev 在跑，Ctrl+C
pnpm tauri dev
```

- [ ] **步骤 4：手动冒烟**

1. 侧边导航点 Settings
2. Sub-panel 应出现 "运行时"（或对应语言）导航项
3. 点进去看到 RuntimeEditor 的 8 个控件
4. 随机改 model="opus"、fastMode=on、autoCompactWindow=500000，点 Save
5. DevTools Console：无报错
6. 打开对应 settings.json，3 字段出现
7. 取消 model（清空），Save，`"model"` 从文件消失（因为 `undefined` 不序列化）

- [ ] **步骤 5：提交**

```bash
git add src/lib/components/settings/SettingsEditor.svelte src/App.svelte
git commit -m "feat(settings-ui): register Runtime tab"
```

### Task 17：fixture + 里程碑收尾

- [ ] **步骤 1：更新 `tests/fixtures/settings-full.json`**

在根对象 `effortLevel` 后追加：
```json
  "model": "opus",
  "outputStyle": "default",
  "fastMode": true,
  "fastModePerSessionOptIn": false,
  "availableModels": ["sonnet", "opus", "haiku"],
  "autoCompactWindow": 500000,
  "showClearContextOnPlanAccept": true,
  "promptSuggestionEnabled": true,
```

- [ ] **步骤 2：跑所有测试**

```bash
cargo test --workspace && pnpm build
```
Expected：全绿

- [ ] **步骤 3：打 tag**

```bash
git add tests/fixtures/settings-full.json
git commit -m "test: cover Runtime fields in fixture"
git push
git tag milestone/M3-runtime-tab
git push --tags
```

---

## Milestone 4 — General tab 扩展

**目标**：在现有 GeneralEditor 里追加 7 个内容/文件相关字段，不新开 tab。

**新字段**：`autoMemoryEnabled`、`includeGitInstructions`、`respectGitignore`、`cleanupPeriodDays`、`claudeMdExcludes`、`plansDirectory`、`syntaxHighlightingDisabled`

**Files:**
- Modify: `crates/claude-types/src/settings.rs`
- Modify: `src/lib/api/types.ts`
- Modify: `src/lib/components/settings/GeneralEditor.svelte`
- Modify: `src/lib/locales/{zh-CN,en-US,ja-JP}.json`
- Modify: `tests/fixtures/settings-full.json`

### Task 18：Rust + TS 类型 + 类型化测试

- [ ] **步骤 1：在 `settings.rs` tests 末尾写 fail 测试**

```rust
    #[test]
    fn parse_general_extension_fields_are_typed() {
        let json = r#"{
            "autoMemoryEnabled": true,
            "includeGitInstructions": false,
            "respectGitignore": true,
            "cleanupPeriodDays": 60,
            "claudeMdExcludes": ["vendor/", ".venv/"],
            "plansDirectory": "~/.claude/plans",
            "syntaxHighlightingDisabled": false
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(s.auto_memory_enabled, Some(true));
        assert_eq!(s.include_git_instructions, Some(false));
        assert_eq!(s.respect_gitignore, Some(true));
        assert_eq!(s.cleanup_period_days, Some(60));
        assert_eq!(
            s.claude_md_excludes.as_ref().map(Vec::as_slice),
            Some(&["vendor/".to_string(), ".venv/".to_string()][..])
        );
        assert_eq!(s.plans_directory.as_deref(), Some("~/.claude/plans"));
        assert_eq!(s.syntax_highlighting_disabled, Some(false));
        for k in [
            "autoMemoryEnabled", "includeGitInstructions", "respectGitignore",
            "cleanupPeriodDays", "claudeMdExcludes", "plansDirectory",
            "syntaxHighlightingDisabled",
        ] {
            assert!(!s.extra.contains_key(k));
        }
    }
```

- [ ] **步骤 2：运行测试确认失败**

```bash
cargo test -p claude-types parse_general_extension_fields_are_typed
```

- [ ] **步骤 3：加字段到 Settings struct（在 prompt_suggestion_enabled 后、extra 前）**

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_memory_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_git_instructions: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub respect_gitignore: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_period_days: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_md_excludes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plans_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_highlighting_disabled: Option<bool>,
```

- [ ] **步骤 4：更新 snapshot 测试的 modeled/skipped 列表**

在 `settings_struct_matches_schema_snapshot` 测试里：
- `modeled` 追加：`"autoMemoryEnabled", "includeGitInstructions", "respectGitignore", "cleanupPeriodDays", "claudeMdExcludes", "plansDirectory", "syntaxHighlightingDisabled",`
- `skipped` 移除这 7 个

- [ ] **步骤 5：跑测试**

```bash
cargo test -p claude-types
```
Expected：全绿

- [ ] **步骤 6：TS 同步——在 `src/lib/api/types.ts` 的 Settings interface 追加**

```typescript
  autoMemoryEnabled?: boolean;
  includeGitInstructions?: boolean;
  respectGitignore?: boolean;
  cleanupPeriodDays?: number;
  claudeMdExcludes?: string[];
  plansDirectory?: string;
  syntaxHighlightingDisabled?: boolean;
```

- [ ] **步骤 7：跑 build**

```bash
pnpm build
```

- [ ] **步骤 8：提交**

```bash
git add crates/claude-types/src/settings.rs src/lib/api/types.ts
git commit -m "feat(claude-types,types): add 7 general extension fields"
```

### Task 19：i18n 键

- [ ] **步骤 1：在 `zh-CN.json` 追加**

```json
  "settings.fields.autoMemoryEnabled.label": "启用自动记忆",
  "settings.fields.autoMemoryEnabled.tooltip": "启用 Claude Code 自动 memory 功能",
  "settings.fields.includeGitInstructions.label": "注入 Git 指令",
  "settings.fields.includeGitInstructions.tooltip": "启用后系统提示会附带 Git 操作相关指引",
  "settings.fields.respectGitignore.label": "遵循 .gitignore",
  "settings.fields.respectGitignore.tooltip": "读取/搜索时自动排除 .gitignore 内条目",
  "settings.fields.cleanupPeriodDays.label": "会话保留天数",
  "settings.fields.cleanupPeriodDays.tooltip": "超过此天数的历史会话将被自动清理",
  "settings.fields.claudeMdExcludes.label": "CLAUDE.md 排除路径",
  "settings.fields.claudeMdExcludes.tooltip": "上行搜索 CLAUDE.md 时跳过这些路径（每行一条）",
  "settings.fields.plansDirectory.label": "Plans 目录",
  "settings.fields.plansDirectory.tooltip": "保存 plan 文件的目录路径",
  "settings.fields.syntaxHighlightingDisabled.label": "禁用 diff 语法高亮",
  "settings.fields.syntaxHighlightingDisabled.tooltip": "关闭 diff 视图的语法高亮",
```

- [ ] **步骤 2：同样在 `en-US.json` 追加**

```json
  "settings.fields.autoMemoryEnabled.label": "Enable Auto Memory",
  "settings.fields.autoMemoryEnabled.tooltip": "Enable Claude Code's auto-memory feature",
  "settings.fields.includeGitInstructions.label": "Include Git Instructions",
  "settings.fields.includeGitInstructions.tooltip": "When enabled, the system prompt includes Git operation guidance",
  "settings.fields.respectGitignore.label": "Respect .gitignore",
  "settings.fields.respectGitignore.tooltip": "Skip entries listed in .gitignore during read/search",
  "settings.fields.cleanupPeriodDays.label": "Cleanup Period (Days)",
  "settings.fields.cleanupPeriodDays.tooltip": "Sessions older than this many days are cleaned up",
  "settings.fields.claudeMdExcludes.label": "CLAUDE.md Excludes",
  "settings.fields.claudeMdExcludes.tooltip": "Paths skipped when walking upward to find CLAUDE.md (one per line)",
  "settings.fields.plansDirectory.label": "Plans Directory",
  "settings.fields.plansDirectory.tooltip": "Directory path for storing plan files",
  "settings.fields.syntaxHighlightingDisabled.label": "Disable Diff Syntax Highlighting",
  "settings.fields.syntaxHighlightingDisabled.tooltip": "Turn off syntax highlighting in diff views",
```

- [ ] **步骤 3：同样 `ja-JP.json`**

```json
  "settings.fields.autoMemoryEnabled.label": "自動メモリを有効化",
  "settings.fields.autoMemoryEnabled.tooltip": "Claude Code の自動メモリ機能を有効化",
  "settings.fields.includeGitInstructions.label": "Git 指示を含める",
  "settings.fields.includeGitInstructions.tooltip": "有効時、システムプロンプトに Git 操作ガイダンスを付与",
  "settings.fields.respectGitignore.label": ".gitignore を尊重",
  "settings.fields.respectGitignore.tooltip": "読取/検索で .gitignore のエントリをスキップ",
  "settings.fields.cleanupPeriodDays.label": "セッション保持日数",
  "settings.fields.cleanupPeriodDays.tooltip": "この日数を超えた履歴セッションは自動クリーンアップ",
  "settings.fields.claudeMdExcludes.label": "CLAUDE.md 除外パス",
  "settings.fields.claudeMdExcludes.tooltip": "CLAUDE.md 探索時に除外するパス（1 行 1 件）",
  "settings.fields.plansDirectory.label": "Plans ディレクトリ",
  "settings.fields.plansDirectory.tooltip": "プランファイルを保存するディレクトリパス",
  "settings.fields.syntaxHighlightingDisabled.label": "diff のシンタックスハイライトを無効化",
  "settings.fields.syntaxHighlightingDisabled.tooltip": "diff 表示のシンタックスハイライトを無効化",
```

- [ ] **步骤 4：提交**

```bash
git add src/lib/locales/
git commit -m "i18n: add 7 general extension field keys"
```

### Task 20：GeneralEditor 扩展

- [ ] **步骤 1：在 `GeneralEditor.svelte` `<script>` 节追加 state**

在 `let effortLevel = $state(...)` 之后追加：
```typescript
  let autoMemoryEnabled = $state((settings.autoMemoryEnabled as boolean) ?? false);
  let includeGitInstructions = $state((settings.includeGitInstructions as boolean) ?? false);
  let respectGitignore = $state((settings.respectGitignore as boolean) ?? false);
  let cleanupPeriodDays = $state((settings.cleanupPeriodDays as number | undefined) ?? "");
  let claudeMdExcludesText = $state(
    ((settings.claudeMdExcludes as string[]) ?? []).join("\n"),
  );
  let plansDirectory = $state((settings.plansDirectory as string) ?? "");
  let syntaxHighlightingDisabled = $state(
    (settings.syntaxHighlightingDisabled as boolean) ?? false,
  );
```

在 `$effect` 里对应追加：
```typescript
    autoMemoryEnabled = (settings.autoMemoryEnabled as boolean) ?? false;
    includeGitInstructions = (settings.includeGitInstructions as boolean) ?? false;
    respectGitignore = (settings.respectGitignore as boolean) ?? false;
    cleanupPeriodDays = (settings.cleanupPeriodDays as number | undefined) ?? "";
    claudeMdExcludesText = ((settings.claudeMdExcludes as string[]) ?? []).join("\n");
    plansDirectory = (settings.plansDirectory as string) ?? "";
    syntaxHighlightingDisabled =
      (settings.syntaxHighlightingDisabled as boolean) ?? false;
```

追加 dirty derived：
```typescript
  const autoMemoryDirty = $derived(
    autoMemoryEnabled !== ((settings.autoMemoryEnabled as boolean) ?? false),
  );
  const gitInstrDirty = $derived(
    includeGitInstructions !==
      ((settings.includeGitInstructions as boolean) ?? false),
  );
  const gitignoreDirty = $derived(
    respectGitignore !== ((settings.respectGitignore as boolean) ?? false),
  );
  const cleanupDirty = $derived(
    String(cleanupPeriodDays) !==
      String((settings.cleanupPeriodDays as number | undefined) ?? ""),
  );
  const excludesDirty = $derived(
    claudeMdExcludesText !==
      ((settings.claudeMdExcludes as string[]) ?? []).join("\n"),
  );
  const plansDirDirty = $derived(
    plansDirectory !== ((settings.plansDirectory as string) ?? ""),
  );
  const syntaxDisabledDirty = $derived(
    syntaxHighlightingDisabled !==
      ((settings.syntaxHighlightingDisabled as boolean) ?? false),
  );
```

- [ ] **步骤 2：修改 `previewData` 和 `save()`**

在 `previewData` 对象末尾追加：
```typescript
    autoMemoryEnabled,
    includeGitInstructions,
    respectGitignore,
    cleanupPeriodDays: cleanupPeriodDays ? Number(cleanupPeriodDays) : undefined,
    claudeMdExcludes:
      claudeMdExcludesText
        .split("\n")
        .map((s) => s.trim())
        .filter(Boolean).length === 0
        ? undefined
        : claudeMdExcludesText
            .split("\n")
            .map((s) => s.trim())
            .filter(Boolean),
    plansDirectory: plansDirectory || undefined,
    syntaxHighlightingDisabled,
```

`save()` 同样处理（可抽为 helper）。

- [ ] **步骤 3：模板追加控件**

在 `Effort Level` 下拉之后、Save 按钮之前，插入：
```svelte
  <!-- Auto Memory -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={autoMemoryEnabled}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.autoMemoryEnabled.tooltip")}>
      {t("settings.fields.autoMemoryEnabled.label")}
      <DirtyDot dirty={autoMemoryDirty} />
    </span>
  </label>

  <!-- Include Git Instructions -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={includeGitInstructions}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.includeGitInstructions.tooltip")}>
      {t("settings.fields.includeGitInstructions.label")}
      <DirtyDot dirty={gitInstrDirty} />
    </span>
  </label>

  <!-- Respect .gitignore -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={respectGitignore}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.respectGitignore.tooltip")}>
      {t("settings.fields.respectGitignore.label")}
      <DirtyDot dirty={gitignoreDirty} />
    </span>
  </label>

  <!-- Cleanup Period Days -->
  <div class="space-y-1">
    <label for="cleanupPeriodDays" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.cleanupPeriodDays.tooltip")}>
      {t("settings.fields.cleanupPeriodDays.label")}
      <DirtyDot dirty={cleanupDirty} />
    </label>
    <input id="cleanupPeriodDays" type="number" min="1"
           bind:value={cleanupPeriodDays}
           oninput={() => configStore.markDirty()}
           placeholder="30"
           class="input-base" />
  </div>

  <!-- CLAUDE.md Excludes -->
  <div class="space-y-1">
    <label for="claudeMdExcludes" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.claudeMdExcludes.tooltip")}>
      {t("settings.fields.claudeMdExcludes.label")}
      <DirtyDot dirty={excludesDirty} />
    </label>
    <textarea id="claudeMdExcludes" rows="3"
              bind:value={claudeMdExcludesText}
              oninput={() => configStore.markDirty()}
              placeholder={"vendor/\n.venv/"}
              class="input-base font-mono text-xs"></textarea>
  </div>

  <!-- Plans Directory -->
  <div class="space-y-1">
    <label for="plansDirectory" class="block text-sm font-medium"
           style="color: var(--text-secondary)"
           title={t("settings.fields.plansDirectory.tooltip")}>
      {t("settings.fields.plansDirectory.label")}
      <DirtyDot dirty={plansDirDirty} />
    </label>
    <input id="plansDirectory" type="text"
           bind:value={plansDirectory}
           oninput={() => configStore.markDirty()}
           placeholder="~/.claude/plans"
           class="input-base" />
  </div>

  <!-- Syntax Highlighting Disabled -->
  <label class="flex items-center gap-3 cursor-pointer">
    <input type="checkbox" bind:checked={syntaxHighlightingDisabled}
           onchange={() => configStore.markDirty()}
           class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
    <span class="text-sm" style="color: var(--text-secondary)"
          title={t("settings.fields.syntaxHighlightingDisabled.tooltip")}>
      {t("settings.fields.syntaxHighlightingDisabled.label")}
      <DirtyDot dirty={syntaxDisabledDirty} />
    </span>
  </label>
```

- [ ] **步骤 4：重启 dev server + 冒烟**

```bash
pnpm tauri dev
```
在 UI 里逐项改动 7 个字段，观察 JSON Preview 和 Save 后的文件。

- [ ] **步骤 5：提交**

```bash
git add src/lib/components/settings/GeneralEditor.svelte
git commit -m "feat(settings-ui): extend General with 7 content/file fields"
```

### Task 21：fixture + 里程碑收尾

- [ ] **步骤 1：`tests/fixtures/settings-full.json` 根对象追加**

```json
  "autoMemoryEnabled": true,
  "includeGitInstructions": true,
  "respectGitignore": true,
  "cleanupPeriodDays": 30,
  "claudeMdExcludes": ["vendor/", ".venv/"],
  "plansDirectory": "~/.claude/plans",
  "syntaxHighlightingDisabled": false,
```

- [ ] **步骤 2：跑测试和 build**

```bash
cargo test --workspace && pnpm build
```

- [ ] **步骤 3：打 tag**

```bash
git add tests/fixtures/settings-full.json
git commit -m "test: cover general extension fields in fixture"
git push
git tag milestone/M4-general-extension
git push --tags
```

---

## Milestone 5 — MCP tab

**目标**：新增 MCP tab，包含 allowed/denied 列表、jsonServers 开关、全局策略。现有 `deniedMcpServers` UI（如果在别处）迁入新 tab。

**新字段**：`allowedMcpServers`（list of `McpServerRef`）、`enabledMcpjsonServers`（string[]）、`disabledMcpjsonServers`（string[]）、`enableAllProjectMcpServers`（bool）、`allowManagedMcpServersOnly`（bool）

**Files:**
- Modify: `crates/claude-types/src/settings.rs`（加 5 字段；`McpServerRef` 可复用）
- Modify: `src/lib/api/types.ts`
- Create: `src/lib/components/settings/McpPolicyEditor.svelte`
- Create: `src/lib/components/settings/sub/McpServerForm.svelte`
- Modify: `src/lib/components/settings/SettingsEditor.svelte`
- Modify: `src/App.svelte`（`settingsSections` 加 `"mcpPolicy"`）
- Modify: `src/lib/locales/{zh-CN,en-US,ja-JP}.json`
- Modify: `tests/fixtures/settings-full.json`

### Task 22：Rust 字段 + 类型化测试

- [ ] **步骤 1：写 fail 测试**

```rust
    #[test]
    fn parse_mcp_policy_fields_are_typed() {
        let json = r#"{
            "allowedMcpServers": [{"serverName":"context7"}],
            "enabledMcpjsonServers": ["trusted-a"],
            "disabledMcpjsonServers": ["blocked-a"],
            "enableAllProjectMcpServers": true,
            "allowManagedMcpServersOnly": false
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(
            s.allowed_mcp_servers.as_ref().map(|v| v.len()),
            Some(1)
        );
        assert_eq!(
            s.enabled_mcpjson_servers.as_ref().map(Vec::as_slice),
            Some(&["trusted-a".to_string()][..])
        );
        assert_eq!(
            s.disabled_mcpjson_servers.as_ref().map(Vec::as_slice),
            Some(&["blocked-a".to_string()][..])
        );
        assert_eq!(s.enable_all_project_mcp_servers, Some(true));
        assert_eq!(s.allow_managed_mcp_servers_only, Some(false));
        for k in [
            "allowedMcpServers", "enabledMcpjsonServers", "disabledMcpjsonServers",
            "enableAllProjectMcpServers", "allowManagedMcpServersOnly",
        ] {
            assert!(!s.extra.contains_key(k));
        }
    }
```

- [ ] **步骤 2：运行失败 → 加字段 → 通过**

在 Settings struct 追加（在 syntax_highlighting_disabled 之后、extra 之前）：
```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mcp_servers: Option<Vec<McpServerRef>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_mcpjson_servers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_mcpjson_servers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_all_project_mcp_servers: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_mcp_servers_only: Option<bool>,
```

- [ ] **步骤 3：更新 snapshot 测试的 modeled/skipped 列表**

- [ ] **步骤 4：跑测试**

```bash
cargo test -p claude-types
```

- [ ] **步骤 5：TS 同步 — 在 `Settings` 追加**

```typescript
  allowedMcpServers?: McpServerRef[];
  enabledMcpjsonServers?: string[];
  disabledMcpjsonServers?: string[];
  enableAllProjectMcpServers?: boolean;
  allowManagedMcpServersOnly?: boolean;
```

- [ ] **步骤 6：提交**

```bash
cargo test --workspace && pnpm build
git add crates/claude-types/src/settings.rs src/lib/api/types.ts
git commit -m "feat(claude-types,types): add 5 MCP policy fields"
```

### Task 23：i18n 键

- [ ] **步骤 1：zh-CN.json**

```json
  "settings.mcpPolicy": "MCP",
  "settings.mcpPolicy.allowed": "Allowed MCP 服务器",
  "settings.mcpPolicy.denied": "Denied MCP 服务器",
  "settings.mcpPolicy.enabledJson": "Enabled MCP.json 服务器",
  "settings.mcpPolicy.disabledJson": "Disabled MCP.json 服务器",
  "settings.fields.enableAllProjectMcpServers.label": "启用全部项目 MCP 服务器",
  "settings.fields.enableAllProjectMcpServers.tooltip": "开启后，项目 .mcp.json 声明的所有服务器默认启用",
  "settings.fields.allowManagedMcpServersOnly.label": "仅允许托管 MCP 服务器",
  "settings.fields.allowManagedMcpServersOnly.tooltip": "开启后，仅接受由管理员分发的 MCP 服务器",
  "settings.mcpPolicy.serverNamePlaceholder": "服务器名（必填）",
  "settings.mcpPolicy.serverUrlPlaceholder": "URL（可选）",
  "settings.mcpPolicy.jsonServerPlaceholder": "添加服务器名并回车",
  "settings.mcpPolicy.addServer": "+ 添加服务器",
  "settings.mcpPolicy.addAllowed": "+ 加入 Allowed",
  "settings.mcpPolicy.addDenied": "+ 加入 Denied",
```

- [ ] **步骤 2：en-US.json**

```json
  "settings.mcpPolicy": "MCP",
  "settings.mcpPolicy.allowed": "Allowed MCP Servers",
  "settings.mcpPolicy.denied": "Denied MCP Servers",
  "settings.mcpPolicy.enabledJson": "Enabled MCP.json Servers",
  "settings.mcpPolicy.disabledJson": "Disabled MCP.json Servers",
  "settings.fields.enableAllProjectMcpServers.label": "Enable All Project MCP Servers",
  "settings.fields.enableAllProjectMcpServers.tooltip": "When enabled, all servers declared in a project's .mcp.json are enabled by default",
  "settings.fields.allowManagedMcpServersOnly.label": "Allow Managed MCP Servers Only",
  "settings.fields.allowManagedMcpServersOnly.tooltip": "When enabled, only MCP servers distributed by an administrator are accepted",
  "settings.mcpPolicy.serverNamePlaceholder": "server name (required)",
  "settings.mcpPolicy.serverUrlPlaceholder": "URL (optional)",
  "settings.mcpPolicy.jsonServerPlaceholder": "type a server name and press Enter",
  "settings.mcpPolicy.addServer": "+ Add server",
  "settings.mcpPolicy.addAllowed": "+ Add to Allowed",
  "settings.mcpPolicy.addDenied": "+ Add to Denied",
```

- [ ] **步骤 3：ja-JP.json（对应日文翻译）**

```json
  "settings.mcpPolicy": "MCP",
  "settings.mcpPolicy.allowed": "許可された MCP サーバー",
  "settings.mcpPolicy.denied": "拒否された MCP サーバー",
  "settings.mcpPolicy.enabledJson": "有効な MCP.json サーバー",
  "settings.mcpPolicy.disabledJson": "無効な MCP.json サーバー",
  "settings.fields.enableAllProjectMcpServers.label": "全プロジェクト MCP サーバーを有効化",
  "settings.fields.enableAllProjectMcpServers.tooltip": "有効時、プロジェクトの .mcp.json に宣言された全サーバーを既定で有効化",
  "settings.fields.allowManagedMcpServersOnly.label": "マネージド MCP サーバーのみ許可",
  "settings.fields.allowManagedMcpServersOnly.tooltip": "有効時、管理者配布の MCP サーバーのみを受け入れ",
  "settings.mcpPolicy.serverNamePlaceholder": "サーバー名（必須）",
  "settings.mcpPolicy.serverUrlPlaceholder": "URL（任意）",
  "settings.mcpPolicy.jsonServerPlaceholder": "サーバー名を入力して Enter",
  "settings.mcpPolicy.addServer": "+ サーバー追加",
  "settings.mcpPolicy.addAllowed": "+ Allowed に追加",
  "settings.mcpPolicy.addDenied": "+ Denied に追加",
```

- [ ] **步骤 4：提交**

```bash
git add src/lib/locales/
git commit -m "i18n: add MCP tab keys"
```

### Task 24：创建 McpServerForm.svelte（Allowed/Denied 列表的行组件）

- [ ] **步骤 1：创建 `src/lib/components/settings/sub/McpServerForm.svelte`**

```svelte
<script lang="ts">
  import type { McpServerRef } from "$lib/api/types";
  import { t } from "$lib/i18n";

  let {
    servers = $bindable(),
    onChange,
  }: {
    servers: McpServerRef[];
    onChange: () => void;
  } = $props();

  let newName = $state("");
  let newUrl = $state("");

  function add() {
    const name = newName.trim();
    if (!name) return;
    servers = [...servers, { serverName: name, serverUrl: newUrl.trim() || undefined }];
    newName = "";
    newUrl = "";
    onChange();
  }

  function remove(i: number) {
    servers = servers.filter((_, idx) => idx !== i);
    onChange();
  }
</script>

<div class="space-y-2">
  <ul class="space-y-1">
    {#each servers as server, i (i + ":" + (server.serverName ?? ""))}
      <li class="flex items-center gap-2 text-sm">
        <span style="color: var(--text-primary)">{server.serverName ?? "(unnamed)"}</span>
        {#if server.serverUrl}
          <span class="font-mono text-xs" style="color: var(--text-muted)">
            {server.serverUrl}
          </span>
        {/if}
        <button type="button"
                onclick={() => remove(i)}
                class="text-xs px-2 py-0.5 rounded"
                style="color: var(--status-error-text); background-color: var(--status-error-bg)">
          ✕
        </button>
      </li>
    {/each}
  </ul>

  <div class="flex gap-2 items-center">
    <input type="text" bind:value={newName}
           placeholder={t("settings.mcpPolicy.serverNamePlaceholder")}
           class="input-base flex-1" />
    <input type="text" bind:value={newUrl}
           placeholder={t("settings.mcpPolicy.serverUrlPlaceholder")}
           class="input-base flex-1" />
    <button type="button" onclick={add}
            class="btn-secondary text-xs px-3 py-1">
      {t("settings.mcpPolicy.addServer")}
    </button>
  </div>
</div>
```

- [ ] **步骤 2：提交**

```bash
mkdir -p src/lib/components/settings/sub
git add src/lib/components/settings/sub/McpServerForm.svelte
git commit -m "feat(settings-ui): add shared McpServerForm component"
```

### Task 25：创建 McpPolicyEditor.svelte

- [ ] **步骤 1：创建 `src/lib/components/settings/McpPolicyEditor.svelte`**

```svelte
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import DirtyDot from "$lib/components/shared/DirtyDot.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import McpServerForm from "./sub/McpServerForm.svelte";
  import { t } from "$lib/i18n";
  import type { McpServerRef } from "$lib/api/types";

  const settings = $derived(configStore.activeSettings);

  let allowed = $state<McpServerRef[]>(
    (settings.allowedMcpServers as McpServerRef[]) ?? [],
  );
  let denied = $state<McpServerRef[]>(
    (settings.deniedMcpServers as McpServerRef[]) ?? [],
  );
  let enabledJsonText = $state(
    ((settings.enabledMcpjsonServers as string[]) ?? []).join("\n"),
  );
  let disabledJsonText = $state(
    ((settings.disabledMcpjsonServers as string[]) ?? []).join("\n"),
  );
  let enableAllProjectMcpServers = $state(
    (settings.enableAllProjectMcpServers as boolean) ?? false,
  );
  let allowManagedMcpServersOnly = $state(
    (settings.allowManagedMcpServersOnly as boolean) ?? false,
  );

  $effect(() => {
    allowed = (settings.allowedMcpServers as McpServerRef[]) ?? [];
    denied = (settings.deniedMcpServers as McpServerRef[]) ?? [];
    enabledJsonText = ((settings.enabledMcpjsonServers as string[]) ?? []).join("\n");
    disabledJsonText = ((settings.disabledMcpjsonServers as string[]) ?? []).join("\n");
    enableAllProjectMcpServers = (settings.enableAllProjectMcpServers as boolean) ?? false;
    allowManagedMcpServersOnly = (settings.allowManagedMcpServersOnly as boolean) ?? false;
  });

  function parseLines(text: string): string[] | undefined {
    const lines = text.split("\n").map((s) => s.trim()).filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  const previewData = $derived({
    allowedMcpServers: allowed.length === 0 ? undefined : allowed,
    deniedMcpServers: denied.length === 0 ? undefined : denied,
    enabledMcpjsonServers: parseLines(enabledJsonText),
    disabledMcpjsonServers: parseLines(disabledJsonText),
    enableAllProjectMcpServers,
    allowManagedMcpServersOnly,
  });

  function save() {
    configStore.save({
      allowedMcpServers: allowed.length === 0 ? undefined : allowed,
      deniedMcpServers: denied.length === 0 ? undefined : denied,
      enabledMcpjsonServers: parseLines(enabledJsonText),
      disabledMcpjsonServers: parseLines(disabledJsonText),
      enableAllProjectMcpServers,
      allowManagedMcpServersOnly,
    });
  }
</script>

<div class="space-y-6 max-w-2xl">
  <!-- Project MCP flags -->
  <div class="space-y-3">
    <label class="flex items-center gap-3 cursor-pointer">
      <input type="checkbox" bind:checked={enableAllProjectMcpServers}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" style="color: var(--text-secondary)"
            title={t("settings.fields.enableAllProjectMcpServers.tooltip")}>
        {t("settings.fields.enableAllProjectMcpServers.label")}
      </span>
    </label>
    <label class="flex items-center gap-3 cursor-pointer">
      <input type="checkbox" bind:checked={allowManagedMcpServersOnly}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" style="color: var(--text-secondary)"
            title={t("settings.fields.allowManagedMcpServersOnly.tooltip")}>
        {t("settings.fields.allowManagedMcpServersOnly.label")}
      </span>
    </label>
  </div>

  <!-- Allowed -->
  <section>
    <h3 class="text-sm font-semibold mb-2" style="color: var(--text-primary)">
      {t("settings.mcpPolicy.allowed")}
    </h3>
    <McpServerForm bind:servers={allowed} onChange={() => configStore.markDirty()} />
  </section>

  <!-- Denied -->
  <section>
    <h3 class="text-sm font-semibold mb-2" style="color: var(--text-primary)">
      {t("settings.mcpPolicy.denied")}
    </h3>
    <McpServerForm bind:servers={denied} onChange={() => configStore.markDirty()} />
  </section>

  <!-- MCP.json enabled/disabled -->
  <section class="space-y-3">
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.mcpPolicy.enabledJson")}
      </label>
      <textarea bind:value={enabledJsonText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.mcpPolicy.jsonServerPlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.mcpPolicy.disabledJson")}
      </label>
      <textarea bind:value={disabledJsonText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.mcpPolicy.jsonServerPlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
  </section>

  <!-- Save -->
  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button type="button" onclick={save}
            disabled={!configStore.isDirty || configStore.saving}
            class="btn-primary text-sm px-4 py-2">
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button type="button" onclick={() => configStore.revert()}
            disabled={!configStore.isDirty}
            class="btn-secondary text-sm px-4 py-2">
      {t("common.revert")}
    </button>
  </div>

  <JsonPreview data={previewData} title="MCP JSON" />
</div>
```

- [ ] **步骤 2：在 SettingsEditor.svelte 注册**

```typescript
  import McpPolicyEditor from "./McpPolicyEditor.svelte";
```

模板里追加：
```svelte
  {:else if activeSection === "mcpPolicy"}
    <McpPolicyEditor />
```

- [ ] **步骤 3：在 App.svelte 的 `settingsSections` 追加**

```typescript
    { id: "mcpPolicy", labelKey: "settings.mcpPolicy" },
```

- [ ] **步骤 4：重启 dev server + 冒烟**

```bash
pnpm tauri dev
```
UI 里添加 allowed 和 denied 的 server 各一项、填若干 JSON server 名，保存，查文件。

- [ ] **步骤 5：提交**

```bash
git add src/lib/components/settings/McpPolicyEditor.svelte src/lib/components/settings/SettingsEditor.svelte src/App.svelte
git commit -m "feat(settings-ui): add MCP tab with allowed/denied + json flags"
```

### Task 26：M5 收尾

- [ ] **步骤 1：fixture 更新**

在 `tests/fixtures/settings-full.json`（找到已有 `deniedMcpServers` 位置附近）追加：
```json
  "allowedMcpServers": [
    { "serverName": "context7", "serverUrl": "https://mcp.context7.com/mcp" }
  ],
  "enabledMcpjsonServers": ["trusted-a"],
  "disabledMcpjsonServers": ["blocked-a"],
  "enableAllProjectMcpServers": true,
  "allowManagedMcpServersOnly": false,
```

- [ ] **步骤 2：跑测试、build、打 tag**

```bash
cargo test --workspace && pnpm build
git add tests/fixtures/settings-full.json
git commit -m "test: cover MCP policy fields in fixture"
git push
git tag milestone/M5-mcp-tab
git push --tags
```

---

## Milestone 6 — Plugins & Marketplace tab

**目标**：新增 Plugins & Marketplace tab。`pluginConfigs` 用 `serde_json::Value` 存，UI 走内嵌 JSON 编辑器。

**新字段**：`strictKnownMarketplaces`、`blockedMarketplaces`、`skippedMarketplaces`、`skippedPlugins`、`pluginConfigs`、`pluginTrustMessage`、`skillOverrides`

**Files:**
- Modify: `crates/claude-types/src/settings.rs`
- Modify: `src/lib/api/types.ts`
- Create: `src/lib/components/settings/PluginsMarketplaceEditor.svelte`
- Modify: `src/lib/components/settings/SettingsEditor.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/locales/{zh-CN,en-US,ja-JP}.json`
- Modify: `tests/fixtures/settings-full.json`

### Task 27：Rust 字段 + 类型化测试

- [ ] **步骤 1：fail 测试**

```rust
    #[test]
    fn parse_plugins_marketplace_fields_are_typed() {
        let json = r#"{
            "strictKnownMarketplaces": [{"source":{"source":"github","repo":"a/b"}}],
            "blockedMarketplaces": [{"source":{"source":"github","repo":"c/d"}}],
            "skippedMarketplaces": ["skipMe"],
            "skippedPlugins": ["badPlugin"],
            "pluginTrustMessage": "All approved",
            "skillOverrides": {"k":"v"},
            "pluginConfigs": {"p@m":{"options":{"apiKey":"x"}}}
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(s.strict_known_marketplaces.as_ref().map(Vec::len), Some(1));
        assert_eq!(s.blocked_marketplaces.as_ref().map(Vec::len), Some(1));
        assert_eq!(
            s.skipped_marketplaces.as_ref().map(Vec::as_slice),
            Some(&["skipMe".to_string()][..])
        );
        assert_eq!(
            s.skipped_plugins.as_ref().map(Vec::as_slice),
            Some(&["badPlugin".to_string()][..])
        );
        assert_eq!(s.plugin_trust_message.as_deref(), Some("All approved"));
        assert!(s.skill_overrides.is_some());
        assert!(s.plugin_configs.is_some());
        for k in [
            "strictKnownMarketplaces","blockedMarketplaces","skippedMarketplaces",
            "skippedPlugins","pluginTrustMessage","skillOverrides","pluginConfigs",
        ] {
            assert!(!s.extra.contains_key(k));
        }
    }
```

- [ ] **步骤 2：加字段到 `Settings`**

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_known_marketplaces: Option<Vec<MarketplaceSource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_marketplaces: Option<Vec<MarketplaceSource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_marketplaces: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_plugins: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_trust_message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_overrides: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_configs: Option<serde_json::Value>,
```

- [ ] **步骤 3：更新 snapshot 测试列表，跑测试**

```bash
cargo test -p claude-types
```

- [ ] **步骤 4：TS 同步**

```typescript
  strictKnownMarketplaces?: MarketplaceSource[];
  blockedMarketplaces?: MarketplaceSource[];
  skippedMarketplaces?: string[];
  skippedPlugins?: string[];
  pluginTrustMessage?: string;
  skillOverrides?: Record<string, unknown>;
  pluginConfigs?: Record<string, unknown>;
```

- [ ] **步骤 5：提交**

```bash
cargo test --workspace && pnpm build
git add crates/claude-types/src/settings.rs src/lib/api/types.ts
git commit -m "feat(claude-types,types): add 7 plugins/marketplace fields"
```

### Task 28：i18n 键（加到三个 locale，下面给中英双语模板，日文复用结构翻译）

- [ ] **步骤 1：zh-CN.json**

```json
  "settings.pluginsMarketplace": "插件与市场",
  "settings.pluginsMarketplace.strictKnown": "Strict Known Marketplaces",
  "settings.pluginsMarketplace.blocked": "Blocked Marketplaces",
  "settings.pluginsMarketplace.skippedMarkets": "Skipped Marketplaces",
  "settings.pluginsMarketplace.skippedPlugins": "Skipped Plugins",
  "settings.pluginsMarketplace.pluginConfigs": "Plugin Configs（JSON）",
  "settings.pluginsMarketplace.skillOverrides": "Skill Overrides（JSON）",
  "settings.fields.pluginTrustMessage.label": "插件信任提示",
  "settings.fields.pluginTrustMessage.tooltip": "加载插件时向用户展示的信任说明（多行文本）",
  "settings.pluginsMarketplace.mpSourcePlaceholder": "source 类型（如 github）",
  "settings.pluginsMarketplace.mpRepoPlaceholder": "repo（如 org/project）",
  "settings.pluginsMarketplace.addMarketplace": "+ 添加",
  "settings.pluginsMarketplace.addLinePlaceholder": "添加条目并回车",
  "settings.pluginsMarketplace.invalidJson": "JSON 格式错误",
```

- [ ] **步骤 2：en-US.json**（对应英文）

```json
  "settings.pluginsMarketplace": "Plugins & Marketplace",
  "settings.pluginsMarketplace.strictKnown": "Strict Known Marketplaces",
  "settings.pluginsMarketplace.blocked": "Blocked Marketplaces",
  "settings.pluginsMarketplace.skippedMarkets": "Skipped Marketplaces",
  "settings.pluginsMarketplace.skippedPlugins": "Skipped Plugins",
  "settings.pluginsMarketplace.pluginConfigs": "Plugin Configs (JSON)",
  "settings.pluginsMarketplace.skillOverrides": "Skill Overrides (JSON)",
  "settings.fields.pluginTrustMessage.label": "Plugin Trust Message",
  "settings.fields.pluginTrustMessage.tooltip": "Trust message shown to users when loading plugins (multi-line)",
  "settings.pluginsMarketplace.mpSourcePlaceholder": "source type (e.g. github)",
  "settings.pluginsMarketplace.mpRepoPlaceholder": "repo (e.g. org/project)",
  "settings.pluginsMarketplace.addMarketplace": "+ Add",
  "settings.pluginsMarketplace.addLinePlaceholder": "type an entry and press Enter",
  "settings.pluginsMarketplace.invalidJson": "Invalid JSON",
```

- [ ] **步骤 3：ja-JP.json**

```json
  "settings.pluginsMarketplace": "プラグインとマーケット",
  "settings.pluginsMarketplace.strictKnown": "Strict Known Marketplaces",
  "settings.pluginsMarketplace.blocked": "Blocked Marketplaces",
  "settings.pluginsMarketplace.skippedMarkets": "Skipped Marketplaces",
  "settings.pluginsMarketplace.skippedPlugins": "Skipped Plugins",
  "settings.pluginsMarketplace.pluginConfigs": "Plugin Configs（JSON）",
  "settings.pluginsMarketplace.skillOverrides": "Skill Overrides（JSON）",
  "settings.fields.pluginTrustMessage.label": "プラグイン信頼メッセージ",
  "settings.fields.pluginTrustMessage.tooltip": "プラグイン読込時に表示する信頼メッセージ（複数行）",
  "settings.pluginsMarketplace.mpSourcePlaceholder": "source 種別（例: github）",
  "settings.pluginsMarketplace.mpRepoPlaceholder": "repo（例: org/project）",
  "settings.pluginsMarketplace.addMarketplace": "+ 追加",
  "settings.pluginsMarketplace.addLinePlaceholder": "エントリを入力して Enter",
  "settings.pluginsMarketplace.invalidJson": "JSON 形式が不正",
```

- [ ] **步骤 4：提交**

```bash
git add src/lib/locales/
git commit -m "i18n: add plugins/marketplace tab keys"
```

### Task 29：PluginsMarketplaceEditor.svelte

- [ ] **步骤 1：创建组件**

文件 `src/lib/components/settings/PluginsMarketplaceEditor.svelte`：

```svelte
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import { t } from "$lib/i18n";
  import type { MarketplaceSource } from "$lib/api/types";

  const settings = $derived(configStore.activeSettings);

  let strictKnown = $state<MarketplaceSource[]>(
    (settings.strictKnownMarketplaces as MarketplaceSource[]) ?? [],
  );
  let blocked = $state<MarketplaceSource[]>(
    (settings.blockedMarketplaces as MarketplaceSource[]) ?? [],
  );
  let skippedMarketsText = $state(
    ((settings.skippedMarketplaces as string[]) ?? []).join("\n"),
  );
  let skippedPluginsText = $state(
    ((settings.skippedPlugins as string[]) ?? []).join("\n"),
  );
  let trustMessage = $state((settings.pluginTrustMessage as string) ?? "");
  let pluginConfigsText = $state(
    JSON.stringify((settings.pluginConfigs as object) ?? {}, null, 2),
  );
  let skillOverridesText = $state(
    JSON.stringify((settings.skillOverrides as object) ?? {}, null, 2),
  );
  let pluginConfigsError = $state("");
  let skillOverridesError = $state("");

  let newSrc = $state("");
  let newRepo = $state("");
  let targetList = $state<"strict" | "blocked">("strict");

  $effect(() => {
    strictKnown = (settings.strictKnownMarketplaces as MarketplaceSource[]) ?? [];
    blocked = (settings.blockedMarketplaces as MarketplaceSource[]) ?? [];
    skippedMarketsText = ((settings.skippedMarketplaces as string[]) ?? []).join("\n");
    skippedPluginsText = ((settings.skippedPlugins as string[]) ?? []).join("\n");
    trustMessage = (settings.pluginTrustMessage as string) ?? "";
    pluginConfigsText = JSON.stringify((settings.pluginConfigs as object) ?? {}, null, 2);
    skillOverridesText = JSON.stringify(
      (settings.skillOverrides as object) ?? {},
      null,
      2,
    );
  });

  function addMp() {
    if (!newSrc.trim() || !newRepo.trim()) return;
    const entry: MarketplaceSource = {
      source: { source: newSrc.trim(), url: undefined },
      repo: newRepo.trim(),
    } as unknown as MarketplaceSource;
    if (targetList === "strict") strictKnown = [...strictKnown, entry];
    else blocked = [...blocked, entry];
    newSrc = "";
    newRepo = "";
    configStore.markDirty();
  }

  function removeMp(target: "strict" | "blocked", i: number) {
    if (target === "strict") strictKnown = strictKnown.filter((_, idx) => idx !== i);
    else blocked = blocked.filter((_, idx) => idx !== i);
    configStore.markDirty();
  }

  function parseLines(text: string): string[] | undefined {
    const lines = text.split("\n").map((s) => s.trim()).filter(Boolean);
    return lines.length === 0 ? undefined : lines;
  }

  function parseJson(text: string, setError: (e: string) => void): object | undefined {
    const trimmed = text.trim();
    if (!trimmed || trimmed === "{}") {
      setError("");
      return undefined;
    }
    try {
      const parsed = JSON.parse(trimmed);
      setError("");
      return parsed;
    } catch (e) {
      setError(t("settings.pluginsMarketplace.invalidJson") + ": " + String(e));
      return undefined;
    }
  }

  function save() {
    const pc = parseJson(pluginConfigsText, (e) => (pluginConfigsError = e));
    const so = parseJson(skillOverridesText, (e) => (skillOverridesError = e));
    if (pluginConfigsError || skillOverridesError) return;

    configStore.save({
      strictKnownMarketplaces: strictKnown.length === 0 ? undefined : strictKnown,
      blockedMarketplaces: blocked.length === 0 ? undefined : blocked,
      skippedMarketplaces: parseLines(skippedMarketsText),
      skippedPlugins: parseLines(skippedPluginsText),
      pluginTrustMessage: trustMessage || undefined,
      pluginConfigs: pc as Record<string, unknown> | undefined,
      skillOverrides: so as Record<string, unknown> | undefined,
    });
  }
</script>

<div class="space-y-6 max-w-2xl">
  <!-- Marketplace list editor (shared for strict/blocked) -->
  <section>
    <div class="flex items-center gap-2 mb-2">
      <select bind:value={targetList} class="input-base w-40 text-sm">
        <option value="strict">{t("settings.pluginsMarketplace.strictKnown")}</option>
        <option value="blocked">{t("settings.pluginsMarketplace.blocked")}</option>
      </select>
      <input type="text" bind:value={newSrc}
             placeholder={t("settings.pluginsMarketplace.mpSourcePlaceholder")}
             class="input-base flex-1" />
      <input type="text" bind:value={newRepo}
             placeholder={t("settings.pluginsMarketplace.mpRepoPlaceholder")}
             class="input-base flex-1" />
      <button type="button" onclick={addMp}
              class="btn-secondary text-xs px-3 py-1">
        {t("settings.pluginsMarketplace.addMarketplace")}
      </button>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <h4 class="text-xs font-semibold mb-1" style="color: var(--text-muted)">
          {t("settings.pluginsMarketplace.strictKnown")}
        </h4>
        <ul class="space-y-1 text-sm">
          {#each strictKnown as mp, i (i + ":strict")}
            <li class="flex items-center gap-2">
              <span style="color: var(--text-primary)">
                {mp.source?.source ?? "?"} / {(mp as unknown as { repo?: string }).repo ?? "?"}
              </span>
              <button type="button" onclick={() => removeMp("strict", i)}
                      class="text-xs px-2 py-0.5 rounded"
                      style="color: var(--status-error-text); background-color: var(--status-error-bg)">
                ✕
              </button>
            </li>
          {/each}
        </ul>
      </div>
      <div>
        <h4 class="text-xs font-semibold mb-1" style="color: var(--text-muted)">
          {t("settings.pluginsMarketplace.blocked")}
        </h4>
        <ul class="space-y-1 text-sm">
          {#each blocked as mp, i (i + ":blocked")}
            <li class="flex items-center gap-2">
              <span style="color: var(--text-primary)">
                {mp.source?.source ?? "?"} / {(mp as unknown as { repo?: string }).repo ?? "?"}
              </span>
              <button type="button" onclick={() => removeMp("blocked", i)}
                      class="text-xs px-2 py-0.5 rounded"
                      style="color: var(--status-error-text); background-color: var(--status-error-bg)">
                ✕
              </button>
            </li>
          {/each}
        </ul>
      </div>
    </div>
  </section>

  <!-- Skipped marketplaces / plugins -->
  <section class="grid grid-cols-2 gap-4">
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.pluginsMarketplace.skippedMarkets")}
      </label>
      <textarea bind:value={skippedMarketsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.pluginsMarketplace.addLinePlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
    <div>
      <label class="text-sm font-semibold" style="color: var(--text-primary)">
        {t("settings.pluginsMarketplace.skippedPlugins")}
      </label>
      <textarea bind:value={skippedPluginsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder={t("settings.pluginsMarketplace.addLinePlaceholder")}
                class="input-base font-mono text-xs mt-1"></textarea>
    </div>
  </section>

  <!-- Trust message -->
  <section>
    <label class="text-sm font-semibold" style="color: var(--text-primary)"
           title={t("settings.fields.pluginTrustMessage.tooltip")}>
      {t("settings.fields.pluginTrustMessage.label")}
    </label>
    <textarea bind:value={trustMessage} rows="3"
              oninput={() => configStore.markDirty()}
              class="input-base text-sm mt-1"></textarea>
  </section>

  <!-- Plugin configs JSON -->
  <section>
    <label class="text-sm font-semibold" style="color: var(--text-primary)">
      {t("settings.pluginsMarketplace.pluginConfigs")}
    </label>
    <textarea bind:value={pluginConfigsText} rows="6"
              oninput={() => configStore.markDirty()}
              class="input-base font-mono text-xs mt-1"></textarea>
    {#if pluginConfigsError}
      <p class="text-xs" style="color: var(--status-error-text)">{pluginConfigsError}</p>
    {/if}
  </section>

  <!-- Skill overrides JSON -->
  <section>
    <label class="text-sm font-semibold" style="color: var(--text-primary)">
      {t("settings.pluginsMarketplace.skillOverrides")}
    </label>
    <textarea bind:value={skillOverridesText} rows="4"
              oninput={() => configStore.markDirty()}
              class="input-base font-mono text-xs mt-1"></textarea>
    {#if skillOverridesError}
      <p class="text-xs" style="color: var(--status-error-text)">{skillOverridesError}</p>
    {/if}
  </section>

  <div class="flex gap-2 pt-4 border-t" style="border-color: var(--border-color)">
    <button type="button" onclick={save}
            disabled={!configStore.isDirty || configStore.saving}
            class="btn-primary text-sm px-4 py-2">
      {configStore.saving ? t("common.saving") : t("common.save")}
    </button>
    <button type="button" onclick={() => configStore.revert()}
            disabled={!configStore.isDirty}
            class="btn-secondary text-sm px-4 py-2">
      {t("common.revert")}
    </button>
  </div>

  <JsonPreview data={{
    strictKnownMarketplaces: strictKnown.length === 0 ? undefined : strictKnown,
    blockedMarketplaces: blocked.length === 0 ? undefined : blocked,
    pluginTrustMessage: trustMessage || undefined,
  }} title="Plugins & Marketplace JSON" />
</div>
```

- [ ] **步骤 2：在 SettingsEditor.svelte 注册**

```typescript
  import PluginsMarketplaceEditor from "./PluginsMarketplaceEditor.svelte";
```

模板追加：
```svelte
  {:else if activeSection === "pluginsMarketplace"}
    <PluginsMarketplaceEditor />
```

- [ ] **步骤 3：App.svelte `settingsSections` 追加**

```typescript
    { id: "pluginsMarketplace", labelKey: "settings.pluginsMarketplace" },
```

- [ ] **步骤 4：重启 dev server + 冒烟**

```bash
pnpm tauri dev
```
UI 里分别加 strict/blocked marketplaces、填 skipped、pluginConfigs JSON、保存。试输入非法 JSON 确认错误提示正确。

- [ ] **步骤 5：提交 + fixture + 打 tag**

```bash
git add src/lib/components/settings/PluginsMarketplaceEditor.svelte src/lib/components/settings/SettingsEditor.svelte src/App.svelte
git commit -m "feat(settings-ui): add Plugins & Marketplace tab"
```

在 `tests/fixtures/settings-full.json` 追加：
```json
  "strictKnownMarketplaces": [
    { "source": { "source": "github", "repo": "anthropics/claude-plugins" } }
  ],
  "blockedMarketplaces": [],
  "skippedMarketplaces": ["some-old-market"],
  "skippedPlugins": ["deprecated-plugin"],
  "pluginTrustMessage": "Only install plugins from vetted marketplaces.",
  "pluginConfigs": {
    "context7@official": { "options": { "apiKey": "xxx" } }
  },
  "skillOverrides": {
    "my-skill": { "disabled": true }
  },
```

```bash
cargo test --workspace && pnpm build
git add tests/fixtures/settings-full.json
git commit -m "test: cover plugins/marketplace fields in fixture"
git push
git tag milestone/M6-plugins-marketplace
git push --tags
```

---

## Milestone 7 — Hooks Policy 扩展

**目标**：在现有 HooksEditor 底部追加 Policy 小节，容纳 6 个 hooks/permissions 策略字段。不新开 tab。

**新字段**：`disableAllHooks`、`allowedHttpHookUrls`、`httpHookAllowedEnvVars`、`allowManagedHooksOnly`、`allowManagedPermissionRulesOnly`、`disableSkillShellExecution`

**Files:**
- Modify: `crates/claude-types/src/settings.rs`
- Modify: `src/lib/api/types.ts`
- Modify: `src/lib/components/settings/HooksEditor.svelte`
- Modify: `src/lib/locales/{zh-CN,en-US,ja-JP}.json`
- Modify: `tests/fixtures/settings-full.json`

### Task 30：Rust 字段 + 类型化测试

- [ ] **步骤 1：fail 测试**

```rust
    #[test]
    fn parse_hooks_policy_fields_are_typed() {
        let json = r#"{
            "disableAllHooks": false,
            "allowedHttpHookUrls": ["https://hooks.corp"],
            "httpHookAllowedEnvVars": ["GITHUB_TOKEN"],
            "allowManagedHooksOnly": true,
            "allowManagedPermissionRulesOnly": false,
            "disableSkillShellExecution": true
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(s.disable_all_hooks, Some(false));
        assert_eq!(
            s.allowed_http_hook_urls.as_ref().map(Vec::as_slice),
            Some(&["https://hooks.corp".to_string()][..])
        );
        assert_eq!(
            s.http_hook_allowed_env_vars.as_ref().map(Vec::as_slice),
            Some(&["GITHUB_TOKEN".to_string()][..])
        );
        assert_eq!(s.allow_managed_hooks_only, Some(true));
        assert_eq!(s.allow_managed_permission_rules_only, Some(false));
        assert_eq!(s.disable_skill_shell_execution, Some(true));
        for k in [
            "disableAllHooks","allowedHttpHookUrls","httpHookAllowedEnvVars",
            "allowManagedHooksOnly","allowManagedPermissionRulesOnly",
            "disableSkillShellExecution",
        ] {
            assert!(!s.extra.contains_key(k));
        }
    }
```

- [ ] **步骤 2：加字段**

```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_all_hooks: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_http_hook_urls: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_hook_allowed_env_vars: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_hooks_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_managed_permission_rules_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_skill_shell_execution: Option<bool>,
```

- [ ] **步骤 3：更新 snapshot 测试列表，跑测试**

```bash
cargo test -p claude-types
```

- [ ] **步骤 4：TS 同步 Settings interface 追加**

```typescript
  disableAllHooks?: boolean;
  allowedHttpHookUrls?: string[];
  httpHookAllowedEnvVars?: string[];
  allowManagedHooksOnly?: boolean;
  allowManagedPermissionRulesOnly?: boolean;
  disableSkillShellExecution?: boolean;
```

- [ ] **步骤 5：提交**

```bash
cargo test --workspace && pnpm build
git add crates/claude-types/src/settings.rs src/lib/api/types.ts
git commit -m "feat(claude-types,types): add 6 hooks policy fields"
```

### Task 31：i18n + HooksEditor 扩展

- [ ] **步骤 1：i18n 键（三语）**

`zh-CN.json`:
```json
  "settings.hooks.policy": "策略",
  "settings.fields.disableAllHooks.label": "禁用全部 Hooks",
  "settings.fields.disableAllHooks.tooltip": "开启后所有 hook 事件不再执行",
  "settings.fields.allowedHttpHookUrls.label": "允许的 HTTP Hook URL",
  "settings.fields.allowedHttpHookUrls.tooltip": "允许发起 HTTP hook 的 URL 列表（每行一条）",
  "settings.fields.httpHookAllowedEnvVars.label": "HTTP Hook 允许环境变量",
  "settings.fields.httpHookAllowedEnvVars.tooltip": "HTTP hook 可透传的环境变量名（每行一条）",
  "settings.fields.allowManagedHooksOnly.label": "仅允许托管 Hooks",
  "settings.fields.allowManagedHooksOnly.tooltip": "开启后只执行管理员分发的 hooks",
  "settings.fields.allowManagedPermissionRulesOnly.label": "仅允许托管权限规则",
  "settings.fields.allowManagedPermissionRulesOnly.tooltip": "开启后只接受管理员分发的 permissions 规则",
  "settings.fields.disableSkillShellExecution.label": "禁用 Skill Shell 执行",
  "settings.fields.disableSkillShellExecution.tooltip": "开启后 skill 中的 shell 脚本不会被执行",
```

`en-US.json`:
```json
  "settings.hooks.policy": "Policy",
  "settings.fields.disableAllHooks.label": "Disable All Hooks",
  "settings.fields.disableAllHooks.tooltip": "When enabled, all hook events are skipped",
  "settings.fields.allowedHttpHookUrls.label": "Allowed HTTP Hook URLs",
  "settings.fields.allowedHttpHookUrls.tooltip": "URLs allowed to fire HTTP hooks (one per line)",
  "settings.fields.httpHookAllowedEnvVars.label": "HTTP Hook Allowed Env Vars",
  "settings.fields.httpHookAllowedEnvVars.tooltip": "Env vars that HTTP hooks are allowed to pass through (one per line)",
  "settings.fields.allowManagedHooksOnly.label": "Allow Managed Hooks Only",
  "settings.fields.allowManagedHooksOnly.tooltip": "When enabled, only hooks distributed by an administrator run",
  "settings.fields.allowManagedPermissionRulesOnly.label": "Allow Managed Permission Rules Only",
  "settings.fields.allowManagedPermissionRulesOnly.tooltip": "When enabled, only permission rules distributed by an administrator are accepted",
  "settings.fields.disableSkillShellExecution.label": "Disable Skill Shell Execution",
  "settings.fields.disableSkillShellExecution.tooltip": "When enabled, shell scripts embedded in skills are not executed",
```

`ja-JP.json`:
```json
  "settings.hooks.policy": "ポリシー",
  "settings.fields.disableAllHooks.label": "全 Hooks を無効化",
  "settings.fields.disableAllHooks.tooltip": "有効時、全 hook イベントがスキップ",
  "settings.fields.allowedHttpHookUrls.label": "許可する HTTP Hook URL",
  "settings.fields.allowedHttpHookUrls.tooltip": "HTTP hook を発火可能な URL 一覧（1 行 1 件）",
  "settings.fields.httpHookAllowedEnvVars.label": "HTTP Hook 許可環境変数",
  "settings.fields.httpHookAllowedEnvVars.tooltip": "HTTP hook がパススルー可能な環境変数名（1 行 1 件）",
  "settings.fields.allowManagedHooksOnly.label": "マネージド Hooks のみ許可",
  "settings.fields.allowManagedHooksOnly.tooltip": "有効時、管理者配布の hooks のみ実行",
  "settings.fields.allowManagedPermissionRulesOnly.label": "マネージド権限ルールのみ許可",
  "settings.fields.allowManagedPermissionRulesOnly.tooltip": "有効時、管理者配布の permissions ルールのみ受入",
  "settings.fields.disableSkillShellExecution.label": "Skill Shell 実行を無効化",
  "settings.fields.disableSkillShellExecution.tooltip": "有効時、skill 内の shell スクリプトは実行されない",
```

- [ ] **步骤 2：在 `HooksEditor.svelte` 底部（Save/Revert 按钮之前）追加 Policy 小节**

读已有 HooksEditor 结构，在 `<script>` 节的现有 state 之后追加：
```typescript
  let disableAllHooks = $state((settings.disableAllHooks as boolean) ?? false);
  let allowedHttpHookUrlsText = $state(
    ((settings.allowedHttpHookUrls as string[]) ?? []).join("\n"),
  );
  let httpHookAllowedEnvVarsText = $state(
    ((settings.httpHookAllowedEnvVars as string[]) ?? []).join("\n"),
  );
  let allowManagedHooksOnly = $state((settings.allowManagedHooksOnly as boolean) ?? false);
  let allowManagedPermissionRulesOnly = $state(
    (settings.allowManagedPermissionRulesOnly as boolean) ?? false,
  );
  let disableSkillShellExecution = $state(
    (settings.disableSkillShellExecution as boolean) ?? false,
  );
```

在 `$effect` 里同步赋值，在 save 函数对应把这些字段写入 patch（按 parseLines 规则处理两个 text 字段，其余直接给 boolean）。

在模板底部 Save/Revert 按钮之前插入 Policy 小节：
```svelte
  <section class="border-t pt-4 mt-4" style="border-color: var(--border-color)">
    <h3 class="text-sm font-semibold mb-3" style="color: var(--text-primary)">
      {t("settings.hooks.policy")}
    </h3>

    <label class="flex items-center gap-3 cursor-pointer mb-2">
      <input type="checkbox" bind:checked={disableAllHooks}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.disableAllHooks.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.disableAllHooks.label")}
      </span>
    </label>

    <label class="flex items-center gap-3 cursor-pointer mb-2">
      <input type="checkbox" bind:checked={allowManagedHooksOnly}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.allowManagedHooksOnly.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.allowManagedHooksOnly.label")}
      </span>
    </label>

    <label class="flex items-center gap-3 cursor-pointer mb-2">
      <input type="checkbox" bind:checked={allowManagedPermissionRulesOnly}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.allowManagedPermissionRulesOnly.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.allowManagedPermissionRulesOnly.label")}
      </span>
    </label>

    <label class="flex items-center gap-3 cursor-pointer mb-3">
      <input type="checkbox" bind:checked={disableSkillShellExecution}
             onchange={() => configStore.markDirty()}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" title={t("settings.fields.disableSkillShellExecution.tooltip")}
            style="color: var(--text-secondary)">
        {t("settings.fields.disableSkillShellExecution.label")}
      </span>
    </label>

    <div class="space-y-1 mb-3">
      <label class="block text-sm font-medium" style="color: var(--text-secondary)"
             title={t("settings.fields.allowedHttpHookUrls.tooltip")}>
        {t("settings.fields.allowedHttpHookUrls.label")}
      </label>
      <textarea bind:value={allowedHttpHookUrlsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder="https://hooks.internal.corp"
                class="input-base font-mono text-xs"></textarea>
    </div>

    <div class="space-y-1">
      <label class="block text-sm font-medium" style="color: var(--text-secondary)"
             title={t("settings.fields.httpHookAllowedEnvVars.tooltip")}>
        {t("settings.fields.httpHookAllowedEnvVars.label")}
      </label>
      <textarea bind:value={httpHookAllowedEnvVarsText} rows="3"
                oninput={() => configStore.markDirty()}
                placeholder="GITHUB_TOKEN"
                class="input-base font-mono text-xs"></textarea>
    </div>
  </section>
```

在 `save()` patch 里追加：
```typescript
    disableAllHooks,
    allowedHttpHookUrls:
      allowedHttpHookUrlsText.split("\n").map((s) => s.trim()).filter(Boolean).length === 0
        ? undefined
        : allowedHttpHookUrlsText.split("\n").map((s) => s.trim()).filter(Boolean),
    httpHookAllowedEnvVars:
      httpHookAllowedEnvVarsText.split("\n").map((s) => s.trim()).filter(Boolean).length === 0
        ? undefined
        : httpHookAllowedEnvVarsText.split("\n").map((s) => s.trim()).filter(Boolean),
    allowManagedHooksOnly,
    allowManagedPermissionRulesOnly,
    disableSkillShellExecution,
```

- [ ] **步骤 3：重启 dev server + 冒烟**

```bash
pnpm tauri dev
```
打开 Hooks tab，滚到底看 Policy 小节，改动 6 个字段保存。

- [ ] **步骤 4：提交 + fixture + 打 tag**

```bash
git add src/lib/locales/ src/lib/components/settings/HooksEditor.svelte
git commit -m "feat(settings-ui): extend Hooks with Policy section"
```

`tests/fixtures/settings-full.json` 追加：
```json
  "disableAllHooks": false,
  "allowedHttpHookUrls": ["https://hooks.internal"],
  "httpHookAllowedEnvVars": ["GITHUB_TOKEN"],
  "allowManagedHooksOnly": false,
  "allowManagedPermissionRulesOnly": false,
  "disableSkillShellExecution": false,
```

```bash
cargo test --workspace && pnpm build
git add tests/fixtures/settings-full.json
git commit -m "test: cover hooks policy fields in fixture"
git push
git tag milestone/M7-hooks-policy
git push --tags
```

---

## Milestone 8 — Advanced JSON tab

**目标**：建模所有剩余长尾字段（Rust 显式声明，sub-object 用 `serde_json::Value`），实现 Advanced tab（左侧 schema key picker + 右侧按类型切换编辑器）。

**新字段**（共 29 个）：
- 认证/运维：`apiKeyHelper`、`awsCredentialExport`、`awsAuthRefresh`、`gcpAuthRefresh`、`forceLoginMethod`、`forceLoginOrgUUID`、`otelHeadersHelper`
- 体验：`prefersReducedMotion`、`companyAnnouncements`、`feedbackSurveyRate`、`terminalTitleFromRename`、`awaySummaryEnabled`、`showThinkingSummaries`
- 其他：`advisorModel`、`agent`、`autoDreamEnabled`、`autoMemoryDirectory`、`skillListingBudgetFraction`、`skillListingMaxDescChars`、`skipWebFetchPreflight`、`forceRemoteSettingsRefresh`
- 子对象（Value）：`attribution`、`autoMode`、`fileSuggestion`、`worktree`、`subagentStatusLine`、`spinnerVerbs`、`spinnerTipsOverride`、`remote`

**Files:**
- Modify: `crates/claude-types/src/settings.rs`
- Modify: `src/lib/api/types.ts`
- Create: `src/lib/components/settings/AdvancedJsonEditor.svelte`
- Create: `src/lib/components/settings/sub/SchemaKeyPicker.svelte`
- Create: `src/lib/components/settings/sub/JsonValueEditor.svelte`
- Modify: `src/lib/components/settings/SettingsEditor.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/locales/{zh-CN,en-US,ja-JP}.json`
- Modify: `tests/fixtures/settings-full.json`

### Task 32：Rust 字段（Scalar 长尾 + Sub-object Value）+ 测试

- [ ] **步骤 1：fail 测试（scalar 字段）**

```rust
    #[test]
    fn parse_advanced_scalar_fields_are_typed() {
        let json = r#"{
            "apiKeyHelper": "/usr/local/bin/get-key",
            "awsCredentialExport": "arn:aws:iam::123:role/X",
            "awsAuthRefresh": "aws sso login",
            "gcpAuthRefresh": "gcloud auth login",
            "forceLoginMethod": "claudeai",
            "forceLoginOrgUUID": "org-xyz",
            "otelHeadersHelper": "/usr/local/bin/otel-headers",
            "prefersReducedMotion": true,
            "companyAnnouncements": ["Welcome"],
            "feedbackSurveyRate": 0.1,
            "terminalTitleFromRename": true,
            "awaySummaryEnabled": false,
            "showThinkingSummaries": true,
            "advisorModel": "claude-sonnet-4-6",
            "agent": "code-simplifier",
            "autoDreamEnabled": false,
            "autoMemoryDirectory": "~/.claude/memory",
            "skillListingBudgetFraction": 0.3,
            "skillListingMaxDescChars": 200,
            "skipWebFetchPreflight": true,
            "forceRemoteSettingsRefresh": false
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert_eq!(s.api_key_helper.as_deref(), Some("/usr/local/bin/get-key"));
        assert_eq!(s.aws_credential_export.as_deref(), Some("arn:aws:iam::123:role/X"));
        assert_eq!(s.aws_auth_refresh.as_deref(), Some("aws sso login"));
        assert_eq!(s.gcp_auth_refresh.as_deref(), Some("gcloud auth login"));
        assert_eq!(s.force_login_method.as_deref(), Some("claudeai"));
        assert_eq!(s.force_login_org_uuid.as_deref(), Some("org-xyz"));
        assert_eq!(s.otel_headers_helper.as_deref(), Some("/usr/local/bin/otel-headers"));
        assert_eq!(s.prefers_reduced_motion, Some(true));
        assert_eq!(
            s.company_announcements.as_ref().map(Vec::as_slice),
            Some(&["Welcome".to_string()][..])
        );
        assert_eq!(s.feedback_survey_rate, Some(0.1));
        assert_eq!(s.terminal_title_from_rename, Some(true));
        assert_eq!(s.away_summary_enabled, Some(false));
        assert_eq!(s.show_thinking_summaries, Some(true));
        assert_eq!(s.advisor_model.as_deref(), Some("claude-sonnet-4-6"));
        assert_eq!(s.agent.as_deref(), Some("code-simplifier"));
        assert_eq!(s.auto_dream_enabled, Some(false));
        assert_eq!(s.auto_memory_directory.as_deref(), Some("~/.claude/memory"));
        assert_eq!(s.skill_listing_budget_fraction, Some(0.3));
        assert_eq!(s.skill_listing_max_desc_chars, Some(200));
        assert_eq!(s.skip_web_fetch_preflight, Some(true));
        assert_eq!(s.force_remote_settings_refresh, Some(false));
    }

    #[test]
    fn parse_advanced_subobject_fields_are_value_typed() {
        let json = r#"{
            "attribution": {"enabled": true},
            "autoMode": {"allow": ["Bash"]},
            "fileSuggestion": {"enabled": true},
            "worktree": {"startDirectory": "~/code"},
            "subagentStatusLine": {"type": "command", "command": "echo sub"},
            "spinnerVerbs": {"verbs": ["think"]},
            "spinnerTipsOverride": {"tips": ["tip1"], "excludeDefault": false},
            "remote": {"defaultEnvironmentId": "env-a"}
        }"#;
        let s: Settings = serde_json::from_str(json).expect("should parse");
        assert!(s.attribution.is_some());
        assert!(s.auto_mode.is_some());
        assert!(s.file_suggestion.is_some());
        assert!(s.worktree.is_some());
        assert!(s.subagent_status_line.is_some());
        assert!(s.spinner_verbs.is_some());
        assert!(s.spinner_tips_override.is_some());
        assert!(s.remote.is_some());
        for k in [
            "attribution","autoMode","fileSuggestion","worktree",
            "subagentStatusLine","spinnerVerbs","spinnerTipsOverride","remote",
        ] {
            assert!(!s.extra.contains_key(k));
        }
    }
```

- [ ] **步骤 2：加字段（所有 29 个字段）**

在 `Settings` struct 最后追加（仍在 `extra` 之前）：
```rust
    // Scalar long-tail fields (M8)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_helper: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_credential_export: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_auth_refresh: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_auth_refresh: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_method: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_org_uuid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub otel_headers_helper: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefers_reduced_motion: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_announcements: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_survey_rate: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_title_from_rename: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub away_summary_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_thinking_summaries: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisor_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_dream_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_memory_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_listing_budget_fraction: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_listing_max_desc_chars: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_web_fetch_preflight: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_remote_settings_refresh: Option<bool>,

    // Sub-object fields stored as raw JSON Value (UI: Advanced JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_mode: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_suggestion: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_status_line: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub spinner_verbs: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub spinner_tips_override: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<serde_json::Value>,
```

- [ ] **步骤 3：更新 snapshot 测试 modeled/skipped 列表 —— 把 M8 涉及的 29 个字段全部从 skipped 移到 modeled**

- [ ] **步骤 4：跑测试**

```bash
cargo test -p claude-types
```
Expected：全绿

- [ ] **步骤 5：TS 同步 `Settings` 接口追加**

```typescript
  apiKeyHelper?: string;
  awsCredentialExport?: string;
  awsAuthRefresh?: string;
  gcpAuthRefresh?: string;
  forceLoginMethod?: string;
  forceLoginOrgUUID?: string;
  otelHeadersHelper?: string;
  prefersReducedMotion?: boolean;
  companyAnnouncements?: string[];
  feedbackSurveyRate?: number;
  terminalTitleFromRename?: boolean;
  awaySummaryEnabled?: boolean;
  showThinkingSummaries?: boolean;
  advisorModel?: string;
  agent?: string;
  autoDreamEnabled?: boolean;
  autoMemoryDirectory?: string;
  skillListingBudgetFraction?: number;
  skillListingMaxDescChars?: number;
  skipWebFetchPreflight?: boolean;
  forceRemoteSettingsRefresh?: boolean;
  attribution?: Record<string, unknown>;
  autoMode?: Record<string, unknown>;
  fileSuggestion?: Record<string, unknown>;
  worktree?: Record<string, unknown>;
  subagentStatusLine?: Record<string, unknown>;
  spinnerVerbs?: Record<string, unknown>;
  spinnerTipsOverride?: Record<string, unknown>;
  remote?: Record<string, unknown>;
```

- [ ] **步骤 6：提交**

```bash
cargo test --workspace && pnpm build
git add crates/claude-types/src/settings.rs src/lib/api/types.ts
git commit -m "feat(claude-types,types): add 29 advanced tier-3 fields"
```

### Task 33：i18n（Advanced tab + tier 3 字段）

- [ ] **步骤 1：三个 locale 文件里加 Advanced tab 名和 tier-3 字段 label/tooltip。**

zh-CN.json（节选示例 + tab 名；tooltip 可直接复用 schema snapshot 的 describe 英文，然后翻译）：
```json
  "settings.advanced": "高级",
  "settings.advanced.keyPicker": "字段选择",
  "settings.advanced.searchPlaceholder": "搜索字段...",
  "settings.advanced.hideSet": "隐藏已设置的字段",
  "settings.advanced.groupAuth": "认证 / 运维",
  "settings.advanced.groupExperience": "体验",
  "settings.advanced.groupSubObjects": "子对象（JSON）",
  "settings.advanced.groupMisc": "其他",
  "settings.advanced.groupUnknown": "未识别（未建模）",
  "settings.advanced.noSelection": "从左侧选择一个字段开始编辑",
  "settings.advanced.describeLabel": "说明",
  "settings.advanced.valueLabel": "值",
  "settings.advanced.typeLabel": "类型",
  "settings.advanced.resetButton": "重置为默认（移除此 key）",
  "settings.advanced.invalidJson": "JSON 格式错误",
```

对应 en-US.json：
```json
  "settings.advanced": "Advanced",
  "settings.advanced.keyPicker": "Field Picker",
  "settings.advanced.searchPlaceholder": "Search fields...",
  "settings.advanced.hideSet": "Hide already-set fields",
  "settings.advanced.groupAuth": "Auth / Ops",
  "settings.advanced.groupExperience": "Experience",
  "settings.advanced.groupSubObjects": "Sub-objects (JSON)",
  "settings.advanced.groupMisc": "Misc",
  "settings.advanced.groupUnknown": "Unknown (not modeled)",
  "settings.advanced.noSelection": "Select a field from the left to start editing",
  "settings.advanced.describeLabel": "Description",
  "settings.advanced.valueLabel": "Value",
  "settings.advanced.typeLabel": "Type",
  "settings.advanced.resetButton": "Reset to default (remove key)",
  "settings.advanced.invalidJson": "Invalid JSON",
```

对应 ja-JP.json：
```json
  "settings.advanced": "詳細",
  "settings.advanced.keyPicker": "フィールド選択",
  "settings.advanced.searchPlaceholder": "フィールド検索...",
  "settings.advanced.hideSet": "設定済みフィールドを隠す",
  "settings.advanced.groupAuth": "認証 / 運用",
  "settings.advanced.groupExperience": "体験",
  "settings.advanced.groupSubObjects": "サブオブジェクト（JSON）",
  "settings.advanced.groupMisc": "その他",
  "settings.advanced.groupUnknown": "未識別（未モデル化）",
  "settings.advanced.noSelection": "左側からフィールドを選択して編集開始",
  "settings.advanced.describeLabel": "説明",
  "settings.advanced.valueLabel": "値",
  "settings.advanced.typeLabel": "タイプ",
  "settings.advanced.resetButton": "既定にリセット（キーを削除）",
  "settings.advanced.invalidJson": "JSON 形式が不正",
```

- [ ] **步骤 2：tier-3 字段标签可按需补（本任务先不逐字段翻，AdvancedJsonEditor fallback 显示英文 describe 即可；M9 会扫尾补全）**

- [ ] **步骤 3：提交**

```bash
git add src/lib/locales/
git commit -m "i18n: add Advanced tab keys"
```

### Task 34：Schema snapshot 加载器（前端可访问）

- [ ] **步骤 1：在 `src/lib/api/schema-snapshot.ts` 新建辅助加载器**

创建 `src/lib/api/schema-snapshot.ts`：
```typescript
// Loads the schema snapshot at build time via Vite's JSON import.
// The snapshot lives at docs/claude-schema-snapshot.json.

import snapshot from "../../../docs/claude-schema-snapshot.json";

export interface SchemaField {
  name: string;
  type: string;
  optional?: boolean;
  enumValues?: string[];
  describe?: string;
}

export interface SchemaSnapshot {
  claudeCodeVersion: string;
  extractedAt: string;
  settingsFields: SchemaField[];
  globalFields: { name: string; source: string; type: string; describe?: string }[];
  warnings: string[];
}

export const schemaSnapshot: SchemaSnapshot = snapshot as SchemaSnapshot;

export function categorizeField(name: string): "auth" | "experience" | "subObject" | "misc" {
  const auth = new Set([
    "apiKeyHelper", "awsCredentialExport", "awsAuthRefresh", "gcpAuthRefresh",
    "forceLoginMethod", "forceLoginOrgUUID", "otelHeadersHelper",
  ]);
  const experience = new Set([
    "prefersReducedMotion", "companyAnnouncements", "feedbackSurveyRate",
    "terminalTitleFromRename", "awaySummaryEnabled", "showThinkingSummaries",
  ]);
  const sub = new Set([
    "attribution", "autoMode", "fileSuggestion", "worktree",
    "subagentStatusLine", "spinnerVerbs", "spinnerTipsOverride", "remote",
  ]);
  if (auth.has(name)) return "auth";
  if (experience.has(name)) return "experience";
  if (sub.has(name)) return "subObject";
  return "misc";
}
```

- [ ] **步骤 2：验证**

```bash
pnpm build
```
Expected：无错误（Vite 原生支持 JSON 导入）

- [ ] **步骤 3：提交**

```bash
git add src/lib/api/schema-snapshot.ts
git commit -m "feat(ui): add schema snapshot loader for Advanced tab"
```

### Task 35：JsonValueEditor.svelte（按字段类型切换控件）

- [ ] **步骤 1：创建 `src/lib/components/settings/sub/JsonValueEditor.svelte`**

```svelte
<script lang="ts">
  import { t } from "$lib/i18n";
  import type { SchemaField } from "$lib/api/schema-snapshot";

  let {
    field,
    value = $bindable(),
    onChange,
  }: {
    field: SchemaField;
    value: unknown;
    onChange: () => void;
  } = $props();

  let textBuffer = $state("");
  let parseError = $state("");

  $effect(() => {
    if (field.type === "object" || field.type === "array" || field.type === "record") {
      textBuffer = value === undefined ? "" : JSON.stringify(value, null, 2);
      parseError = "";
    }
  });

  function syncFromText() {
    const trimmed = textBuffer.trim();
    if (!trimmed) {
      value = undefined;
      parseError = "";
      onChange();
      return;
    }
    try {
      value = JSON.parse(trimmed);
      parseError = "";
      onChange();
    } catch (e) {
      parseError = t("settings.advanced.invalidJson") + ": " + String(e);
    }
  }

  function strValue(): string {
    return value === undefined || value === null ? "" : String(value);
  }

  function numValue(): string {
    return typeof value === "number" ? String(value) : "";
  }

  function boolValue(): boolean {
    return Boolean(value);
  }
</script>

<div class="space-y-2">
  {#if field.type === "boolean"}
    <label class="flex items-center gap-3 cursor-pointer">
      <input type="checkbox" checked={boolValue()}
             onchange={(e) => { value = (e.currentTarget as HTMLInputElement).checked; onChange(); }}
             class="h-4 w-4 rounded" style="accent-color: var(--accent-primary)" />
      <span class="text-sm" style="color: var(--text-secondary)">
        {boolValue() ? "on" : "off"}
      </span>
    </label>

  {:else if field.type === "enum"}
    <select value={strValue()}
            onchange={(e) => { value = (e.currentTarget as HTMLSelectElement).value || undefined; onChange(); }}
            class="input-base">
      <option value="">(unset)</option>
      {#each field.enumValues ?? [] as opt}
        <option value={opt}>{opt}</option>
      {/each}
    </select>

  {:else if field.type === "number"}
    <input type="number" value={numValue()}
           oninput={(e) => { const n = Number((e.currentTarget as HTMLInputElement).value); value = Number.isFinite(n) ? n : undefined; onChange(); }}
           class="input-base" />

  {:else if field.type === "string" || field.type === "literal"}
    <input type="text" value={strValue()}
           oninput={(e) => { value = (e.currentTarget as HTMLInputElement).value || undefined; onChange(); }}
           class="input-base" />

  {:else}
    <textarea bind:value={textBuffer} rows="8"
              oninput={syncFromText}
              class="input-base font-mono text-xs"
              placeholder="{}"></textarea>
    {#if parseError}
      <p class="text-xs" style="color: var(--status-error-text)">{parseError}</p>
    {/if}
  {/if}
</div>
```

- [ ] **步骤 2：提交**

```bash
git add src/lib/components/settings/sub/JsonValueEditor.svelte
git commit -m "feat(settings-ui): add JsonValueEditor for Advanced tab"
```

### Task 36：SchemaKeyPicker.svelte

- [ ] **步骤 1：创建 `src/lib/components/settings/sub/SchemaKeyPicker.svelte`**

```svelte
<script lang="ts">
  import { t } from "$lib/i18n";
  import { schemaSnapshot, categorizeField, type SchemaField } from "$lib/api/schema-snapshot";

  let {
    settingsMap,
    selected = $bindable(),
  }: {
    settingsMap: Record<string, unknown>;
    selected: string | null;
  } = $props();

  let search = $state("");
  let hideSet = $state(false);

  // 字段按组收敛
  const groups = $derived.by(() => {
    const g: Record<string, SchemaField[]> = {
      auth: [],
      experience: [],
      subObject: [],
      misc: [],
      unknown: [],
    };
    const modeled = new Set(schemaSnapshot.settingsFields.map((f) => f.name));

    for (const f of schemaSnapshot.settingsFields) {
      const cat = categorizeField(f.name);
      g[cat].push(f);
    }

    // settings 里有但 snapshot 不认识的 key → unknown
    for (const key of Object.keys(settingsMap)) {
      if (!modeled.has(key) && key !== "extra") {
        g.unknown.push({ name: key, type: "unknown", describe: "Not in schema snapshot" });
      }
    }

    for (const arr of Object.values(g)) {
      arr.sort((a, b) => a.name.localeCompare(b.name));
    }
    return g;
  });

  function matches(f: SchemaField): boolean {
    if (search && !f.name.toLowerCase().includes(search.toLowerCase())) return false;
    if (hideSet && settingsMap[f.name] !== undefined) return false;
    return true;
  }
</script>

<div class="space-y-3 overflow-auto h-full" style="max-height: calc(100vh - 200px);">
  <div class="space-y-2 sticky top-0 z-10 pb-2"
       style="background-color: var(--bg-primary); border-bottom: 1px solid var(--border-color)">
    <input type="text" bind:value={search}
           placeholder={t("settings.advanced.searchPlaceholder")}
           class="input-base text-sm" />
    <label class="flex items-center gap-2 text-xs">
      <input type="checkbox" bind:checked={hideSet}
             class="h-3 w-3 rounded" style="accent-color: var(--accent-primary)" />
      <span style="color: var(--text-secondary)">
        {t("settings.advanced.hideSet")}
      </span>
    </label>
  </div>

  {#each [
    ["auth", t("settings.advanced.groupAuth")],
    ["experience", t("settings.advanced.groupExperience")],
    ["subObject", t("settings.advanced.groupSubObjects")],
    ["misc", t("settings.advanced.groupMisc")],
    ["unknown", t("settings.advanced.groupUnknown")],
  ] as [groupKey, label] (groupKey)}
    {@const visibleFields = groups[groupKey].filter(matches)}
    {#if visibleFields.length > 0}
      <div class="space-y-1">
        <h4 class="text-xs font-semibold uppercase" style="color: var(--text-muted)">
          {label} ({visibleFields.length})
        </h4>
        <ul class="text-sm">
          {#each visibleFields as f (f.name)}
            <li>
              <button type="button"
                      onclick={() => (selected = f.name)}
                      class="w-full text-left px-2 py-1 rounded text-xs"
                      style={selected === f.name
                        ? "background-color: var(--bg-accent); color: var(--text-primary)"
                        : "color: var(--text-secondary)"}>
                <span class="font-mono">{f.name}</span>
                {#if settingsMap[f.name] !== undefined}
                  <span style="color: var(--accent-primary)">✓</span>
                {/if}
                <span class="ml-1" style="color: var(--text-muted)">
                  ({f.type}{f.enumValues ? ` · ${f.enumValues.length}` : ""})
                </span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  {/each}
</div>
```

- [ ] **步骤 2：提交**

```bash
git add src/lib/components/settings/sub/SchemaKeyPicker.svelte
git commit -m "feat(settings-ui): add SchemaKeyPicker for Advanced tab"
```

### Task 37：AdvancedJsonEditor.svelte

- [ ] **步骤 1：创建 `src/lib/components/settings/AdvancedJsonEditor.svelte`**

```svelte
<script lang="ts">
  import { configStore } from "$lib/stores/config.svelte";
  import JsonPreview from "./JsonPreview.svelte";
  import SchemaKeyPicker from "./sub/SchemaKeyPicker.svelte";
  import JsonValueEditor from "./sub/JsonValueEditor.svelte";
  import { schemaSnapshot, type SchemaField } from "$lib/api/schema-snapshot";
  import { t } from "$lib/i18n";

  const settings = $derived(configStore.activeSettings);
  const settingsMap = $derived(settings as Record<string, unknown>);

  let selectedKey = $state<string | null>(null);
  const selectedField = $derived<SchemaField | null>(
    selectedKey
      ? schemaSnapshot.settingsFields.find((f) => f.name === selectedKey) ?? {
          name: selectedKey,
          type: "unknown",
          describe: "Not in schema snapshot",
        }
      : null,
  );
  let pendingValue = $state<unknown>(undefined);

  $effect(() => {
    if (selectedKey) {
      pendingValue = settingsMap[selectedKey];
    } else {
      pendingValue = undefined;
    }
  });

  function saveField() {
    if (!selectedKey) return;
    configStore.save({ [selectedKey]: pendingValue });
  }

  function resetField() {
    if (!selectedKey) return;
    pendingValue = undefined;
    configStore.save({ [selectedKey]: undefined });
  }
</script>

<div class="flex gap-4 h-full">
  <!-- Left: key picker -->
  <aside class="w-80 shrink-0 border-r pr-4"
         style="border-color: var(--border-color)">
    <h3 class="text-sm font-semibold mb-3" style="color: var(--text-primary)">
      {t("settings.advanced.keyPicker")}
    </h3>
    <SchemaKeyPicker {settingsMap} bind:selected={selectedKey} />
  </aside>

  <!-- Right: value editor -->
  <section class="flex-1 min-w-0">
    {#if !selectedField}
      <p class="text-sm" style="color: var(--text-muted)">
        {t("settings.advanced.noSelection")}
      </p>
    {:else}
      <div class="space-y-3 max-w-2xl">
        <div>
          <h3 class="text-lg font-semibold font-mono" style="color: var(--text-primary)">
            {selectedField.name}
          </h3>
          <p class="text-xs font-mono" style="color: var(--text-muted)">
            {t("settings.advanced.typeLabel")}: {selectedField.type}
            {selectedField.enumValues
              ? " · enum(" + selectedField.enumValues.join(", ") + ")"
              : ""}
          </p>
        </div>

        {#if selectedField.describe}
          <div>
            <label class="block text-xs font-semibold uppercase mb-1"
                   style="color: var(--text-muted)">
              {t("settings.advanced.describeLabel")}
            </label>
            <p class="text-sm" style="color: var(--text-secondary)">
              {selectedField.describe}
            </p>
          </div>
        {/if}

        <div>
          <label class="block text-xs font-semibold uppercase mb-1"
                 style="color: var(--text-muted)">
            {t("settings.advanced.valueLabel")}
          </label>
          <JsonValueEditor
            field={selectedField}
            bind:value={pendingValue}
            onChange={() => configStore.markDirty()} />
        </div>

        <div class="flex gap-2 pt-3 border-t" style="border-color: var(--border-color)">
          <button type="button" onclick={saveField}
                  disabled={!configStore.isDirty || configStore.saving}
                  class="btn-primary text-sm px-4 py-2">
            {configStore.saving ? t("common.saving") : t("common.save")}
          </button>
          <button type="button" onclick={resetField}
                  class="btn-secondary text-sm px-4 py-2">
            {t("settings.advanced.resetButton")}
          </button>
        </div>

        <JsonPreview
          data={{ [selectedField.name]: pendingValue }}
          title="Current field" />
      </div>
    {/if}
  </section>
</div>
```

- [ ] **步骤 2：在 SettingsEditor.svelte 注册**

```typescript
  import AdvancedJsonEditor from "./AdvancedJsonEditor.svelte";
```

模板追加：
```svelte
  {:else if activeSection === "advanced"}
    <AdvancedJsonEditor />
```

- [ ] **步骤 3：App.svelte `settingsSections` 追加**

```typescript
    { id: "advanced", labelKey: "settings.advanced" },
```

- [ ] **步骤 4：重启 dev server + 冒烟**

```bash
pnpm tauri dev
```

1. 打开 Advanced tab
2. 左侧按 Auth / Experience / Sub-objects / Misc / Unknown 分组
3. 搜索 `apiKeyHelper`，选中，右侧看到 describe 和 string 输入框
4. 填入 `/usr/local/bin/foo`，点 Save
5. 打开 settings.json 确认 `apiKeyHelper` 已写入
6. 回到 UI，点 "Reset to default (remove key)"，文件里该 key 消失
7. 手工在 settings.json 加一个 `someUnknownKey: "x"`，刷新 UI，Advanced 的 Unknown 分组出现该 key

- [ ] **步骤 5：提交**

```bash
git add src/lib/components/settings/AdvancedJsonEditor.svelte src/lib/components/settings/SettingsEditor.svelte src/App.svelte
git commit -m "feat(settings-ui): add Advanced JSON tab"
```

### Task 38：M8 收尾

- [ ] **步骤 1：fixture 追加 Advanced 字段覆盖**

在 `tests/fixtures/settings-full.json` 根对象追加：
```json
  "apiKeyHelper": "/usr/local/bin/get-api-key",
  "awsCredentialExport": "arn:aws:iam::123:role/foo",
  "awsAuthRefresh": "aws sso login",
  "gcpAuthRefresh": "gcloud auth login",
  "forceLoginMethod": "claudeai",
  "forceLoginOrgUUID": "org-xyz",
  "otelHeadersHelper": "/usr/local/bin/otel-headers",
  "prefersReducedMotion": false,
  "companyAnnouncements": ["Welcome to the team!"],
  "feedbackSurveyRate": 0.1,
  "terminalTitleFromRename": true,
  "awaySummaryEnabled": false,
  "showThinkingSummaries": true,
  "advisorModel": "claude-sonnet-4-6",
  "agent": "code-simplifier",
  "autoDreamEnabled": false,
  "autoMemoryDirectory": "~/.claude/memory",
  "skillListingBudgetFraction": 0.3,
  "skillListingMaxDescChars": 200,
  "skipWebFetchPreflight": false,
  "forceRemoteSettingsRefresh": false,
  "attribution": { "enabled": true },
  "autoMode": { "allow": [], "soft_deny": [] },
  "fileSuggestion": { "enabled": true },
  "worktree": { "startDirectory": "~/code" },
  "subagentStatusLine": { "type": "command", "command": "echo sub" },
  "spinnerVerbs": { "verbs": ["cook", "think"] },
  "spinnerTipsOverride": { "tips": ["tip1"], "excludeDefault": false },
  "remote": { "defaultEnvironmentId": "env-a" },
```

- [ ] **步骤 2：跑 test + build**

```bash
cargo test --workspace && pnpm build
```

- [ ] **步骤 3：打 tag**

```bash
git add tests/fixtures/settings-full.json
git commit -m "test: cover advanced fields in fixture"
git push
git tag milestone/M8-advanced-tab
git push --tags
```

---

## Milestone 9 — i18n 扫尾 + 端到端回归

**目标**：补翻漏翻字段；端到端跑一遍完整 fixture；回归清单逐项勾选。

### Task 39：i18n 扫尾

- [ ] **步骤 1：检查每个 tab 的所有可见 label 都有对应 key**

打开 `pnpm tauri dev`，切到各个 tab：
- General
- Env
- Permissions
- Hooks
- Sandbox
- Status Line
- Runtime
- MCP
- Plugins & Marketplace
- Advanced

用浏览器 DevTools 的 Elements tab 搜索 `settings.fields.`，确保每条显示都解析为翻译文本（不是 key 自身）。

若发现未翻译的键（DevTools 控制台会有 `[i18n] missing key:` 警告），在三个 locale 文件分别补充。

- [ ] **步骤 2：语言切换对比**

在 App Settings tab 切换 `zh-CN` / `en-US` / `ja-JP`，每切一次回到 Settings 下每个 tab 扫一遍。确认：
- 所有 label 都随语言切换
- Advanced tab 对 tier-3 字段 fallback 显示英文 describe（可接受）
- Tooltip 内容语言匹配

- [ ] **步骤 3：修复在步骤 1/2 发现的漏翻**

逐个 key 补到 `zh-CN.json` / `en-US.json` / `ja-JP.json`。

- [ ] **步骤 4：跑 build 确认**

```bash
pnpm build
```
Expected：无错误

- [ ] **步骤 5：提交**

```bash
git add src/lib/locales/
git commit -m "i18n: M9 sweep — fix missing translations"
```

### Task 40：端到端回归

- [ ] **步骤 1：完整 fixture roundtrip 测试**

```bash
cargo test -p claude-types parse_full_settings
```
Expected：PASS（fixture 现含 ~55 字段，全部建模）

- [ ] **步骤 2：snapshot 对齐测试**

```bash
cargo test -p claude-types settings_struct_matches_schema_snapshot
```
Expected：PASS。如果 PASS 但 `skipped` 列表仍很长，仔细审查：哪些字段是 sub-object 内部字段（`"name"`, `"command"`, `"padding"` 等，本身不是 Settings 顶层 key），哪些确实漏建模——漏的加到对应里程碑里或单独补。

- [ ] **步骤 3：workspace 测试**

```bash
cargo test --workspace
```
Expected：全绿

- [ ] **步骤 4：TS build**

```bash
pnpm build
```
Expected：无错误

- [ ] **步骤 5：用完整 fixture 做手工端到端**

```bash
cp tests/fixtures/settings-full.json ~/.claude/settings.json
pnpm tauri dev
```

（⚠️ 先把你现有的 `~/.claude/settings.json` 备份：`cp ~/.claude/settings.json ~/.claude/settings.json.bak.M9`）

在 UI 里：
1. 切换 10 个 tab，每个 tab 的字段都应回显 fixture 里的值
2. 每个 tab 随机改 1-2 字段
3. 点 Save
4. `diff tests/fixtures/settings-full.json ~/.claude/settings.json` 应只展示改过的 key
5. 退出 tauri dev，启动真实 claude CLI：`claude --help`
6. 确认 Claude Code 启动无 schema validation 报错

- [ ] **步骤 6：恢复你自己的 settings.json**

```bash
mv ~/.claude/settings.json.bak.M9 ~/.claude/settings.json
```

- [ ] **步骤 7：回归清单（手工逐项勾选）**

- [ ] 既有 General tab 所有旧字段仍可编辑
- [ ] Permissions allow/deny/ask 列表增删
- [ ] Hooks 事件列表增删（PreToolUse、PostToolUse 等）
- [ ] Sandbox allow/deny paths 增删
- [ ] Status Line 切换 type，测试 command
- [ ] Env tab CRUD
- [ ] Effective Config view（侧边导航 "有效配置"）能合并所有层
- [ ] 外部修改 `~/.claude/settings.json` 触发 `config-changed` 事件（UI 实时刷新）
- [ ] 切换 project scope，项目 settings 正确加载
- [ ] Ctrl+R 刷新 UI 状态一致

- [ ] **步骤 8：提交验收笔记**

如果步骤 7 任何一项不通过，记到 issue 里但不阻塞 M9 验收（那是回归 bug，新开 issue）。

```bash
git commit --allow-empty -m "chore: M9 regression verification complete"
```

### Task 41：M9 收尾与分支合并

- [ ] **步骤 1：推送、打 tag**

```bash
git push
git tag milestone/M9-regression
git push --tags
```

- [ ] **步骤 2：建 PR（如果使用 PR 流程）**

```bash
gh pr create --title "feat(settings): schema catchup — 9 milestones" --body "$(cat <<'EOF'
## Summary

一次性把 `~/.claude/settings.json` Rust Settings struct 从 15 字段扩展到 ~55，
新增 Runtime / MCP / Plugins & Marketplace / Advanced 四个 GUI tab，
扩展 General / Hooks 现有 tab，兑现 Claude Code 2.1.110+ 新字段
（`tui`、`effortLevel.xhigh`），并引入 `pnpm check:schema` 漂移检测
+ 每周一 CI。

## 里程碑

- M1 Schema 抽取工具 + CI
- M2 tui + effortLevel
- M3 Runtime tab
- M4 General 扩展
- M5 MCP tab
- M6 Plugins & Marketplace tab
- M7 Hooks Policy 扩展
- M8 Advanced JSON tab
- M9 i18n 扫尾 + 回归验收

## Test plan

- [x] cargo test --workspace 全绿
- [x] pnpm build 无错误
- [x] 完整 fixture roundtrip
- [x] snapshot 对齐测试
- [x] 10 个 tab 手工冒烟
- [x] `claude` CLI 启动无 schema 报错
- [x] i18n 三语覆盖

关联 spec：`docs/superpowers/specs/2026-04-17-settings-json-schema-catchup-design.md`
EOF
)"
```

- [ ] **步骤 3：合并策略**

PR 审查通过后按项目习惯合并（merge commit 或 rebase）。不做 force push。合并后在本地：
```bash
git checkout main
git pull
git branch -d feat/settings-schema-catchup
```

---

## 完整回顾：本计划覆盖 spec 所有目标

| Spec 目标 | 里程碑 |
|----------|-------|
| Settings struct 从 15 扩展到 ~55 字段 | M2, M3, M4, M5, M6, M7, M8 |
| GUI tab 数从 6 扩展到 10 | M3, M5, M6, M8 |
| Release notes 2.1.110+ 字段（tui, effortLevel.xhigh） | M2 |
| Schema 漂移检测（extract + check + CI） | M1 |
| i18n zh-CN / en-US / ja-JP | M2, M3, M4, M5, M6, M7, M8, M9 |
| 每里程碑独立可合入、独立可验收 | 全程 |
| 类型化测试 + snapshot 对齐 | M1, M2, M3, M4, M5, M6, M7, M8 |
| 端到端回归 | M9 |
| 不使用大 PR 模式（单分支分里程碑 tag） | 全程 |

