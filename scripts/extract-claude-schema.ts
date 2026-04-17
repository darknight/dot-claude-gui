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

/**
 * Slice the value expression of a single zod field. Starts at `matchEnd` (right
 * after `name:y.type(`). Tracks paren, brace, bracket, and quote depth so that
 * braces inside strings or nested calls don't confuse boundary detection.
 *
 * Terminates at the first top-level `,` or `}` — that's where the next field
 * starts (or the enclosing object ends). Returns the substring between
 * matchEnd and that terminator. Capped at 2000 chars as a safety net.
 */
function sliceFieldTail(source: string, matchEnd: number): string {
  const maxLen = 2000;
  let paren = 1; // we're inside the opening `(` from name:y.type(
  let brace = 0;
  let bracket = 0;
  let i = matchEnd;
  const end = Math.min(source.length, matchEnd + maxLen);

  while (i < end) {
    const ch = source[i];

    // Skip string/template literals
    if (ch === '"' || ch === "'" || ch === "`") {
      const quote = ch;
      i++;
      while (i < end && source[i] !== quote) {
        if (source[i] === "\\") i++;
        i++;
      }
      i++;
      continue;
    }

    if (ch === "(") paren++;
    else if (ch === ")") paren--;
    else if (ch === "{") brace++;
    else if (ch === "}") {
      if (paren === 0 && brace === 0 && bracket === 0) break;
      brace--;
    } else if (ch === "[") bracket++;
    else if (ch === "]") bracket--;
    else if (ch === "," && paren === 0 && brace === 0 && bracket === 0) {
      break;
    }

    i++;
  }

  return source.slice(matchEnd, i);
}

/**
 * Extract the outermost `.describe("...")` text from a field tail.
 *
 * The tail begins inside the opening `(` of `y.<type>(`. Nested schemas (e.g.
 * `y.array(y.object({ id:y.string().describe("…") }))`) contain their own
 * `.describe()` calls at deeper paren/brace depths; we want the one attached to
 * the field's own value expression, which appears at paren=0 / brace=0 /
 * bracket=0 after the outer `y.<type>(...)` closes.
 *
 * Returns the string literal body, or undefined if no top-level describe exists.
 */
function findOwnDescribe(tail: string): string | undefined {
  let paren = 1; // tail starts inside `y.<type>(`
  let brace = 0;
  let bracket = 0;
  let i = 0;
  while (i < tail.length) {
    const ch = tail[i];

    // Skip strings
    if (ch === '"' || ch === "'" || ch === "`") {
      const quote = ch;
      i++;
      while (i < tail.length && tail[i] !== quote) {
        if (tail[i] === "\\") i++;
        i++;
      }
      i++;
      continue;
    }

    if (ch === "(") {
      // Only consider `.describe(` when we're at the top level (paren=0) of
      // the field's chained calls. The outer `y.<type>(` starts at paren=1, so
      // we look for `.describe(` matched when paren transitions 0 -> 1.
      if (
        paren === 0 &&
        brace === 0 &&
        bracket === 0 &&
        tail.slice(Math.max(0, i - 9), i) === ".describe"
      ) {
        // Parse the string literal argument
        let j = i + 1;
        // skip whitespace
        while (j < tail.length && /\s/.test(tail[j])) j++;
        const q = tail[j];
        if (q === '"' || q === "'" || q === "`") {
          j++;
          let buf = "";
          while (j < tail.length && tail[j] !== q) {
            if (tail[j] === "\\" && j + 1 < tail.length) {
              // unescape simple cases
              const next = tail[j + 1];
              if (next === "n") buf += "\n";
              else if (next === "t") buf += "\t";
              else if (next === "r") buf += "\r";
              else buf += next;
              j += 2;
              continue;
            }
            buf += tail[j];
            j++;
          }
          return buf;
        }
        // Not a string literal argument — skip
      }
      paren++;
    } else if (ch === ")") paren--;
    else if (ch === "{") brace++;
    else if (ch === "}") brace--;
    else if (ch === "[") bracket++;
    else if (ch === "]") bracket--;

    i++;
  }
  return undefined;
}

function extractSettingsFields(source: string, warnings: string[]): SchemaField[] {
  // Anchor to a known top-level field that lives at depth 1 of the settings schema.
  // Use a plain-string search for `tui:y.enum` (the settings object may contain nested
  // y.object({...}) blocks before tui, so a `[^{}]`-bounded regex would miss it).
  const anchorKey = "tui:y.enum";
  const anchorIdx = source.indexOf(anchorKey);
  if (anchorIdx < 0) {
    warnings.push(
      "Could not find settings schema anchor (tui enum); falling back to whole-file scan",
    );
    return extractSettingsFieldsWholeFile(source);
  }

  // Walk backwards from the anchor tracking brace depth to locate the enclosing
  // `y.object({` that opens the settings schema. We cannot just use
  // `lastIndexOf("y.object({")` because the nearest `y.object({` may belong to a
  // nested field schema (e.g. remoteSession), not the top-level settings object.
  let objectStart = -1;
  {
    let depth = 0; // counts unmatched `}` seen while walking left
    let j = anchorIdx - 1;
    while (j >= 0) {
      const ch = source[j];
      if (ch === "}") {
        depth++;
      } else if (ch === "{") {
        if (depth === 0) {
          // This `{` opens the block that contains the anchor.
          // Confirm it is preceded by `y.object(`.
          const prefix = source.slice(Math.max(0, j - 10), j + 1);
          if (prefix.endsWith("y.object({")) {
            objectStart = j - "y.object(".length;
          }
          break;
        }
        depth--;
      }
      // NOTE: we do not skip string literals here. Zod schema sources use a
      // constrained JS subset; quoted `{`/`}` inside .describe() are rare and
      // the anchor is close enough that string scanning is not needed.
      j--;
    }
  }

  if (objectStart < 0) {
    warnings.push("Anchor found but could not locate enclosing y.object(");
    return extractSettingsFieldsWholeFile(source);
  }

  // Walk from after "y.object({" tracking brace depth.
  // depth starts at 1 (we just entered the object).
  const fields: SchemaField[] = [];
  const seen = new Set<string>();
  // Direct zod field: `name:y.<type>(`
  const topRe =
    /([a-zA-Z_][a-zA-Z0-9_]{1,40}):y\.(string|boolean|number|enum|array|object|record|union|literal|any)\(/g;
  // Function-reference field: `name:<Ident>(` (e.g. `hooks:sN()`, `env:Li5()`).
  // Minified names are typically short identifiers; we require an opening `(` to
  // distinguish from plain value assignments like `mode:"x"`.
  const refRe =
    /([a-zA-Z_][a-zA-Z0-9_]{1,40}):([A-Za-z_$][A-Za-z0-9_$]{0,40})\(/g;

  let depth = 1;
  let i = objectStart + "y.object({".length;

  while (i < source.length && depth > 0) {
    const ch = source[i];
    if (ch === "{") {
      depth++;
      i++;
      continue;
    }
    if (ch === "}") {
      depth--;
      i++;
      continue;
    }
    // Skip string literals to avoid counting { / } inside them
    if (ch === '"' || ch === "'" || ch === "`") {
      const quote = ch;
      i++;
      while (i < source.length && source[i] !== quote) {
        if (source[i] === "\\") i++; // skip escape
        i++;
      }
      i++;
      continue;
    }
    if (depth === 1) {
      topRe.lastIndex = i;
      const m = topRe.exec(source);
      if (m && m.index === i) {
        const name = m[1];
        const type = m[2];
        if (!seen.has(name)) {
          seen.add(name);
          const matchEnd = m.index + m[0].length;
          const tail = sliceFieldTail(source, matchEnd);
          const optional = /\)\.optional\(\)/.test(tail);
          let enumValues: string[] | undefined;
          if (type === "enum") {
            // enum values live at the very start of the value expression:
            // `y.enum([...])`; search within the (bounded) tail prefix.
            const em = source
              .slice(m.index, matchEnd + tail.length)
              .match(/y\.enum\(\[([^\]]+)\]/);
            if (em) {
              enumValues = em[1]
                .split(",")
                .map((s) => s.trim().replace(/^"|"$/g, ""));
            } else {
              warnings.push(`Could not extract enum values for ${name}`);
            }
          }
          const describe = findOwnDescribe(tail);
          fields.push({ name, type, optional, enumValues, describe });
        }
        i = m.index + m[0].length;
        continue;
      }
      // Not a direct y.<type> match — try function-reference (e.g. `hooks:sN()`).
      refRe.lastIndex = i;
      const rm = refRe.exec(source);
      if (rm && rm.index === i && rm[2] !== "y") {
        const name = rm[1];
        const fnRef = rm[2];
        if (!seen.has(name)) {
          seen.add(name);
          const matchEnd = rm.index + rm[0].length;
          const tail = sliceFieldTail(source, matchEnd);
          const optional = /\)\.optional\(\)/.test(tail);
          const ownDesc = findOwnDescribe(tail);
          // Record the referenced validator name in describe (prefixed) so
          // downstream readers can see this is a function-reference field
          // without guessing.
          const describe = ownDesc ?? `(validator: ${fnRef}())`;
          fields.push({ name, type: "ref", optional, describe });
        }
        i = rm.index + rm[0].length;
        continue;
      }
    }
    i++;
  }

  return fields;
}

function extractSettingsFieldsWholeFile(source: string): SchemaField[] {
  // Fallback: original whole-file scan. Captures sub-object fields too.
  const fields: SchemaField[] = [];
  const re =
    /([a-zA-Z_][a-zA-Z0-9_]{1,40}):y\.(string|boolean|number|enum|array|object|record|union|literal|any)\(/g;
  const seen = new Set<string>();

  for (const m of source.matchAll(re)) {
    const name = m[1];
    const type = m[2];
    if (seen.has(name)) continue;
    seen.add(name);

    const matchEnd = m.index! + m[0].length;
    const tail = sliceFieldTail(source, matchEnd);
    const optional = /\)\.optional\(\)/.test(tail);

    let enumValues: string[] | undefined;
    if (type === "enum") {
      const enumMatch = source
        .slice(m.index!, matchEnd + tail.length)
        .match(/y\.enum\(\[([^\]]+)\]/);
      if (enumMatch) {
        enumValues = enumMatch[1].split(",").map((s) => s.trim().replace(/^"|"$/g, ""));
      }
    }

    const describe = findOwnDescribe(tail);
    fields.push({ name, type, optional, enumValues, describe });
  }

  return fields;
}

function extractGlobalFields(source: string, warnings: string[]): GlobalField[] {
  const fields: GlobalField[] = [];
  // Matches: foo:{source:"global",type:"...",description:"…"}
  //          foo:{source:"settings",type:"...",description:"…",options:[…]}
  // We need to match the whole braced object because options can follow description.
  const re =
    /([a-zA-Z_][a-zA-Z0-9_.]{1,40}):\{source:"(settings|global)",type:"([a-zA-Z_]+)",description:([^{}]{0,800})\}/g;

  for (const m of source.matchAll(re)) {
    const name = m[1];
    const src = m[2] as "settings" | "global";
    const type = m[3];
    const descPayload = m[4];

    let describe: string | undefined;
    const descMatch = descPayload.match(/^["'`]([^"'`]+)["'`]/);
    if (descMatch) describe = descMatch[1];
    else warnings.push(`Could not parse description for ${name}`);

    let options: string[] | undefined;
    const optMatch = descPayload.match(/options:\[([^\]]+)\]/);
    if (optMatch) {
      options = optMatch[1]
        .split(",")
        .map((s) => s.trim().replace(/^"|"$/g, ""))
        .filter((s) => s.length > 0 && !s.includes(".") && !s.includes("("));
      if (options.length === 0) options = undefined;
    }

    fields.push({ name, source: src, type, describe, options });
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
