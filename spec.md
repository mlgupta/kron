# Build a macOS Time Tracking Application using Tauri + React + TypeScript

Build a complete, production-quality **macOS desktop time-tracking application** called **TimeTracker**.

The application is intended for an individual consultant who works for multiple clients and needs to track time spent on each client.

## Technology Requirements

Use:

* Tauri 2.x
* React
* TypeScript
* Vite
* Rust for the Tauri backend
* SQLite for persistent local storage
* A suitable SQLite Rust library such as `rusqlite` or `sqlx`
* Modern React functional components and hooks
* TypeScript with strict type checking

The application must be a **self-contained macOS desktop application**.

The user should ultimately be able to build a single `TimeTracker.app` that starts everything when launched.

Do NOT create a separate web server or require Docker.

The React frontend communicates with the Rust backend through Tauri commands/events.

---

# 1. Application Architecture

Use this architecture:

```text
┌─────────────────────────────────────────────────────────────┐
│                     TimeTracker.app                         │
│                                                             │
│  ┌───────────────────────┐      ┌────────────────────────┐ │
│  │ React + TypeScript    │      │ Tauri / Rust           │ │
│  │                       │      │                        │ │
│  │ Timer UI              │◄────►│ Client Service         │ │
│  │ Client Management     │      │ Timer Service          │ │
│  │ Timesheet              │      │ Report Service         │ │
│  │ Reports               │      │ Database Service       │ │
│  └───────────────────────┘      └───────────┬────────────┘ │
│                                             │              │
│                                             ▼              │
│                                      ┌──────────────┐      │
│                                      │    SQLite    │      │
│                                      │ local DB     │      │
│                                      └──────────────┘      │
│                                                             │
│  macOS integration: menu bar, notifications, startup       │
└─────────────────────────────────────────────────────────────┘
```

Do not allow the React frontend to directly access SQLite.

All database operations must go through Tauri/Rust commands.

---

# 2. Core Features

The application must support:

1. CRUD operations for clients
2. Start a timer for a client
3. Stop a timer
4. Persist completed timer sessions
5. Round elapsed time UP to the next 15-minute increment
6. View time by client
7. View totals by:

   * Day
   * Week
   * Month
8. View individual time sessions
9. Edit/delete previously recorded sessions
10. Persistent data across application restarts
11. A macOS menu-bar interface for quick timer operations

---

# 3. Client Management

Create a Clients screen.

Each client has:

```text
Client
------
id
name
description
active
created_at
updated_at
```

Support:

* Create client
* Read/list clients
* Update client
* Delete client
* Activate/deactivate client

Do not allow deletion of a client that has associated time sessions unless the user explicitly confirms.

Prefer soft deletion/deactivation rather than physically deleting clients that have historical sessions.

Example UI:

```text
Clients

┌──────────────────────────────────────────────────────────┐
│ Client Name              Status       Actions             │
├──────────────────────────────────────────────────────────┤
│ Client A                 Active       Edit   Deactivate   │
│ Client B                 Active       Edit   Deactivate   │
│ Client C                 Inactive     Edit   Activate     │
└──────────────────────────────────────────────────────────┘

                         + Add Client
```

---

# 4. Timer

Create a prominent Timer view.

The user selects a client and clicks Start.

Example:

```text
┌──────────────────────────────────────┐
│              TIME TRACKER             │
│                                      │
│ Client                               │
│ ┌──────────────────────────────────┐ │
│ │ Client A                       ▼ │ │
│ └──────────────────────────────────┘ │
│                                      │
│             02:17:34                 │
│                                      │
│             [ START ]                │
│                                      │
└──────────────────────────────────────┘
```

When running:

```text
┌──────────────────────────────────────┐
│              TIME TRACKER             │
│                                      │
│ Client A                            │
│                                      │
│             02:17:34                 │
│                                      │
│             [ STOP ]                 │
│                                      │
│ Started:  9:02 AM                   │
└──────────────────────────────────────┘
```

Only one timer may be running at a time.

If the user attempts to start a timer while another timer is running, clearly warn the user.

Prefer allowing the user to switch clients by stopping the current timer and starting a new one, but require confirmation before doing so.

---

# 5. Timer Persistence

The timer must survive application restarts.

Do NOT rely solely on an in-memory JavaScript timer.

When Start is clicked:

```text
started_at = current timestamp
```

Persist the running timer state.

When Stop is clicked:

```text
stopped_at = current timestamp

actual_duration =
    stopped_at - started_at
```

Calculate elapsed time from timestamps rather than counting JavaScript timer ticks.

This ensures accurate time even if the application is suspended or restarted.

On application startup:

1. Check whether a timer was running.
2. Restore the timer state.
3. Calculate current elapsed time from `started_at`.
4. Display the correct elapsed time.

---

# 6. 15-Minute Rounding Rule

When a timer is stopped, calculate the actual elapsed duration and round it UP to the next 15-minute increment.

Formula:

```text
rounded_minutes =
    ceil(actual_minutes / 15) * 15
```

Examples:

```text
Actual        Stored/Billable
--------------------------------
1 minute      15 minutes
7 minutes     15 minutes
14 minutes    15 minutes
15 minutes    15 minutes
16 minutes    30 minutes
22 minutes    30 minutes
29 minutes    30 minutes
30 minutes    30 minutes
31 minutes    45 minutes
44 minutes    45 minutes
45 minutes    45 minutes
46 minutes    60 minutes
61 minutes    75 minutes
```

Store both:

```text
actual_duration_seconds
rounded_duration_minutes
```

Do NOT round the running timer display.

The running timer should show the actual elapsed time.

Only round when the timer is stopped and the session is persisted.

---

# 7. Database Schema

Use SQLite.

Create migrations.

Initial schema:

```sql
CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE time_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    stopped_at TEXT NOT NULL,
    actual_duration_seconds INTEGER NOT NULL,
    rounded_duration_minutes INTEGER NOT NULL,
    note TEXT,
    created_at TEXT NOT NULL,

    FOREIGN KEY (client_id)
        REFERENCES clients(id)
);
```

Also create a mechanism for tracking the currently running timer.

This may be a dedicated table such as:

```sql
CREATE TABLE active_timer (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    client_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    note TEXT,

    FOREIGN KEY (client_id)
        REFERENCES clients(id)
);
```

There must never be more than one active timer.

Use transactions where appropriate.

---

# 8. Rust/Tauri Commands

Create clean Tauri commands.

At minimum:

```text
create_client
get_clients
get_client
update_client
delete_client
set_client_active

start_timer
stop_timer
get_active_timer
is_timer_running

get_sessions
get_session
update_session
delete_session

get_daily_totals
get_weekly_totals
get_monthly_totals
```

Return strongly typed serializable Rust structures.

Do not return raw database rows directly to the frontend.

Create appropriate Rust models/DTOs.

---

# 9. Reporting

Create a Reports screen.

The user must be able to select:

```text
[ Day ] [ Week ] [ Month ]
```

and navigate between periods.

For example:

```text
                 September 3, 2026
                 <              >

Client                     Time
------------------------------------
Client A                   4h 15m
Client B                   2h 30m
Client C                   1h 45m
Internal                   0h 30m
------------------------------------
Total                      9h 00m
```

Weekly:

```text
              Week of Aug 31
              <           >

Client                     Time
------------------------------------
Client A                  18h 45m
Client B                   9h 30m
Client C                   6h 15m
------------------------------------
Total                     34h 30m
```

Monthly:

```text
                 September 2026
                 <            >

Client                     Time
------------------------------------
Client A                  72h 15m
Client B                  38h 30m
Client C                  21h 45m
------------------------------------
Total                    132h 30m
```

Reports should use `rounded_duration_minutes`, not actual duration.

---

# 10. Timesheet

Create a Timesheet view showing individual sessions.

Example:

```text
Date       Client       Start       End         Actual       Rounded
-----------------------------------------------------------------------
Sep 3      Client A     09:02       10:11       1h 09m      1h 15m
Sep 3      Client B     10:30       11:02       0h 32m      0h 45m
Sep 3      Client A     13:15       14:01       0h 46m      1h 00m
```

Allow:

* Filtering by client
* Filtering by date range
* Editing a session
* Deleting a session

When editing a session, recalculate `actual_duration_seconds` and `rounded_duration_minutes`.

---

# 11. Menu Bar

Make this a proper macOS menu-bar application using Tauri.

The menu bar should show the current state.

When no timer is running:

```text
🕐 TimeTracker
```

When running:

```text
🕐 Client A  02:17
```

Clicking the menu-bar icon should show:

```text
┌──────────────────────────────┐
│ Client A                     │
│                              │
│        02:17:34              │
│                              │
│        [ Stop ]              │
│                              │
│ ──────────────────────────── │
│ Switch Client                │
│   Client B                   │
│   Client C                   │
│   Client D                   │
│                              │
│ ──────────────────────────── │
│ Today's total: 6h 45m        │
│                              │
│ Open TimeTracker              │
│ Quit                          │
└──────────────────────────────┘
```

If there is no active timer:

```text
┌──────────────────────────────┐
│ Start Timer                  │
│                              │
│   Client A                   │
│   Client B                   │
│   Client C                   │
│                              │
│ ──────────────────────────── │
│ Today's total: 6h 45m        │
│                              │
│ Open TimeTracker              │
│ Quit                          │
└──────────────────────────────┘
```

---

# 12. UI Design

Use a clean, modern macOS-style interface.

Keep it simple rather than making it look like a complex enterprise application.

Main navigation:

```text
┌────────────────────────────────────────────┐
│ TimeTracker                                │
├──────────────┬─────────────────────────────┤
│              │                             │
│ Timer        │                             │
│              │                             │
│ Timesheet    │       Main Content          │
│              │                             │
│ Reports      │                             │
│              │                             │
│ Clients      │                             │
│              │                             │
│ Settings     │                             │
│              │                             │
└──────────────┴─────────────────────────────┘
```

Support both light and dark macOS appearance.

Use accessible controls and keyboard navigation.

---

# 13. Date and Time Handling

Store timestamps in a consistent format.

Prefer UTC internally and convert to local time for display.

The application is initially intended for a single user on a single Mac.

Make the reporting date boundaries use the user's local timezone.

Pay particular attention to:

* Midnight boundaries
* Week boundaries
* Month boundaries
* Daylight saving time

Do not implement naive date calculations in the React frontend.

Date-range calculations should be handled consistently in the Rust/backend layer.

---

# 14. Error Handling

Implement proper error handling.

Examples:

* Cannot create duplicate client name
* Cannot start timer when another timer is running
* Cannot stop timer when no timer is running
* Database failure
* Invalid session
* Attempt to delete client with historical sessions
* Invalid date range

Show user-friendly error messages.

Do not expose Rust/database stack traces to the user.

Log technical errors appropriately for debugging.

---

# 15. Project Structure

Use a clean project structure similar to:

```text
timetracker/
│
├── src/
│   ├── components/
│   │   ├── Timer/
│   │   ├── Clients/
│   │   ├── Timesheet/
│   │   └── Reports/
│   │
│   ├── pages/
│   │   ├── TimerPage.tsx
│   │   ├── ClientsPage.tsx
│   │   ├── TimesheetPage.tsx
│   │   └── ReportsPage.tsx
│   │
│   ├── services/
│   │   └── tauri.ts
│   │
│   ├── hooks/
│   │   └── useTimer.ts
│   │
│   ├── types/
│   │   └── index.ts
│   │
│   └── App.tsx
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── clients.rs
│   │   │   ├── timer.rs
│   │   │   ├── sessions.rs
│   │   │   └── reports.rs
│   │   │
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   └── migrations.rs
│   │   │
│   │   ├── models/
│   │   │   └── mod.rs
│   │   │
│   │   └── services/
│   │       ├── timer_service.rs
│   │       ├── client_service.rs
│   │       └── report_service.rs
│   │
│   └── tauri.conf.json
│
└── package.json
```

The exact structure may be adjusted if Tauri conventions suggest a better organization.

---

# 16. State Management

Avoid unnecessary global state.

Use React state/hooks where possible.

A lightweight state-management library such as Zustand may be used if it makes the architecture cleaner.

The source of truth for persistent data must remain SQLite.

The active timer state should be synchronized between:

```text
SQLite
   ↕
Rust
   ↕
React
   ↕
Menu Bar
```

---

# 17. Testing

Create tests for the most important business logic.

Especially test the rounding algorithm:

```text
1   -> 15
14  -> 15
15  -> 15
16  -> 30
29  -> 30
30  -> 30
31  -> 45
44  -> 45
45  -> 45
46  -> 60
60  -> 60
61  -> 75
```

Also test:

* Starting timer
* Stopping timer
* Restarting application while timer is active
* Multiple clients
* Daily aggregation
* Weekly aggregation
* Monthly aggregation
* Sessions crossing midnight
* Sessions crossing daylight-saving boundaries
* Editing sessions
* Deleting sessions

---

# 18. Build and Packaging

The final application must be buildable as a macOS `.app`.

Configure Tauri so that:

```text
npm run tauri build
```

(or the appropriate package-manager command)

produces a macOS application/bundle.

The end user should NOT need:

* Node.js
* Rust
* SQLite
* A web browser
* A local server
* Docker

The application must contain everything it needs at runtime.

Store the SQLite database in the appropriate macOS application-data directory rather than inside the application bundle.

---

# 19. Important Design Principles

Keep the application simple.

Do NOT add:

* User accounts
* Cloud synchronization
* A web server
* Authentication
* Multi-user support
* Billing
* Jira integration
* Project-management features
* AI features

Those may be considered later.

The goal is a reliable personal time-tracking application.

Prioritize:

1. Accurate time tracking
2. Reliable persistence
3. Simple client management
4. Correct 15-minute rounding
5. Useful day/week/month reports
6. Excellent macOS menu-bar experience
7. Clean maintainable code

---

# 20. Implementation Approach

Do not generate everything as one giant implementation without validation.

Implement in logical phases:

### Phase 1

Create the Tauri + React + TypeScript project and verify it builds and launches on macOS.

### Phase 2

Implement SQLite database, migrations, and client CRUD.

### Phase 3

Implement timer start/stop and persistent active timer.

### Phase 4

Implement 15-minute rounding and session persistence.

### Phase 5

Implement Timesheet.

### Phase 6

Implement day/week/month reports.

### Phase 7

Implement macOS menu-bar integration.

### Phase 8

Add testing, error handling, polish, and packaging.

After each phase, make sure the project still builds successfully.

At the end, provide:

1. Project structure
2. Installation instructions
3. Development commands
4. Build commands
5. Database location
6. Explanation of the major architectural decisions
7. Instructions for running the finished `.app`

Generate actual working code, not pseudocode or placeholders.
# Build a macOS Time Tracking Application using Tauri + React + TypeScript

Build a complete, production-quality **macOS desktop time-tracking application** called **TimeTracker**.

The application is intended for an individual consultant who works for multiple clients and needs to track time spent on each client.

## Technology Requirements

Use:

* Tauri 2.x
* React
* TypeScript
* Vite
* Rust for the Tauri backend
* SQLite for persistent local storage
* A suitable SQLite Rust library such as `rusqlite` or `sqlx`
* Modern React functional components and hooks
* TypeScript with strict type checking

The application must be a **self-contained macOS desktop application**.

The user should ultimately be able to build a single `TimeTracker.app` that starts everything when launched.

Do NOT create a separate web server or require Docker.

The React frontend communicates with the Rust backend through Tauri commands/events.

---

# 1. Application Architecture

Use this architecture:

```text
┌─────────────────────────────────────────────────────────────┐
│                     TimeTracker.app                         │
│                                                             │
│  ┌───────────────────────┐      ┌────────────────────────┐ │
│  │ React + TypeScript    │      │ Tauri / Rust           │ │
│  │                       │      │                        │ │
│  │ Timer UI              │◄────►│ Client Service         │ │
│  │ Client Management     │      │ Timer Service          │ │
│  │ Timesheet              │      │ Report Service         │ │
│  │ Reports               │      │ Database Service       │ │
│  └───────────────────────┘      └───────────┬────────────┘ │
│                                             │              │
│                                             ▼              │
│                                      ┌──────────────┐      │
│                                      │    SQLite    │      │
│                                      │ local DB     │      │
│                                      └──────────────┘      │
│                                                             │
│  macOS integration: menu bar, notifications, startup       │
└─────────────────────────────────────────────────────────────┘
```

Do not allow the React frontend to directly access SQLite.

All database operations must go through Tauri/Rust commands.

---

# 2. Core Features

The application must support:

1. CRUD operations for clients
2. Start a timer for a client
3. Stop a timer
4. Persist completed timer sessions
5. Round elapsed time UP to the next 15-minute increment
6. View time by client
7. View totals by:

   * Day
   * Week
   * Month
8. View individual time sessions
9. Edit/delete previously recorded sessions
10. Persistent data across application restarts
11. A macOS menu-bar interface for quick timer operations

---

# 3. Client Management

Create a Clients screen.

Each client has:

```text
Client
------
id
name
description
active
created_at
updated_at
```

Support:

* Create client
* Read/list clients
* Update client
* Delete client
* Activate/deactivate client

Do not allow deletion of a client that has associated time sessions unless the user explicitly confirms.

Prefer soft deletion/deactivation rather than physically deleting clients that have historical sessions.

Example UI:

```text
Clients

┌──────────────────────────────────────────────────────────┐
│ Client Name              Status       Actions             │
├──────────────────────────────────────────────────────────┤
│ Client A                 Active       Edit   Deactivate   │
│ Client B                 Active       Edit   Deactivate   │
│ Client C                 Inactive     Edit   Activate     │
└──────────────────────────────────────────────────────────┘

                         + Add Client
```

---

# 4. Timer

Create a prominent Timer view.

The user selects a client and clicks Start.

Example:

```text
┌──────────────────────────────────────┐
│              TIME TRACKER             │
│                                      │
│ Client                               │
│ ┌──────────────────────────────────┐ │
│ │ Client A                       ▼ │ │
│ └──────────────────────────────────┘ │
│                                      │
│             02:17:34                 │
│                                      │
│             [ START ]                │
│                                      │
└──────────────────────────────────────┘
```

When running:

```text
┌──────────────────────────────────────┐
│              TIME TRACKER             │
│                                      │
│ Client A                            │
│                                      │
│             02:17:34                 │
│                                      │
│             [ STOP ]                 │
│                                      │
│ Started:  9:02 AM                   │
└──────────────────────────────────────┘
```

Only one timer may be running at a time.

If the user attempts to start a timer while another timer is running, clearly warn the user.

Prefer allowing the user to switch clients by stopping the current timer and starting a new one, but require confirmation before doing so.

---

# 5. Timer Persistence

The timer must survive application restarts.

Do NOT rely solely on an in-memory JavaScript timer.

When Start is clicked:

```text
started_at = current timestamp
```

Persist the running timer state.

When Stop is clicked:

```text
stopped_at = current timestamp

actual_duration =
    stopped_at - started_at
```

Calculate elapsed time from timestamps rather than counting JavaScript timer ticks.

This ensures accurate time even if the application is suspended or restarted.

On application startup:

1. Check whether a timer was running.
2. Restore the timer state.
3. Calculate current elapsed time from `started_at`.
4. Display the correct elapsed time.

---

# 6. 15-Minute Rounding Rule

When a timer is stopped, calculate the actual elapsed duration and round it UP to the next 15-minute increment.

Formula:

```text
rounded_minutes =
    ceil(actual_minutes / 15) * 15
```

Examples:

```text
Actual        Stored/Billable
--------------------------------
1 minute      15 minutes
7 minutes     15 minutes
14 minutes    15 minutes
15 minutes    15 minutes
16 minutes    30 minutes
22 minutes    30 minutes
29 minutes    30 minutes
30 minutes    30 minutes
31 minutes    45 minutes
44 minutes    45 minutes
45 minutes    45 minutes
46 minutes    60 minutes
61 minutes    75 minutes
```

Store both:

```text
actual_duration_seconds
rounded_duration_minutes
```

Do NOT round the running timer display.

The running timer should show the actual elapsed time.

Only round when the timer is stopped and the session is persisted.

---

# 7. Database Schema

Use SQLite.

Create migrations.

Initial schema:

```sql
CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE time_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    stopped_at TEXT NOT NULL,
    actual_duration_seconds INTEGER NOT NULL,
    rounded_duration_minutes INTEGER NOT NULL,
    note TEXT,
    created_at TEXT NOT NULL,

    FOREIGN KEY (client_id)
        REFERENCES clients(id)
);
```

Also create a mechanism for tracking the currently running timer.

This may be a dedicated table such as:

```sql
CREATE TABLE active_timer (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    client_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    note TEXT,

    FOREIGN KEY (client_id)
        REFERENCES clients(id)
);
```

There must never be more than one active timer.

Use transactions where appropriate.

---

# 8. Rust/Tauri Commands

Create clean Tauri commands.

At minimum:

```text
create_client
get_clients
get_client
update_client
delete_client
set_client_active

start_timer
stop_timer
get_active_timer
is_timer_running

get_sessions
get_session
update_session
delete_session

get_daily_totals
get_weekly_totals
get_monthly_totals
```

Return strongly typed serializable Rust structures.

Do not return raw database rows directly to the frontend.

Create appropriate Rust models/DTOs.

---

# 9. Reporting

Create a Reports screen.

The user must be able to select:

```text
[ Day ] [ Week ] [ Month ]
```

and navigate between periods.

For example:

```text
                 September 3, 2026
                 <              >

Client                     Time
------------------------------------
Client A                   4h 15m
Client B                   2h 30m
Client C                   1h 45m
Internal                   0h 30m
------------------------------------
Total                      9h 00m
```

Weekly:

```text
              Week of Aug 31
              <           >

Client                     Time
------------------------------------
Client A                  18h 45m
Client B                   9h 30m
Client C                   6h 15m
------------------------------------
Total                     34h 30m
```

Monthly:

```text
                 September 2026
                 <            >

Client                     Time
------------------------------------
Client A                  72h 15m
Client B                  38h 30m
Client C                  21h 45m
------------------------------------
Total                    132h 30m
```

Reports should use `rounded_duration_minutes`, not actual duration.

---

# 10. Timesheet

Create a Timesheet view showing individual sessions.

Example:

```text
Date       Client       Start       End         Actual       Rounded
-----------------------------------------------------------------------
Sep 3      Client A     09:02       10:11       1h 09m      1h 15m
Sep 3      Client B     10:30       11:02       0h 32m      0h 45m
Sep 3      Client A     13:15       14:01       0h 46m      1h 00m
```

Allow:

* Filtering by client
* Filtering by date range
* Editing a session
* Deleting a session

When editing a session, recalculate `actual_duration_seconds` and `rounded_duration_minutes`.

---

# 11. Menu Bar

Make this a proper macOS menu-bar application using Tauri.

The menu bar should show the current state.

When no timer is running:

```text
🕐 TimeTracker
```

When running:

```text
🕐 Client A  02:17
```

Clicking the menu-bar icon should show:

```text
┌──────────────────────────────┐
│ Client A                     │
│                              │
│        02:17:34              │
│                              │
│        [ Stop ]              │
│                              │
│ ──────────────────────────── │
│ Switch Client                │
│   Client B                   │
│   Client C                   │
│   Client D                   │
│                              │
│ ──────────────────────────── │
│ Today's total: 6h 45m        │
│                              │
│ Open TimeTracker              │
│ Quit                          │
└──────────────────────────────┘
```

If there is no active timer:

```text
┌──────────────────────────────┐
│ Start Timer                  │
│                              │
│   Client A                   │
│   Client B                   │
│   Client C                   │
│                              │
│ ──────────────────────────── │
│ Today's total: 6h 45m        │
│                              │
│ Open TimeTracker              │
│ Quit                          │
└──────────────────────────────┘
```

---

# 12. UI Design

Use a clean, modern macOS-style interface.

Keep it simple rather than making it look like a complex enterprise application.

Main navigation:

```text
┌────────────────────────────────────────────┐
│ TimeTracker                                │
├──────────────┬─────────────────────────────┤
│              │                             │
│ Timer        │                             │
│              │                             │
│ Timesheet    │       Main Content          │
│              │                             │
│ Reports      │                             │
│              │                             │
│ Clients      │                             │
│              │                             │
│ Settings     │                             │
│              │                             │
└──────────────┴─────────────────────────────┘
```

Support both light and dark macOS appearance.

Use accessible controls and keyboard navigation.

---

# 13. Date and Time Handling

Store timestamps in a consistent format.

Prefer UTC internally and convert to local time for display.

The application is initially intended for a single user on a single Mac.

Make the reporting date boundaries use the user's local timezone.

Pay particular attention to:

* Midnight boundaries
* Week boundaries
* Month boundaries
* Daylight saving time

Do not implement naive date calculations in the React frontend.

Date-range calculations should be handled consistently in the Rust/backend layer.

---

# 14. Error Handling

Implement proper error handling.

Examples:

* Cannot create duplicate client name
* Cannot start timer when another timer is running
* Cannot stop timer when no timer is running
* Database failure
* Invalid session
* Attempt to delete client with historical sessions
* Invalid date range

Show user-friendly error messages.

Do not expose Rust/database stack traces to the user.

Log technical errors appropriately for debugging.

---

# 15. Project Structure

Use a clean project structure similar to:

```text
timetracker/
│
├── src/
│   ├── components/
│   │   ├── Timer/
│   │   ├── Clients/
│   │   ├── Timesheet/
│   │   └── Reports/
│   │
│   ├── pages/
│   │   ├── TimerPage.tsx
│   │   ├── ClientsPage.tsx
│   │   ├── TimesheetPage.tsx
│   │   └── ReportsPage.tsx
│   │
│   ├── services/
│   │   └── tauri.ts
│   │
│   ├── hooks/
│   │   └── useTimer.ts
│   │
│   ├── types/
│   │   └── index.ts
│   │
│   └── App.tsx
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── clients.rs
│   │   │   ├── timer.rs
│   │   │   ├── sessions.rs
│   │   │   └── reports.rs
│   │   │
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   └── migrations.rs
│   │   │
│   │   ├── models/
│   │   │   └── mod.rs
│   │   │
│   │   └── services/
│   │       ├── timer_service.rs
│   │       ├── client_service.rs
│   │       └── report_service.rs
│   │
│   └── tauri.conf.json
│
└── package.json
```

The exact structure may be adjusted if Tauri conventions suggest a better organization.

---

# 16. State Management

Avoid unnecessary global state.

Use React state/hooks where possible.

A lightweight state-management library such as Zustand may be used if it makes the architecture cleaner.

The source of truth for persistent data must remain SQLite.

The active timer state should be synchronized between:

```text
SQLite
   ↕
Rust
   ↕
React
   ↕
Menu Bar
```

---

# 17. Testing

Create tests for the most important business logic.

Especially test the rounding algorithm:

```text
1   -> 15
14  -> 15
15  -> 15
16  -> 30
29  -> 30
30  -> 30
31  -> 45
44  -> 45
45  -> 45
46  -> 60
60  -> 60
61  -> 75
```

Also test:

* Starting timer
* Stopping timer
* Restarting application while timer is active
* Multiple clients
* Daily aggregation
* Weekly aggregation
* Monthly aggregation
* Sessions crossing midnight
* Sessions crossing daylight-saving boundaries
* Editing sessions
* Deleting sessions

---

# 18. Build and Packaging

The final application must be buildable as a macOS `.app`.

Configure Tauri so that:

```text
npm run tauri build
```

(or the appropriate package-manager command)

produces a macOS application/bundle.

The end user should NOT need:

* Node.js
* Rust
* SQLite
* A web browser
* A local server
* Docker

The application must contain everything it needs at runtime.

Store the SQLite database in the appropriate macOS application-data directory rather than inside the application bundle.

---

# 19. Important Design Principles

Keep the application simple.

Do NOT add:

* User accounts
* Cloud synchronization
* A web server
* Authentication
* Multi-user support
* Billing
* Jira integration
* Project-management features
* AI features

Those may be considered later.

The goal is a reliable personal time-tracking application.

Prioritize:

1. Accurate time tracking
2. Reliable persistence
3. Simple client management
4. Correct 15-minute rounding
5. Useful day/week/month reports
6. Excellent macOS menu-bar experience
7. Clean maintainable code

---

# 20. Implementation Approach

Do not generate everything as one giant implementation without validation.

Implement in logical phases:

### Phase 1

Create the Tauri + React + TypeScript project and verify it builds and launches on macOS.

### Phase 2

Implement SQLite database, migrations, and client CRUD.

### Phase 3

Implement timer start/stop and persistent active timer.

### Phase 4

Implement 15-minute rounding and session persistence.

### Phase 5

Implement Timesheet.

### Phase 6

Implement day/week/month reports.

### Phase 7

Implement macOS menu-bar integration.

### Phase 8

Add testing, error handling, polish, and packaging.

After each phase, make sure the project still builds successfully.

At the end, provide:

1. Project structure
2. Installation instructions
3. Development commands
4. Build commands
5. Database location
6. Explanation of the major architectural decisions
7. Instructions for running the finished `.app`

Generate actual working code, not pseudocode or placeholders.
