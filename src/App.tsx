import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Setup } from "./components/Setup";
import { Login } from "./components/Login";
import { CompanySetup } from "./components/CompanySetup";
import { Dashboard } from "./components/Dashboard";

type Screen = "loading" | "setup" | "login" | "company-setup" | "dashboard";

export type AuthUser = { id: number; name: string; nickname: string };

function App() {
  const [screen, setScreen] = useState<Screen>("loading");
  const [user, setUser] = useState<AuthUser | null>(null);

  useEffect(() => {
    invoke<boolean>("check_setup")
      .then((hasUsers) => setScreen(hasUsers ? "login" : "setup"))
      .catch(() => setScreen("setup"));
  }, []);

  if (screen === "loading") {
    return (
      <div className="min-h-screen bg-teller-bg flex items-center justify-center">
        <span className="text-teller-secondary text-sm">Carregando...</span>
      </div>
    );
  }

  if (screen === "setup") {
    return <Setup onComplete={() => setScreen("login")} />;
  }

  if (screen === "login") {
    return (
      <Login
        onSuccess={async (u) => {
          setUser(u);
          const hasCompany = await invoke<boolean>("check_business_setup").catch(() => true);
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
      onLogout={() => {
        setUser(null);
        setScreen("login");
      }}
    />
  );
}

export default App;
