import { useState } from "react";
import { authedInvoke, clearSessionToken } from "@/lib/api";
import "./App.css";
import { Setup } from "./components/Setup";
import { Login } from "./components/Login";
import { CompanySetup } from "./components/CompanySetup";
import { Dashboard } from "./components/Dashboard";
import { SplashScreen } from "./components/SplashScreen";
import { applyTheme } from "./lib/theme";
import { type Theme } from "./types/theme";

type Screen = "loading" | "setup" | "login" | "company-setup" | "dashboard";

export type AuthUser = { id: number; name: string; nickname: string };

function App() {
  const [screen, setScreen] = useState<Screen>("loading");
  const [user, setUser] = useState<AuthUser | null>(null);

  if (screen === "loading") {
    return (
      <SplashScreen
        onFinished={async () => {
          const hasUsers = await authedInvoke<boolean>("check_setup").catch(() => false);
          setScreen(hasUsers ? "login" : "setup");
        }}
      />
    );
  }

  if (screen === "setup") {
    return <Setup onComplete={() => setScreen("login")} />;
  }

  if (screen === "login") {
    return (
      <Login
        onSuccess={async (u) => {
          const saved = await authedInvoke<string | null>("get_user_setting", {
            userId: u.id,
            key: "theme",
          }).catch(() => null);
          applyTheme(
            (saved as Theme | null) ?? "system",
            window.matchMedia("(prefers-color-scheme: dark)").matches
          );
          setUser(u);
          const hasCompany = await authedInvoke<boolean>("check_business_setup").catch(() => true);
          setScreen(hasCompany ? "dashboard" : "company-setup");
        }}
      />
    );
  }

  if (screen === "company-setup") {
    return (
      <CompanySetup
        user={user!}
        onComplete={() => setScreen("dashboard")}
      />
    );
  }

  return (
    <Dashboard
      user={user!}
      onLogout={async () => {
        await authedInvoke("logout").catch(() => {});
        clearSessionToken();
        setUser(null);
        setScreen("login");
      }}
    />
  );
}

export default App;
