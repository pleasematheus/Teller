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
import { getInitialScreen } from "./lib/startup";

type Screen = "loading" | "setup" | "login" | "company-setup" | "dashboard";

export type AuthUser = { id: number; name: string; nickname: string };

function App() {
  const [screen, setScreen] = useState<Screen>("loading");
  const [user, setUser] = useState<AuthUser | null>(null);
  const [startupError, setStartupError] = useState<string | null>(null);

  if (startupError) {
    return (
      <main className="min-h-screen bg-teller-bg flex items-center justify-center p-6">
        <div role="alert" className="w-full max-w-xl space-y-4 text-teller-text">
          <h1 className="text-xl font-semibold">Não foi possível abrir o Teller</h1>
          <p className="whitespace-pre-wrap break-words text-sm">{startupError}</p>
        </div>
      </main>
    );
  }

  if (screen === "loading") {
    return (
      <SplashScreen
        onFinished={async () => {
          try {
            setScreen(await getInitialScreen());
          } catch (error) {
            setStartupError(
              error instanceof Error ? error.message : String(error)
            );
          }
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
