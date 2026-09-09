export type PageId = "timer" | "clients" | "timesheet" | "reports";

export type BackendStatus = "checking" | "connected" | "unavailable";

export type Client = {
  id: number;
  name: string;
  description: string | null;
  active: boolean;
  created_at: string;
  updated_at: string;
};

export type ActiveTimer = {
  client_id: number;
  client_name: string;
  started_at: string;
  note: string | null;
};

export type TimeSession = {
  id: number;
  client_id: number;
  client_name: string;
  started_at: string;
  stopped_at: string;
  actual_duration_seconds: number;
  rounded_duration_minutes: number;
  note: string | null;
  created_at: string;
};

export type ReportPeriod = "day" | "week" | "month";

export type ReportRow = {
  client_id: number;
  client_name: string;
  rounded_minutes: number;
  daily_minutes: number[];
  short_name: string;
  color: string;
};

export type Report = {
  period: ReportPeriod;
  label: string;
  start_date: string;
  end_date: string;
  previous_date: string;
  next_date: string;
  rows: ReportRow[];
  total_rounded_minutes: number;
  columns: string[];
};

export type AppError = {
  code: string;
  message: string;
};
