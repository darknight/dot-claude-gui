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
