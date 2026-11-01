import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/api", () => ({ authedInvoke: vi.fn() }));

import { authedInvoke } from "@/lib/api";
import { getInitialScreen } from "@/lib/startup";

const invoke = vi.mocked(authedInvoke);

beforeEach(() => {
  invoke.mockReset();
});

describe("inicialização", () => {
  it("bloqueia o cadastro quando o banco não pode ser aberto", async () => {
    invoke.mockResolvedValueOnce("Banco indisponível: preserve o WAL");

    await expect(getInitialScreen()).rejects.toThrow("preserve o WAL");
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).not.toHaveBeenCalledWith("check_setup");
  });

  it("encaminha bancos com usuários para o login", async () => {
    invoke.mockResolvedValueOnce(null).mockResolvedValueOnce(true);

    await expect(getInitialScreen()).resolves.toBe("login");
  });

  it("encaminha um banco novo para o cadastro", async () => {
    invoke.mockResolvedValueOnce(null).mockResolvedValueOnce(false);

    await expect(getInitialScreen()).resolves.toBe("setup");
  });

  it("propaga falhas da consulta de usuários sem tratá-las como banco vazio", async () => {
    invoke.mockResolvedValueOnce(null).mockRejectedValueOnce(new Error("Falha na consulta"));

    await expect(getInitialScreen()).rejects.toThrow("Falha na consulta");
  });

  it("interrompe a inicialização se não conseguir consultar seu estado", async () => {
    invoke.mockRejectedValueOnce(new Error("Falha de comunicação"));

    await expect(getInitialScreen()).rejects.toThrow("Falha de comunicação");
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
