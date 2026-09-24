import { FormEvent, useEffect, useState } from "react";
import {
  createClient,
  deleteClient,
  getClients,
  parseAppError,
  setClientActive,
  updateClient,
} from "../../services/tauri";
import { confirmAction } from "../../services/dialog";
import type { Client } from "../../types";

type EditorMode = "create" | "edit";

type EditorState = {
  mode: EditorMode;
  clientId: number | null;
  name: string;
  description: string;
};

const emptyEditor: EditorState = {
  mode: "create",
  clientId: null,
  name: "",
  description: "",
};

export function ClientsList() {
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [editor, setEditor] = useState<EditorState>(emptyEditor);
  const [saving, setSaving] = useState(false);

  async function loadClients() {
    setError(null);
    try {
      setClients(await getClients());
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadClients();
  }, []);

  function startCreate() {
    setEditor(emptyEditor);
    setError(null);
  }

  function startEdit(client: Client) {
    setEditor({
      mode: "edit",
      clientId: client.id,
      name: client.name,
      description: client.description ?? "",
    });
    setError(null);
  }

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    const description = editor.description.trim() ? editor.description.trim() : null;

    try {
      if (editor.mode === "create") {
        await createClient(editor.name, description);
      } else if (editor.clientId !== null) {
        await updateClient(editor.clientId, editor.name, description);
      }
      setEditor(emptyEditor);
      await loadClients();
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setSaving(false);
    }
  }

  async function onToggleActive(client: Client) {
    setBusyId(client.id);
    setError(null);
    try {
      await setClientActive(client.id, !client.active);
      await loadClients();
    } catch (cause) {
      setError(parseAppError(cause).message);
    } finally {
      setBusyId(null);
    }
  }

  async function onDelete(client: Client) {
    const confirmed = await confirmAction(
      `Delete ${client.name}? If this client has time sessions, you will be asked to confirm again.`,
      "Delete",
    );
    if (!confirmed) {
      return;
    }

    setBusyId(client.id);
    setError(null);
    try {
      await deleteClient(client.id, false);
      if (editor.clientId === client.id) {
        setEditor(emptyEditor);
      }
      await loadClients();
    } catch (cause) {
      const parsed = parseAppError(cause);
      if (parsed.code === "CLIENT_HAS_SESSIONS") {
        const force = await confirmAction(
          `${parsed.message}\n\nDelete ${client.name} and its recorded sessions anyway? Prefer deactivating the client if you want to keep history.`,
          "Delete Anyway",
        );
        if (force) {
          try {
            await deleteClient(client.id, true);
            if (editor.clientId === client.id) {
              setEditor(emptyEditor);
            }
            await loadClients();
          } catch (forceCause) {
            setError(parseAppError(forceCause).message);
          }
        }
      } else {
        setError(parsed.message);
      }
    } finally {
      setBusyId(null);
    }
  }

  return (
    <div className="clients-layout">
      {error ? <p className="form-error" role="alert">{error}</p> : null}

      {loading ? (
        <p className="placeholder">Loading clients…</p>
      ) : clients.length === 0 ? (
        <p className="placeholder">No clients yet. Add your first client below.</p>
      ) : (
        <table className="clients-table">
          <thead>
            <tr>
              <th>Client Name</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {clients.map((client) => (
              <tr key={client.id}>
                <td>
                  <div className="client-name">{client.name}</div>
                  {client.description ? (
                    <div className="client-description">{client.description}</div>
                  ) : null}
                </td>
                <td>{client.active ? "Active" : "Inactive"}</td>
                <td className="client-actions">
                  <button
                    type="button"
                    className="secondary-button"
                    disabled={busyId === client.id}
                    onClick={() => startEdit(client)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    className="secondary-button"
                    disabled={busyId === client.id}
                    onClick={() => void onToggleActive(client)}
                  >
                    {client.active ? "Deactivate" : "Activate"}
                  </button>
                  <button
                    type="button"
                    className="danger-button"
                    disabled={busyId === client.id}
                    onClick={() => void onDelete(client)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <form className="client-form" onSubmit={(event) => void onSubmit(event)}>
        <h3>{editor.mode === "create" ? "Add Client" : "Edit Client"}</h3>
        <label>
          Name
          <input
            value={editor.name}
            onChange={(event) =>
              setEditor((current) => ({ ...current, name: event.target.value }))
            }
            required
            maxLength={120}
            autoComplete="off"
          />
        </label>
        <label>
          Description
          <textarea
            value={editor.description}
            onChange={(event) =>
              setEditor((current) => ({
                ...current,
                description: event.target.value,
              }))
            }
            rows={3}
          />
        </label>
        <div className="form-actions">
          <button type="submit" disabled={saving}>
            {editor.mode === "create" ? "Add Client" : "Save Changes"}
          </button>
          {editor.mode === "edit" ? (
            <button type="button" className="secondary-button" onClick={startCreate}>
              Cancel
            </button>
          ) : null}
        </div>
      </form>
    </div>
  );
}
