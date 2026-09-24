import { ReportsView } from "../components/Reports/ReportsView";

export function ReportsPage() {
  return (
    <section className="page">
      <header className="page-header">
        <h2>Reports</h2>
        <p>See rounded billable time by client and period.</p>
      </header>
      <ReportsView />
    </section>
  );
}
