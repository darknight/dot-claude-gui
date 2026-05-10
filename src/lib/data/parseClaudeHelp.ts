// Parser for `claude --help` stdout. Commander/clap-style output:
//   -e, --effort <level>   Effort level for the model
//   --brief                Use brief output mode

export interface ClaudeArg {
  flag: string;
  short?: string;
  takesValue: boolean;
  valueHint?: string;
  description: string;
}

const FLAG_LINE = /^\s+(?:(-[\w])\s*,\s*)?(--[\w-]+)(?:[ =]+<([^>]+)>)?\s{2,}(.+?)$/;

export function parseClaudeHelp(stdout: string): ClaudeArg[] {
  const out: ClaudeArg[] = [];
  for (const line of stdout.split("\n")) {
    const m = line.match(FLAG_LINE);
    if (!m) continue;
    out.push({
      short: m[1],
      flag: m[2],
      takesValue: !!m[3],
      valueHint: m[3],
      description: m[4].trim(),
    });
  }
  return out;
}
