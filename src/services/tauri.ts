import { invoke } from "@tauri-apps/api/core";
import type { ActiveTimer, AppError, Client, Report, TimeSession } from "../types";

export async function pingBackend(): Promise<string> {
  return invoke<string>("ping");
}

export async function getClients(): Promise<Client[]> {
  return invoke<Client[]>("get_clients");
}

export async function createClient(
  name: string,
  description: string | null,
): Promise<Client> {
  return invoke<Client>("create_client", { name, description });
}

export async function updateClient(
  id: number,
  name: string,
  description: string | null,
): Promise<Client> {
  return invoke<Client>("update_client", { id, name, description });
}

export async function setClientActive(
  id: number,
  active: boolean,
): Promise<Client> {
  return invoke<Client>("set_client_active", { id, active });
}

export async function deleteClient(id: number, force = false): Promise<void> {
  return invoke<void>("delete_client", { id, force });
}

export async function startTimer(
  clientId: number,
  switchClient = false,
): Promise<ActiveTimer> {
  return invoke<ActiveTimer>("start_timer", {
    clientId,
    switchClient,
  });
}

export async function stopTimer(): Promise<TimeSession> {
  return invoke<TimeSession>("stop_timer");
}

export async function getActiveTimer(): Promise<ActiveTimer | null> {
  return invoke<ActiveTimer | null>("get_active_timer");
}

export async function isTimerRunning(): Promise<boolean> {
  return invoke<boolean>("is_timer_running");
}

export async function getSessions(
  clientId: number | null,
  fromDate: string | null,
  toDate: string | null,
): Promise<TimeSession[]> {
  return invoke<TimeSession[]>("get_sessions", { clientId, fromDate, toDate });
}

export async function updateSession(
  id: number,
  clientId: number,
  startedAt: string,
  stoppedAt: string,
  note: string | null,
): Promise<TimeSession> {
  return invoke<TimeSession>("update_session", {
    id,
    clientId,
    startedAt,
    stoppedAt,
    note,
  });
}

export async function deleteSession(id: number): Promise<void> {
  return invoke<void>("delete_session", { id });
}

export async function getDailyTotals(date: string): Promise<Report> {
  return invoke<Report>("get_daily_totals", { date });
}

export async function getWeeklyTotals(date: string): Promise<Report> {
  return invoke<Report>("get_weekly_totals", { date });
}

export async function getMonthlyTotals(date: string): Promise<Report> {
  return invoke<Report>("get_monthly_totals", { date });
}

export function parseAppError(error: unknown): AppError {
  const raw = typeof error === "string" ? error : String(error);
  try {
    const parsed = JSON.parse(raw) as Partial<AppError>;
    if (parsed && typeof parsed.message === "string") {
      return {
        code: parsed.code ?? "UNKNOWN",
        message: parsed.message,
      };
    }
  } catch {
    // The backend may return a plain string.
  }
  return { code: "UNKNOWN", message: raw };
}
