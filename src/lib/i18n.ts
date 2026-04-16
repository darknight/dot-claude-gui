/// <reference types="vite/client" />
import { appSettingsStore } from "$lib/stores/appsettings.svelte";
import zhCN from "./locales/zh-CN.json";
import enUS from "./locales/en-US.json";
import jaJP from "./locales/ja-JP.json";
import esES from "./locales/es-ES.json";
import frFR from "./locales/fr-FR.json";
import koKR from "./locales/ko-KR.json";

export const SUPPORTED_LOCALES = [
  "zh-CN", "en-US", "ja-JP", "es-ES", "fr-FR", "ko-KR",
] as const;
export type Locale = (typeof SUPPORTED_LOCALES)[number];
export const ACTIVE_LOCALES: Locale[] = ["zh-CN", "en-US", "ja-JP"];

export type MessageKey = keyof typeof zhCN;

// Main 3 languages are type-complete; extension slots allow any subset
// (empty string values fall back to en-US at runtime).
const bundles: {
  "zh-CN": Record<MessageKey, string>;
  "en-US": Record<MessageKey, string>;
  "ja-JP": Record<MessageKey, string>;
  "es-ES": Partial<Record<MessageKey, string>>;
  "fr-FR": Partial<Record<MessageKey, string>>;
  "ko-KR": Partial<Record<MessageKey, string>>;
} = {
  "zh-CN": zhCN,
  "en-US": enUS,
  "ja-JP": jaJP,
  "es-ES": esES,
  "fr-FR": frFR,
  "ko-KR": koKR,
};

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  const lang = (appSettingsStore.preferences.language as Locale) ?? "en-US";
  let text: string = bundles[lang]?.[key] || bundles["en-US"][key];
  if (!text) {
    if (import.meta.env.DEV) {
      console.warn(`[i18n] missing key: ${key}`);
    }
    return key;
  }
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, String(v));
    }
  }
  return text;
}

export function detectInitialLocale(): Locale {
  const nav = typeof navigator !== "undefined" ? navigator.language : "";
  if (nav.startsWith("zh")) return "zh-CN";
  if (nav.startsWith("ja")) return "ja-JP";
  return "en-US";
}

export function localeDisplayName(loc: Locale): string {
  return {
    "zh-CN": "简体中文",
    "en-US": "English",
    "ja-JP": "日本語",
    "es-ES": "Español",
    "fr-FR": "Français",
    "ko-KR": "한국어",
  }[loc];
}

export function isSupportedLocale(s: unknown): s is Locale {
  return typeof s === "string" && (SUPPORTED_LOCALES as readonly string[]).includes(s);
}
