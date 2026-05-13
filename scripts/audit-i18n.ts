#!/usr/bin/env tsx
// Locale audit: flags non-en-US values that look like English-as-placeholder
// (pure ASCII while en-US value is a normal English sentence), and locale-parity
// gaps (keys present in en-US but missing in another locale).
//
// Exit codes:
//   0 — no suspect values found
//   1 — at least one suspect value found (gates Stage 5 exit)

import { readFileSync, readdirSync } from "node:fs";
import { join, basename } from "node:path";

type Whitelist = {
  keys: string[];
  keySuffixes: string[];
  valuePatterns: string[];
};

const LOCALE_DIR = join(process.cwd(), "src", "lib", "locales");
const REFERENCE = "en-US";
const WHITELIST_PATH = join(process.cwd(), "scripts", "audit-i18n.whitelist.json");

const whitelist = JSON.parse(readFileSync(WHITELIST_PATH, "utf8")) as Whitelist;
const valueRegexes = whitelist.valuePatterns.map((p) => new RegExp(p));

function loadLocale(file: string): Record<string, string> {
  const raw = JSON.parse(readFileSync(join(LOCALE_DIR, file), "utf8"));
  return raw as Record<string, string>;
}

function isWhitelisted(key: string, value: string): boolean {
  if (whitelist.keys.includes(key)) return true;
  if (whitelist.keySuffixes.some((s) => key.endsWith(s))) return true;
  if (valueRegexes.some((re) => re.test(value))) return true;
  return false;
}

function isNormalEnglishSentence(value: string): boolean {
  return value.length >= 4 || value.includes(" ");
}

function isPureAscii(value: string): boolean {
  return /^[\x00-\x7f]+$/.test(value);
}

function audit(): number {
  const localeFiles = readdirSync(LOCALE_DIR).filter((f) => f.endsWith(".json"));
  if (!localeFiles.includes(`${REFERENCE}.json`)) {
    console.error(`reference locale ${REFERENCE}.json not found in ${LOCALE_DIR}`);
    return 1;
  }
  const reference = loadLocale(`${REFERENCE}.json`);
  const suspect: { locale: string; key: string; value: string }[] = [];
  const missing: { locale: string; key: string }[] = [];

  for (const file of localeFiles) {
    const locale = basename(file, ".json");
    if (locale === REFERENCE) continue;
    const data = loadLocale(file);

    for (const [key, refValue] of Object.entries(reference)) {
      if (typeof refValue !== "string") continue;
      if (!(key in data)) {
        missing.push({ locale, key });
        continue;
      }
      const value = data[key];
      if (typeof value !== "string" || value === "") continue;
      if (!isNormalEnglishSentence(refValue)) continue;
      if (!isPureAscii(value)) continue;
      if (isWhitelisted(key, value)) continue;
      suspect.push({ locale, key, value });
    }
  }

  if (missing.length > 0) {
    console.log(`Locale-parity gaps (informational, not gated):`);
    for (const m of missing) console.log(`  missing in ${m.locale}: ${m.key}`);
  }
  if (suspect.length === 0) {
    console.log(`audit-i18n: 0 suspect values across ${localeFiles.length - 1} locales`);
    return 0;
  }
  console.log(`audit-i18n: ${suspect.length} suspect value(s):`);
  for (const s of suspect) console.log(`  ${s.locale}: ${s.key} = ${s.value}`);
  return 1;
}

process.exit(audit());
