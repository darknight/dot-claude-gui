// src/lib/ipc/events.ts

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Settings, WsValidationError } from "$lib/api/types.js";

export interface ConfigChangedPayload {
  settings: Settings;
  source?: string;
}

export interface ValidationErrorPayload {
  errors: WsValidationError[];
}

export interface CommandOutputPayload {
  commandId: string;
  line: string;
  stream: "stdout" | "stderr";
}

export interface CommandCompletedPayload {
  commandId: string;
  exitCode: number;
}

/**
 * Subscribe to the `config-changed` event emitted by the backend file watcher
 * when a settings file changes on disk. Returns a function to unsubscribe.
 */
export function onConfigChanged(
  handler: (p: ConfigChangedPayload) => void,
): Promise<UnlistenFn> {
  return listen<ConfigChangedPayload>("config-changed", (e) => handler(e.payload));
}

/**
 * Subscribe to the `validation-error` event emitted when settings file parse fails.
 */
export function onValidationError(
  handler: (p: ValidationErrorPayload) => void,
): Promise<UnlistenFn> {
  return listen<ValidationErrorPayload>("validation-error", (e) => handler(e.payload));
}

/**
 * Subscribe to the `command-output` event emitted once per stdout/stderr line
 * from a subprocess spawned via the executor (plugin install, mcp add, etc.).
 */
export function onCommandOutput(
  handler: (p: CommandOutputPayload) => void,
): Promise<UnlistenFn> {
  return listen<CommandOutputPayload>("command-output", (e) => handler(e.payload));
}

/**
 * Subscribe to the `command-completed` event emitted when a subprocess exits.
 */
export function onCommandCompleted(
  handler: (p: CommandCompletedPayload) => void,
): Promise<UnlistenFn> {
  return listen<CommandCompletedPayload>("command-completed", (e) => handler(e.payload));
}
