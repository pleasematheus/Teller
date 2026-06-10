import { useState, useEffect } from "react";
import { authedInvoke } from "@/lib/api";
import { type Theme } from "@/types/theme";
import { applyTheme } from "@/lib/theme";
import { type AuthUser } from "@/App";

export function useTheme(user: AuthUser) {
  const [theme, setThemeState] = useState<Theme>("system");

  useEffect(() => {
    authedInvoke<string | null>("get_user_setting", { userId: user.id, key: "theme" }).then(
      (saved) => {
        const t = (saved as Theme | null) ?? "system";
        setThemeState(t);
        applyTheme(t, window.matchMedia("(prefers-color-scheme: dark)").matches);
      }
    );
  }, [user.id]);

  useEffect(() => {
    if (theme !== "system") return;

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => applyTheme("system", e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  const setTheme = (t: Theme) => {
    setThemeState(t);
    applyTheme(t, window.matchMedia("(prefers-color-scheme: dark)").matches);
    authedInvoke("set_user_setting", { userId: user.id, key: "theme", value: t });
  };

  return { theme, setTheme };
}
