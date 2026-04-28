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
      const beforeEnum = JSON.stringify([...(before.enumValues ?? [])].sort());
      const afterEnum = JSON.stringify([...(f.enumValues ?? [])].sort());
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
