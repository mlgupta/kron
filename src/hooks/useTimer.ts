import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  getActiveTimer,
  getClients,
  parseAppError,
  startTimer,
  stopTimer,
} from "../services/tauri";
import { confirmAction } from "../services/dialog";
import type { ActiveTimer, Client, TimeSession } from "../types";

function elapsedSecondsFrom(startedAt: string): number {
  const started = Date.parse(startedAt);
  if (Number.isNaN(started)) {
    return 0;
  }
  return Math.max(0, Math.floor((Date.now() - started) / 1000));
}

export function useTimer() {
  const [clients, setClients] = useState<Client[]>([]);
  const [selectedClientId, setSelectedClientId] = useState<number | null>(null);
  const [active, setActive] = useState<ActiveTimer | null>(null);
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  const [lastSession, setLastSession] = useState<TimeSession | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectableClients = useMemo(() => {
    const activeClients = clients.filter((client) => client.active);
    if (active && !activeClients.some((client) => client.id === active.client_id)) {
      const current = clients.find((client) => client.id === active.client_id);
      return current ? [current, ...activeClients] : activeClients;
    }
    return activeClients;
  }, [active, clients]);

  const load = useCallback(async () => {
    setError(null);
    try {
      const [clientList, running] = await Promise.all([getClients(), getActiveTimer()]);
      setClients(clientList);
      setActive(running);
      if (running) {
        setSelectedClientId(running.client_id);
        setElapsedSeconds(elapsedSecondsFrom(running.started_at));
      } else {
        setElapsedSeconds(0);
        setSelectedClientId((current) => {
          if (current && clientList.some((client) => client.id === current && client.active)) {
            return current;
          }
          return clientList.find((client) => client.active)?.id ?? null;
        });
      }
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listen("timer-changed", () => {
      void load();
    }).then((fn) => {
      if (cancelled) {
        fn();
        return;
      }
      unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [load]);

  useEffect(() => {
    if (!active) {
      return;
    }

    const tick = () => setElapsedSeconds(elapsedSecondsFrom(active.started_at));
    tick();
    const interval = window.setInterval(tick, 1000);
    return () => window.clearInterval(interval);
  }, [active]);

  async function start() {
    if (selectedClientId === null) {
      setError("Select a client before starting the timer.");
      return;
    }

    if (active && active.client_id !== selectedClientId) {
      const confirmed = await confirmAction(
        `A timer is already running for ${active.client_name}. Stop it and start a new timer for the selected client?`,
        "Switch Client",
      );
      if (!confirmed) {
        return;
      }
    }

    setBusy(true);
    setError(null);
    try {
      const running = await startTimer(
        selectedClientId,
        Boolean(active && active.client_id !== selectedClientId),
      );
      setActive(running);
      setLastSession(null);
      setElapsedSeconds(elapsedSecondsFrom(running.started_at));
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setBusy(false);
    }
  }

  async function stop() {
    setBusy(true);
    setError(null);
    try {
      const session = await stopTimer();
      setLastSession(session);
      setActive(null);
      setElapsedSeconds(0);
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setBusy(false);
    }
  }

  return {
    clients: selectableClients,
    selectedClientId,
    setSelectedClientId,
    active,
    elapsedSeconds,
    lastSession,
    loading,
    busy,
    error,
    start,
    stop,
  };
}
