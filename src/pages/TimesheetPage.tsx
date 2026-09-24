import { TimesheetView } from "../components/Timesheet/TimesheetView";

export function TimesheetPage() {
  return (
    <section className="page">
      <header className="page-header">
        <h2>Timesheet</h2>
        <p>Review and adjust your recorded sessions.</p>
      </header>
      <TimesheetView />
    </section>
  );
}
