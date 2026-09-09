import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  deleteSession,
  getClients,
  getSessions,
  parseAppError,
  updateSession,
} from "../../services/tauri";
import { confirmAction } from "../../services/dialog";
import type { Client, TimeSession } from "../../types";
import {
  formatActualDuration,
  formatHoursMinutes,
  formatLocalDate,
  formatLocalTime,
  fromDateTimeLocal,
  toDateTimeLocal,
  todayDate,
} from "../../utils/format";

type EditorState = {
  id: number;
  clientId: number;
  startedAt: string;
  stoppedAt: string;
  note: string;
};

export function TimesheetView() {
  const [clients, setClients] = useState<Client[]>([]);
  const [sessions, setSessions] = useState<TimeSession[]>([]);
  const [clientId, setClientId] = useState<number | null>(null);
  const [fromDate, setFromDate] = useState(todayDate());
  const [toDate, setToDate] = useState(todayDate());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editor, setEditor] = useState<EditorState | null>(null);
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setError(null);
    try {
      const [clientList, sessionList] = await Promise.all([
        getClients(),
        getSessions(clientId, fromDate || null, toDate || null),
      ]);
      setClients(clientList);
      setSessions(sessionList);
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setLoading(false);
    }
  }, [clientId, fromDate, toDate]);

  useEffect(() => {
    void load();
  }, [load]);

  function startEdit(session: TimeSession) {
    setEditor({
      id: session.id,
      clientId: session.client_id,
      startedAt: toDateTimeLocal(session.started_at),
      stoppedAt: toDateTimeLocal(session.stopped_at),
      note: session.note ?? "",
    });
    setError(null);
  }

  async function onSave(event: FormEvent) {
    event.preventDefault();
    if (!editor) {
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await updateSession(
        editor.id,
        editor.clientId,
        fromDateTimeLocal(editor.startedAt),
        fromDateTimeLocal(editor.stoppedAt),
        editor.note.trim() ? editor.note.trim() : null,
      );
      setEditor(null);
      await load();
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setSaving(false);
    }
  }

  async function onDelete(session: TimeSession) {
    const confirmed = await confirmAction(
      `Delete the session for ${session.client_name}?`,
      "Delete",
    );
    if (!confirmed) {
      return;
    }
    setError(null);
    try {
      await deleteSession(session.id);
      if (editor?.id === session.id) {
        setEditor(null);
      }
      await load();
    } catch (cause) {
      setError(parseAppError(cause).message);
    }
  }

  return (
    <div className="timesheet-layout">
      {error ? <p className="form-error">{error}</p> : null}

      <div className="filter-row">
        <label>
          Client
          <select
            value={clientId ?? ""}
            onChange={(event) =>
              setClientId(event.target.value ? Number(event.target.value) : null)
            }
          >
            <option value="">All clients</option>
            {clients.map((client) => (
              <option key={client.id} value={client.id}>
                {client.name}
              </option>
            ))}
          </select>
        </label>
        <label>
          From
          <input
            type="date"
            value={fromDate}
            onChange={(event) => setFromDate(event.target.value)}
          />
        </label>
        <label>
          To
          <input type="date" value={toDate} onChange={(event) => setToDate(event.target.value)} />
        </label>
      </div>

      {loading ? (
        <p className="placeholder">Loading sessions…</p>
      ) : sessions.length === 0 ? (
        <p className="placeholder">No sessions in this range.</p>
      ) : (
        <table className="clients-table">
          <thead>
            <tr>
              <th>Date</th>
              <th>Client</th>
              <th>Start</th>
              <th>End</th>
              <th>Actual</th>
              <th>Rounded</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((session) => (
              <tr key={session.id}>
                <td>{formatLocalDate(session.started_at)}</td>
                <td>{session.client_name}</td>
                <td>{formatLocalTime(session.started_at)}</td>
                <td>{formatLocalTime(session.stopped_at)}</td>
                <td>{formatActualDuration(session.actual_duration_seconds)}</td>
                <td>{formatHoursMinutes(session.rounded_duration_minutes)}</td>
                <td className="client-actions">
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => startEdit(session)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    className="danger-button"
                    onClick={() => void onDelete(session)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {editor ? (
        <form className="client-form" onSubmit={(event) => void onSave(event)}>
          <h3>Edit Session</h3>
          <label>
            Client
            <select
              value={editor.clientId}
              onChange={(event) =>
                setEditor({ ...editor, clientId: Number(event.target.value) })
              }
            >
              {clients.map((client) => (
                <option key={client.id} value={client.id}>
                  {client.name}
                </option>
              ))}
            </select>
          </label>
          <label>
            Start
            <input
              type="datetime-local"
              value={editor.startedAt}
              onChange={(event) => setEditor({ ...editor, startedAt: event.target.value })}
              required
            />
          </label>
          <label>
            End
            <input
              type="datetime-local"
              value={editor.stoppedAt}
              onChange={(event) => setEditor({ ...editor, stoppedAt: event.target.value })}
              required
            />
          </label>
          <label>
            Note
            <input
              value={editor.note}
              onChange={(event) => setEditor({ ...editor, note: event.target.value })}
            />
          </label>
          <div className="form-actions">
            <button type="submit" disabled={saving}>
              Save Changes
            </button>
            <button type="button" className="secondary-button" onClick={() => setEditor(null)}>
              Cancel
            </button>
          </div>
        </form>
      ) : null}
    </div>
  );
}
