#!/usr/bin/env tsx
// scripts/extract-claude-schema.ts
//
// Build the Claude Code schema snapshot by combining two sources:
//   1. Live fetch of the SchemaStore claude-code-settings.json
//      (community-maintained, tracks upstream within days/weeks)
//   2. docs/schema-extras.json — frozen baseline for fields not yet upstream
//      (real settings missing from schemastore + frozen TUI globalFields
//       extracted from cli.js back when 2.1.112 was current)
//
// Outputs JSON to stdout (or --out <path>) in the same shape the previous
// cli.js-based extractor used, so check-claude-schema.ts and the Rust
// snapshot-alignment test continue to work unchanged.
//
// Usage:
//   pnpm extract:schema
//   pnpm extract:schema --out docs/claude-schema-snapshot.json
//   CLAUDE_CODE_SCHEMA_URL=https://example.com/custom.json pnpm extract:schema

import { writeFileSync, readFileSync, existsSync } from "node:fs";

const SCHEMASTORE_URL =
  process.env.CLAUDE_CODE_SCHEMA_URL ??
  "https://json.schemastore.org/claude-code-settings.json";
const EXTRAS_PATH = "docs/schema-extras.json";

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

interface ExtrasFile {
  lastReviewed: string;
  sourceVersion: string;
  settingsFields: SchemaField[];
  globalFields: GlobalField[];
}

async function fetchSchemastore(url: string): Promise<{ json: any; etag?: string }> {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`Failed to fetch ${url}: ${res.status} ${res.statusText}`);
  }
  const etag = res.headers.get("etag") ?? undefined;
  const json = await res.json();
  return { json, etag };
}

/**
 * Convert a single JSON Schema property entry to our SchemaField shape.
 * Handles the simple cases observed in the schemastore claude-code schema:
 * top-level `properties` are all simple-typed (string/object/array/number/
 * boolean) with optional `enum` and `description`. No oneOf/anyOf at top level.
 */
function propertyToField(name: string, prop: any): SchemaField {
  const field: SchemaField = {
    name,
    type: prop.type ?? "ref",
    optional: true,
  };
  if (Array.isArray(prop.enum)) {
    field.type = "enum";
    field.enumValues = prop.enum.map(String);
  }
  if (typeof prop.description === "string") {
    field.describe = prop.description;
  }
  return field;
}

function loadExtras(): ExtrasFile {
  if (!existsSync(EXTRAS_PATH)) {
    throw new Error(
      `Extras file not found at ${EXTRAS_PATH}. ` +
        `This file holds fields not yet in schemastore plus the frozen TUI globalFields baseline.`
    );
  }
  return JSON.parse(readFileSync(EXTRAS_PATH, "utf8"));
}

function mergeSettingsFields(
  fromSchemastore: SchemaField[],
  fromExtras: SchemaField[]
): { merged: SchemaField[]; collisions: string[] } {
  const byName = new Map<string, SchemaField>();
  for (const f of fromSchemastore) byName.set(f.name, f);

  const collisions: string[] = [];
  for (const f of fromExtras) {
    if (byName.has(f.name)) {
      // Schemastore has caught up with this extra; prefer schemastore's
      // definition and surface a warning so the maintainer knows to drop
      // the now-redundant entry from schema-extras.json.
      collisions.push(f.name);
    } else {
      byName.set(f.name, f);
    }
  }
  const merged = Array.from(byName.values()).sort((a, b) =>
    a.name.localeCompare(b.name)
  );
  return { merged, collisions };
}

async function extract(): Promise<SchemaSnapshot> {
  const { json: schemastore, etag } = await fetchSchemastore(SCHEMASTORE_URL);
  const extras = loadExtras();

  const schemastoreFields: SchemaField[] = [];
  for (const [name, prop] of Object.entries(schemastore.properties ?? {})) {
    if (name === "$schema") continue;
    schemastoreFields.push(propertyToField(name, prop));
  }

  const { merged, collisions } = mergeSettingsFields(
    schemastoreFields,
    extras.settingsFields
  );

  const warnings: string[] = [];
  if (collisions.length > 0) {
    warnings.push(
      `${collisions.length} extras now also present in schemastore; ` +
        `consider removing from schema-extras.json: ${collisions.join(", ")}`
    );
  }

  return {
    claudeCodeVersion: `schemastore@${etag ?? "unknown"}+extras@${extras.sourceVersion}`,
    extractedAt: new Date().toISOString(),
    settingsFields: merged,
    globalFields: extras.globalFields,
    warnings,
  };
}

async function main() {
  const args = process.argv.slice(2);
  const outIdx = args.indexOf("--out");
  const outPath = outIdx >= 0 ? args[outIdx + 1] : null;

  const snapshot = await extract();
  const json = JSON.stringify(snapshot, null, 2) + "\n";

  if (outPath) {
    writeFileSync(outPath, json);
    console.error(
      `Wrote snapshot to ${outPath} (settings=${snapshot.settingsFields.length}, ` +
        `global=${snapshot.globalFields.length})`
    );
  } else {
    process.stdout.write(json);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
