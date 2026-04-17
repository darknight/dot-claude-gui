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

Homebrew cask (`brew install claude-code`) 安装的是 single-binary，不含 cli.js；
需要 `npm i -g @anthropic-ai/claude-code` 或从 tarball 解压出 cli.js 后用 env 指向。

### Regex 维护

脚本靠 regex 从 minified `cli.js` 抽 zod 字段。抽取策略：
- **Settings fields**：从 `tui:y.enum` anchor 出发定位顶层 `y.object({...})`，
  brace-depth 跟踪 + 字符串字面量跳过，只在深度 1 时捕获字段。
- **Ref fields**：`env`/`hooks`/`permissions`/`sandbox` 在 cli.js 里是
  `name:helperFn()` 形式而非 `y.xxx()` 链，需要单独一轮 regex 捕获。
- **Global fields**：`name:{source:"settings|global",type:"...",description:"..."}`
  形式从 `/config` 命令描述表抽取。

如果某字段没被抽到但实际存在，`snapshot.warnings` 会列出。此时：
1. 手工 grep 该字段在 cli.js 里的实际 pattern
2. 升级 `extractSettingsFields` / `extractGlobalFields` 的 regex
3. 重跑 `pnpm extract:schema` 确认字段进入

### 已知限制

- `theme` 等 options 定义为变量引用（`options:ZY4`）的 global 字段，
  静态抽取拿不到合法值，需要手工维护。
- cli.js 压缩器若换新版本（如 esbuild → terser），anchor / depth 逻辑可能失效。
  anchor 若找不到会打 warning 并 fallback 到 whole-file 扫描（会丢 ref 字段，
  字段数会暴涨到 ~800）。出现这种情况要重写 anchor 策略。
