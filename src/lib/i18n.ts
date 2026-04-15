import { appSettingsStore } from "$lib/stores/appsettings.svelte";

type Locale = "zh-CN" | "en-US";

const translations: Record<string, Record<Locale, string>> = {
  "memory.noFilesYet": {
    "zh-CN": "当前项目 {name} 还没有生成记忆文件。",
    "en-US": "This project {name} has no memory files yet.",
  },
  "memory.dropdownDisabledInProjectScope": {
    "zh-CN": "项目 scope 下由顶部 scope 选择器决定，切换到 User Scope 可手动选择。",
    "en-US": "In project scope, memory follows the top-right scope selector. Switch to User Scope to pick manually.",
  },
};

/**
 * Translate a key based on the user's current language preference.
 * Falls back to en-US if the current locale's translation is missing.
 * Placeholders in the form `{name}` are replaced from `params`.
 */
export function t(key: string, params?: Record<string, string>): string {
  const lang = (appSettingsStore.preferences.language as Locale) || "en-US";
  const entry = translations[key];
  if (!entry) return key;
  let text = entry[lang] ?? entry["en-US"] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, v);
    }
  }
  return text;
}
