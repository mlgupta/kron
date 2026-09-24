import { useEffect, useState } from "react";
import { BrandMark, PageIcon } from "./components/AppIcons";
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
        <div className="app-identity">
          <BrandMark />
          <div>
            <h1>Kron</h1>
            <p className="subtitle">Time tracking</p>
          </div>
        </div>
        <p
          className={`status status-${backendStatus}`}
          role="status"
          aria-live="polite"
        >
          <span className="status-dot" />
          {backendStatus === "connected"
            ? "Backend connected"
            : backendStatus === "checking"
              ? "Connecting…"
              : "Backend unavailable"}
        </p>
      </header>

      <nav className="app-nav" aria-label="Primary" role="tablist">
        {pages.map(({ id, label }) => (
          <button
            key={id}
            type="button"
            role="tab"
            aria-selected={page === id}
            aria-controls={`page-${id}`}
            className={page === id ? "nav-button active" : "nav-button"}
            onClick={() => setPage(id)}
          >
            <PageIcon page={id} />
            <span>{label}</span>
          </button>
        ))}
      </nav>

      <main className="app-main" id={`page-${page}`} role="tabpanel">
        {page === "timer" && <TimerPage />}
        {page === "clients" && <ClientsPage />}
        {page === "timesheet" && <TimesheetPage />}
        {page === "reports" && <ReportsPage />}
      </main>
    </div>
  );
}

export default App;
