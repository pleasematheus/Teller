import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { HugeiconsIcon } from "@hugeicons/react";
import { ArrowLeft01Icon } from "@hugeicons/core-free-icons";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { type AuthUser } from "@/App";

type UserInfo = {
  id: number;
  name: string;
  nickname: string;
  active: boolean;
};

type ActiveView = "list" | "form-new" | "form-password";

type Props = {
  user: AuthUser;
};

const inputClass =
  "h-9 bg-teller-bg border-teller-border text-teller-text placeholder:text-teller-muted focus-visible:border-teller-focus focus-visible:ring-teller-focus/30";

export function UsuariosSection({ user }: Props) {
  const [view, setView] = useState<ActiveView>("list");
  const [users, setUsers] = useState<UserInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [passwordTarget, setPasswordTarget] = useState<UserInfo | null>(null);

  // New user form state
  const [newName, setNewName] = useState("");
  const [newNickname, setNewNickname] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [savingNew, setSavingNew] = useState(false);
  const [newError, setNewError] = useState<string | null>(null);

  // Change password form state
  const [newPw, setNewPw] = useState("");
  const [savingPw, setSavingPw] = useState(false);
  const [pwError, setPwError] = useState<string | null>(null);

  const loadUsers = useCallback(() => {
    invoke<UserInfo[]>("list_users")
      .then(setUsers)
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    loadUsers();
  }, [loadUsers]);

  const handleToggleActive = async (u: UserInfo) => {
    setError(null);
    try {
      await invoke("toggle_user_active", { userId: u.id, active: !u.active });
      setUsers((prev) =>
        prev.map((x) => (x.id === u.id ? { ...x, active: !x.active } : x))
      );
    } catch (err) {
      setError(typeof err === "string" ? err : "Erro ao alterar status");
    }
  };

  const handleOpenChangePassword = (u: UserInfo) => {
    setPasswordTarget(u);
    setNewPw("");
    setPwError(null);
    setView("form-password");
  };

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    setSavingNew(true);
    setNewError(null);
    try {
      await invoke("create_user", {
        name: newName.trim(),
        nickname: newNickname.trim(),
        password: newPassword,
      });
      setView("list");
      loadUsers();
    } catch (err) {
      setNewError(typeof err === "string" ? err : "Erro ao criar usuário");
    } finally {
      setSavingNew(false);
    }
  };

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!passwordTarget) return;
    setSavingPw(true);
    setPwError(null);
    try {
      await invoke("change_user_password", {
        userId: passwordTarget.id,
        password: newPw,
      });
      setView("list");
      setPasswordTarget(null);
    } catch (err) {
      setPwError(typeof err === "string" ? err : "Erro ao alterar senha");
    } finally {
      setSavingPw(false);
    }
  };

  // ── List view ────────────────────────────────────────────────────────────────

  if (view === "list") {
    return (
      <div className="max-w-lg">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-teller-text text-base font-semibold">Usuários</h2>
          <Button
            size="sm"
            onClick={() => {
              setNewName(""); setNewNickname(""); setNewPassword("");
              setNewError(null); setView("form-new");
            }}
            className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer"
          >
            + Novo usuário
          </Button>
        </div>

        {error && <p className="text-teller-danger text-xs mb-3">{error}</p>}

        {loading ? (
          <p className="text-teller-muted text-sm">Carregando...</p>
        ) : (
          <div className="flex flex-col divide-y divide-teller-border border border-teller-border rounded-lg overflow-hidden">
            {users.map((u) => (
              <div key={u.id} className="flex items-center gap-3 px-4 py-3 bg-teller-card">
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-teller-text font-medium truncate">{u.name}</p>
                  <p className="text-xs text-teller-muted">@{u.nickname}</p>
                </div>
                <span
                  className={`text-xs px-2 py-0.5 rounded-full ${
                    u.active
                      ? "bg-teller-accent/10 text-teller-accent"
                      : "bg-teller-border text-teller-muted"
                  }`}
                >
                  {u.active ? "ativo" : "inativo"}
                </span>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleOpenChangePassword(u)}
                    className="text-xs text-teller-secondary hover:text-teller-text cursor-pointer transition-colors"
                  >
                    Senha
                  </button>
                  {u.id !== user.id && (
                    <button
                      onClick={() => handleToggleActive(u)}
                      className="text-xs text-teller-secondary hover:text-teller-text cursor-pointer transition-colors"
                    >
                      {u.active ? "Desativar" : "Ativar"}
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    );
  }

  // ── New user form ────────────────────────────────────────────────────────────

  if (view === "form-new") {
    return (
      <div className="max-w-md">
        <button
          onClick={() => setView("list")}
          className="flex items-center gap-1.5 text-teller-secondary hover:text-teller-text text-xs mb-6 cursor-pointer transition-colors"
        >
          <HugeiconsIcon icon={ArrowLeft01Icon} strokeWidth={1.5} className="size-3.5" />
          Voltar
        </button>
        <h2 className="text-teller-text text-base font-semibold mb-6">Novo Usuário</h2>
        <form onSubmit={handleCreateUser} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <label className="text-teller-secondary text-xs font-medium">Nome</label>
            <Input
              autoFocus
              value={newName}
              onChange={(e) => { setNewName(e.target.value); setNewError(null); }}
              placeholder="Ex: João Silva"
              disabled={savingNew}
              className={inputClass}
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-teller-secondary text-xs font-medium">Apelido (login)</label>
            <Input
              value={newNickname}
              onChange={(e) => { setNewNickname(e.target.value); setNewError(null); }}
              placeholder="Ex: joao"
              disabled={savingNew}
              className={inputClass}
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-teller-secondary text-xs font-medium">Senha</label>
            <Input
              type="password"
              value={newPassword}
              onChange={(e) => { setNewPassword(e.target.value); setNewError(null); }}
              placeholder="••••••••"
              disabled={savingNew}
              className={inputClass}
            />
          </div>
          {newError && <p className="text-teller-danger text-xs">{newError}</p>}
          <div className="flex gap-2 justify-end mt-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setView("list")}
              className="h-8 px-3 border-teller-border text-teller-secondary hover:text-teller-text text-xs cursor-pointer"
            >
              Cancelar
            </Button>
            <Button
              type="submit"
              size="sm"
              disabled={savingNew}
              className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer disabled:opacity-50"
            >
              {savingNew ? "Criando..." : "Criar usuário"}
            </Button>
          </div>
        </form>
      </div>
    );
  }

  // ── Change password form ─────────────────────────────────────────────────────

  return (
    <div className="max-w-md">
      <button
        onClick={() => { setView("list"); setPasswordTarget(null); }}
        className="flex items-center gap-1.5 text-teller-secondary hover:text-teller-text text-xs mb-6 cursor-pointer transition-colors"
      >
        <HugeiconsIcon icon={ArrowLeft01Icon} strokeWidth={1.5} className="size-3.5" />
        Voltar
      </button>
      <h2 className="text-teller-text text-base font-semibold mb-1">Alterar senha</h2>
      <p className="text-teller-muted text-xs mb-6">
        {passwordTarget?.name} (@{passwordTarget?.nickname})
      </p>
      <form onSubmit={handleChangePassword} className="flex flex-col gap-4">
        <div className="flex flex-col gap-1.5">
          <label className="text-teller-secondary text-xs font-medium">Nova senha</label>
          <Input
            autoFocus
            type="password"
            value={newPw}
            onChange={(e) => { setNewPw(e.target.value); setPwError(null); }}
            placeholder="••••••••"
            disabled={savingPw}
            className={inputClass}
          />
        </div>
        {pwError && <p className="text-teller-danger text-xs">{pwError}</p>}
        <div className="flex gap-2 justify-end mt-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => { setView("list"); setPasswordTarget(null); }}
            className="h-8 px-3 border-teller-border text-teller-secondary hover:text-teller-text text-xs cursor-pointer"
          >
            Cancelar
          </Button>
          <Button
            type="submit"
            size="sm"
            disabled={savingPw}
            className="h-8 px-3 bg-teller-accent hover:bg-teller-accent-hover text-teller-bg text-xs font-semibold cursor-pointer disabled:opacity-50"
          >
            {savingPw ? "Salvando..." : "Salvar"}
          </Button>
        </div>
      </form>
    </div>
  );
}
