import { useEffect, useState } from "react";
import {
  getDailyTotals,
  getMonthlyTotals,
  getWeeklyTotals,
  parseAppError,
} from "../../services/tauri";
import type { Report, ReportPeriod, ReportRow } from "../../types";
import { formatHoursClock, todayDate } from "../../utils/format";

const periods: { id: ReportPeriod; label: string }[] = [
  { id: "day", label: "Day" },
  { id: "week", label: "Week" },
  { id: "month", label: "Month" },
];

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

async function loadReport(period: ReportPeriod, date: string): Promise<Report> {
  if (period === "week") {
    return getWeeklyTotals(date);
  }
  if (period === "month") {
    return getMonthlyTotals(date);
  }
  return getDailyTotals(date);
}

function formatCell(minutes: number): string {
  if (minutes <= 0) {
    return "—";
  }
  return formatHoursClock(minutes);
}

function columnTotals(report: Report): number[] {
  return report.columns.map((_, index) =>
    report.rows.reduce((sum, row) => sum + (row.daily_minutes[index] ?? 0), 0),
  );
}

function weekdayIndex(isoDate: string): number {
  return new Date(`${isoDate}T12:00:00`).getDay();
}

function ClientLegend({ rows }: { rows: ReportRow[] }) {
  if (rows.length === 0) {
    return null;
  }
  return (
    <ul className="client-legend">
      {rows.map((row) => (
        <li key={row.client_id}>
          <span className="client-swatch" style={{ background: row.color }} />
          <span className="client-legend-short">{row.short_name}</span>
          <span>{row.client_name}</span>
          <span className="client-legend-total">{formatHoursClock(row.rounded_minutes)}</span>
        </li>
      ))}
    </ul>
  );
}

function MonthlyCalendar({ report }: { report: Report }) {
  const lead = weekdayIndex(report.start_date);
  const cells: Array<{ day: number; index: number } | null> = [
    ...Array.from({ length: lead }, () => null),
    ...report.columns.map((_, index) => ({ day: index + 1, index })),
  ];
  while (cells.length % 7 !== 0) {
    cells.push(null);
  }

  return (
    <div className="month-calendar-wrap">
      <ClientLegend rows={report.rows} />
      <div className="month-calendar" role="grid" aria-label="Monthly time calendar">
        {WEEKDAYS.map((day) => (
          <div key={day} className="month-calendar-weekday" role="columnheader">
            {day}
          </div>
        ))}
        {cells.map((cell, index) => (
          <div
            key={index}
            className={cell ? "month-calendar-day" : "month-calendar-day empty"}
            role="gridcell"
          >
            {cell ? (
              <>
                <div className="month-calendar-date">{cell.day}</div>
                <div className="month-calendar-chips">
                  {report.rows
                    .filter((row) => (row.daily_minutes[cell.index] ?? 0) > 0)
                    .map((row) => (
                      <span
                        key={row.client_id}
                        className="client-chip"
                        style={{ background: row.color }}
                        title={`${row.client_name} ${formatHoursClock(row.daily_minutes[cell.index] ?? 0)}`}
                      >
                        {row.short_name} {formatHoursClock(row.daily_minutes[cell.index] ?? 0)}
                      </span>
                    ))}
                </div>
              </>
            ) : null}
          </div>
        ))}
      </div>
      <p className="report-month-total">
        Total {formatHoursClock(report.total_rounded_minutes)}
      </p>
    </div>
  );
}

function ClientLabel({ row }: { row: ReportRow }) {
  return (
    <span className="client-label">
      <span className="client-swatch" style={{ background: row.color }} />
      <span className="client-legend-short">{row.short_name}</span>
      {row.client_name}
    </span>
  );
}

export function ReportsView() {
  const [period, setPeriod] = useState<ReportPeriod>("day");
  const [date, setDate] = useState(todayDate());
  const [report, setReport] = useState<Report | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    loadReport(period, date)
      .then((next) => {
        if (!cancelled) {
          setReport(next);
        }
      })
      .catch((cause) => {
        if (!cancelled) {
          setError(parseAppError(cause).message);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [period, date]);

  const weekly = Boolean(report && report.period === "week" && report.columns.length === 7);
  const monthly = Boolean(report && report.period === "month" && report.columns.length >= 28);
  const totals = report && weekly ? columnTotals(report) : [];

  return (
    <div className="reports-layout">
      {error ? <p className="form-error" role="alert">{error}</p> : null}

      <div className="report-periods" role="tablist" aria-label="Report period">
        {periods.map((item) => (
          <button
            key={item.id}
            type="button"
            role="tab"
            aria-selected={period === item.id}
            className={period === item.id ? "nav-button active" : "nav-button"}
            onClick={() => setPeriod(item.id)}
          >
            {item.label}
          </button>
        ))}
      </div>

      <div className="report-nav">
        <button
          type="button"
          className="secondary-button"
          disabled={!report}
          onClick={() => report && setDate(report.previous_date)}
        >
          &lt;
        </button>
        <h3 className="report-label">{report?.label ?? "Loading…"}</h3>
        <button
          type="button"
          className="secondary-button"
          disabled={!report}
          onClick={() => report && setDate(report.next_date)}
        >
          &gt;
        </button>
      </div>

      {loading || !report ? (
        <p className="placeholder">Loading report…</p>
      ) : monthly ? (
        <MonthlyCalendar report={report} />
      ) : report.rows.length === 0 ? (
        <p className="placeholder">No rounded time recorded in this period.</p>
      ) : weekly ? (
        <div className="weekly-report-scroll">
          <table className="clients-table weekly-report">
            <thead>
              <tr>
                <th>Client</th>
                {report.columns.map((column) => (
                  <th key={column}>{column}</th>
                ))}
                <th>Total</th>
              </tr>
            </thead>
            <tbody>
              {report.rows.map((row) => (
                <tr key={row.client_id}>
                  <td>
                    <ClientLabel row={row} />
                  </td>
                  {report.columns.map((_, index) => (
                    <td key={index} className="report-hours">
                      {formatCell(row.daily_minutes[index] ?? 0)}
                    </td>
                  ))}
                  <td className="report-hours">{formatHoursClock(row.rounded_minutes)}</td>
                </tr>
              ))}
              <tr className="report-total">
                <td>Total</td>
                {totals.map((minutes, index) => (
                  <td key={report.columns[index]} className="report-hours">
                    {formatCell(minutes)}
                  </td>
                ))}
                <td className="report-hours">{formatHoursClock(report.total_rounded_minutes)}</td>
              </tr>
            </tbody>
          </table>
        </div>
      ) : (
        <table className="clients-table">
          <thead>
            <tr>
              <th>Client</th>
              <th>Time</th>
            </tr>
          </thead>
          <tbody>
            {report.rows.map((row) => (
              <tr key={row.client_id}>
                <td>
                  <ClientLabel row={row} />
                </td>
                <td>{formatHoursClock(row.rounded_minutes)}</td>
              </tr>
            ))}
            <tr className="report-total">
              <td>Total</td>
              <td>{formatHoursClock(report.total_rounded_minutes)}</td>
            </tr>
          </tbody>
        </table>
      )}
    </div>
  );
}
