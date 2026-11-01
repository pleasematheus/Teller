import { invoke } from "@tauri-apps/api/core";

let _sessionToken: string | null = null;

const PUBLIC_COMMANDS = new Set([
  "get_startup_error",
  "check_setup",
  "check_business_setup",
  "find_user",
  "login",
  "logout",
]);

export function setSessionToken(token: string): void {
  _sessionToken = token;
}

export function clearSessionToken(): void {
  _sessionToken = null;
}

export function authedInvoke<T>(
  cmd: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  if (PUBLIC_COMMANDS.has(cmd)) {
    return invoke<T>(cmd, args);
  }
  return invoke<T>(cmd, { ...args, sessionToken: _sessionToken ?? "" });
}
