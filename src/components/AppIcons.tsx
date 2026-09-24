import type { SVGProps } from "react";
import type { PageId } from "../types";

type IconProps = SVGProps<SVGSVGElement>;

const commonProps = {
  width: 18,
  height: 18,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.8,
  strokeLinecap: "round",
  strokeLinejoin: "round",
  "aria-hidden": true,
} as const;

export function PageIcon({ page, ...props }: IconProps & { page: PageId }) {
  if (page === "clients") {
    return (
      <svg {...commonProps} {...props}>
        <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
    );
  }

  if (page === "timesheet") {
    return (
      <svg {...commonProps} {...props}>
        <rect x="3" y="5" width="18" height="16" rx="2" />
        <path d="M16 3v4M8 3v4M3 10h18M8 14h.01M12 14h.01M16 14h.01M8 18h.01M12 18h.01" />
      </svg>
    );
  }

  if (page === "reports") {
    return (
      <svg {...commonProps} {...props}>
        <path d="M4 20V10M10 20V4M16 20v-7M22 20H2" />
        <rect x="3" y="10" width="2" height="10" rx="1" />
        <rect x="9" y="4" width="2" height="16" rx="1" />
        <rect x="15" y="13" width="2" height="7" rx="1" />
      </svg>
    );
  }

  return (
    <svg {...commonProps} {...props}>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </svg>
  );
}

export function BrandMark() {
  return (
    <span className="brand-mark" aria-hidden="true">
      <svg viewBox="0 0 40 40">
        <defs>
          <linearGradient id="brand-gradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="#3478f6" />
            <stop offset="1" stopColor="#6f56e8" />
          </linearGradient>
        </defs>
        <rect x="3" y="3" width="34" height="34" rx="10" fill="url(#brand-gradient)" />
        <circle cx="20" cy="21" r="11.5" fill="none" stroke="white" strokeWidth="2.2" />
        <path d="M20 11.5v-3M16.5 7.5h7" stroke="white" strokeWidth="2.2" strokeLinecap="round" />
        <path d="M20 21v-7M20 21l5 3" stroke="white" strokeWidth="2.2" strokeLinecap="round" />
      </svg>
    </span>
  );
}

export function StopIcon(props: IconProps) {
  return (
    <svg {...commonProps} {...props}>
      <rect x="7" y="7" width="10" height="10" rx="1.5" fill="currentColor" stroke="none" />
    </svg>
  );
}

export function PlayIcon(props: IconProps) {
  return (
    <svg {...commonProps} {...props}>
      <path d="m8 5 11 7-11 7Z" fill="currentColor" stroke="none" />
    </svg>
  );
}
