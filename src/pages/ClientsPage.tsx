import { ClientsList } from "../components/Clients/ClientsList";

export function ClientsPage() {
  return (
    <section className="page">
      <header className="page-header">
        <h2>Clients</h2>
        <p>Manage the clients available for time tracking.</p>
      </header>
      <ClientsList />
    </section>
  );
}
