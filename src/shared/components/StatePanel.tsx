import type { ReactNode } from "react";

interface StatePanelProps {
  title: string;
  children: ReactNode;
}

export function StatePanel({ title, children }: StatePanelProps) {
  return (
    <section className="state-panel">
      <h2>{title}</h2>
      <p>{children}</p>
    </section>
  );
}
