import { useEffect, useState } from "react";
import { ClientsPage } from "./pages/ClientsPage";
import { ReportsPage } from "./pages/ReportsPage";
import { TimerPage } from "./pages/TimerPage";
import { TimesheetPage } from "./pages/TimesheetPage";
import { pingBackend } from "./services/tauri";
import type { BackendStatus, PageId } from "./types";

const pages: { id: PageId; label: string }[] = [
  { id: "timer", label: "Timer" },
  { id: "clients", label: "Clients" },
  { id: "timesheet", label: "Timesheet" },
  { id: "reports", label: "Reports" },
];

function App() {
  const [page, setPage] = useState<PageId>("timer");
  const [backendStatus, setBackendStatus] = useState<BackendStatus>("checking");

  useEffect(() => {
    let cancelled = false;

    pingBackend()
      .then((response) => {
        if (!cancelled) {
          setBackendStatus(response === "ok" ? "connected" : "unavailable");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setBackendStatus("unavailable");
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <h1>Kron</h1>
          <p className="subtitle">Personal time tracking for consultants</p>
        </div>
        <p className={`status status-${backendStatus}`}>
          Backend: {backendStatus}
        </p>
      </header>

      <nav className="app-nav" aria-label="Primary">
        {pages.map(({ id, label }) => (
          <button
            key={id}
            type="button"
            className={page === id ? "nav-button active" : "nav-button"}
            onClick={() => setPage(id)}
          >
            {label}
          </button>
        ))}
      </nav>

      <main className="app-main">
        {page === "timer" && <TimerPage />}
        {page === "clients" && <ClientsPage />}
        {page === "timesheet" && <TimesheetPage />}
        {page === "reports" && <ReportsPage />}
      </main>
    </div>
  );
}

export default App;
