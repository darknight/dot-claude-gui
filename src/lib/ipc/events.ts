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

/**
 * Run a streaming subprocess IPC and route its output / completion events.
 *
 * Race-safe contract: listeners are attached BEFORE `startCommand()` runs, so
 * even fast-failing commands (where stdout/stderr/exit fire microseconds
 * after spawn) won't drop events. The helper buffers events that arrive
 * before the requestId is known and replays the matching ones once it is.
 *
 * - `startCommand` is the IPC call that spawns the process and returns the
 *   `{ requestId }` correlation id (or undefined / throws on IPC failure).
 * - `onLine` is called for each stdout/stderr line.
 * - `onComplete` is called exactly once with the exit code.
 *
 * Returns `false` if `startCommand` failed to produce a requestId (in which
 * case `onComplete` is NOT called and the caller should surface the IPC
 * error itself); `true` if the stream was attached and `onComplete` will
 * fire when the process exits.
 */
export async function runStreamingCommand(
  startCommand: () => Promise<{ requestId: string } | undefined>,
  onLine: (line: string) => void,
  onComplete: (exitCode: number) => void | Promise<void>,
): Promise<boolean> {
  let myReqId: string | null = null;
  const earlyOutput: CommandOutputPayload[] = [];
  // Boxed so TS doesn't narrow it to `null` after closure mutation —
  // `let earlyComplete: ... | null = null` gets collapsed in callers.
  const earlyComplete: { value: CommandCompletedPayload | null } = { value: null };
  let finished = false;

  const finish = async (c: CommandCompletedPayload) => {
    if (finished) return;
    finished = true;
    unlistenOutput();
    unlistenCompleted();
    await onComplete(c.exitCode);
  };

  const unlistenOutput = await onCommandOutput((o) => {
    if (myReqId === null) {
      earlyOutput.push(o);
    } else if (o.commandId === myReqId) {
      onLine(o.line);
    }
  });
  const unlistenCompleted = await onCommandCompleted(async (c) => {
    if (myReqId === null) {
      earlyComplete.value = c;
      return;
    }
    if (c.commandId !== myReqId) return;
    await finish(c);
  });

  let result: { requestId: string } | undefined;
  try {
    result = await startCommand();
  } catch {
    unlistenOutput();
    unlistenCompleted();
    return false;
  }
  if (!result?.requestId) {
    unlistenOutput();
    unlistenCompleted();
    return false;
  }
  myReqId = result.requestId;

  // Drain anything that arrived before we knew the requestId.
  for (const o of earlyOutput) {
    if (o.commandId === myReqId) onLine(o.line);
  }
  if (earlyComplete.value && earlyComplete.value.commandId === myReqId) {
    await finish(earlyComplete.value);
  }
  return true;
}


