export function parseSystemSettingBool(
  value: string | null | undefined,
  defaultValue = false,
): boolean {
  if (value === "true") return true;
  if (value === "false") return false;
  return defaultValue;
}
