import { TimerView } from "../components/Timer/TimerView";

export function TimerPage() {
  return (
    <section className="page page-timer">
      <h2 className="visually-hidden">Timer</h2>
      <TimerView />
    </section>
  );
}
