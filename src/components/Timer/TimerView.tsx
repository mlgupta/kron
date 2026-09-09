import { useTimer } from "../../hooks/useTimer";

function formatElapsed(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return [hours, minutes, seconds]
    .map((value) => String(value).padStart(2, "0"))
    .join(":");
}

function formatStartedAt(startedAt: string): string {
  const date = new Date(startedAt);
  if (Number.isNaN(date.getTime())) {
    return startedAt;
  }
  return date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}

export function TimerView() {
  const {
    clients,
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
  } = useTimer();

  if (loading) {
    return <p className="placeholder">Loading timer…</p>;
  }

  const running = active !== null;

  return (
    <div className="timer-layout">
      {error ? <p className="form-error">{error}</p> : null}

      {clients.length === 0 ? (
        <p className="placeholder">Add an active client before starting a timer.</p>
      ) : running ? (
        <p className="timer-client-name">{active.client_name}</p>
      ) : (
        <label className="timer-client-select">
          Client
          <select
            value={selectedClientId ?? ""}
            onChange={(event) => setSelectedClientId(Number(event.target.value))}
          >
            {clients.map((client) => (
              <option key={client.id} value={client.id}>
                {client.name}
              </option>
            ))}
          </select>
        </label>
      )}

      <p className="timer-elapsed" aria-live="polite">
        {formatElapsed(elapsedSeconds)}
      </p>

      <div className="timer-actions">
        {running ? (
          <button type="button" className="timer-stop" disabled={busy} onClick={() => void stop()}>
            Stop
          </button>
        ) : (
          <button
            type="button"
            className="timer-start"
            disabled={busy || clients.length === 0}
            onClick={() => void start()}
          >
            Start
          </button>
        )}
      </div>

      {running ? (
        <p className="timer-started">Started: {formatStartedAt(active.started_at)}</p>
      ) : null}

      {running && clients.length > 1 ? (
        <label className="timer-client-select">
          Switch client
          <select
            value={selectedClientId ?? ""}
            disabled={busy}
            onChange={(event) => setSelectedClientId(Number(event.target.value))}
          >
            {clients.map((client) => (
              <option key={client.id} value={client.id}>
                {client.name}
              </option>
            ))}
          </select>
        </label>
      ) : null}

      {running && selectedClientId !== active.client_id ? (
        <button
          type="button"
          className="secondary-button"
          disabled={busy}
          onClick={() => void start()}
        >
          Switch and start
        </button>
      ) : null}

      {lastSession ? (
        <p className="timer-session">
          Last session: {lastSession.client_name} ·{" "}
          {Math.round(lastSession.actual_duration_seconds / 60)}m actual ·{" "}
          {lastSession.rounded_duration_minutes}m billed
        </p>
      ) : null}
    </div>
  );
}
