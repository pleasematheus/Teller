import { type Theme } from "@/types/theme";

export function resolveTheme(theme: Theme, systemIsDark: boolean): "light" | "dark" {
  if (theme === "system") return systemIsDark ? "dark" : "light";
  return theme;
}

export function applyTheme(theme: Theme, systemIsDark: boolean): void {
  const resolved = resolveTheme(theme, systemIsDark);
  document.documentElement.classList.toggle("dark", resolved === "dark");
}
