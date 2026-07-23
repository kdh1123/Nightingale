import type { ReactNode } from "react";

export function Gauge({ value, label, detail, tone = "mint" }: { value: number; label: string; detail?: string; tone?: "mint" | "blue" | "amber" | "red" }) {
  const safe = Math.max(0, Math.min(100, value));
  return <div className={`gauge gauge-${tone}`} aria-label={`${label} ${safe.toFixed(1)}%`}>
    <svg viewBox="0 0 120 120" aria-hidden="true"><circle className="gauge-track" cx="60" cy="60" r="48" /><circle className="gauge-value" cx="60" cy="60" r="48" pathLength="100" strokeDasharray={`${safe} 100`} /></svg>
    <div className="gauge-copy"><strong>{safe.toFixed(0)}<small>%</small></strong><span>{label}</span>{detail ? <em>{detail}</em> : null}</div>
  </div>;
}

export function MiniChart({ values, tone = "mint", label }: { values: number[]; tone?: "mint" | "blue" | "amber" | "red"; label: string }) {
  const max = Math.max(...values, 1);
  const points = values.map((value, index) => `${(index / Math.max(values.length - 1, 1)) * 100},${92 - (value / max) * 76}`).join(" ");
  return <div className={`mini-chart chart-${tone}`} role="img" aria-label={label}><svg viewBox="0 0 100 100" preserveAspectRatio="none"><polyline points={points} /></svg></div>;
}

export function ResourceMeter({ label, value, values, tone, detail }: { label: string; value: number; values: number[]; tone: "cpu" | "memory"; detail: string }) {
  const safe = Math.max(0, Math.min(100, value));
  return <article className={`resource-meter resource-${tone}`}><div className="resource-heading"><div><span>{label}</span><strong>{safe.toFixed(1)}<small>%</small></strong></div><em>{detail}</em></div><MiniChart label={`${label} usage history`} values={values} tone={tone === "cpu" ? "blue" : "mint"} /><div className="meter-scale"><span>0%</span><div><i style={{ width: `${safe}%` }} /></div><span>100%</span></div></article>;
}

export function SectionHeader({ eyebrow, title, action }: { eyebrow?: string; title: string; action?: ReactNode }) {
  return <div className="section-header"><div>{eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}<h1>{title}</h1></div>{action}</div>;
}

export function SeverityBadge({ severity }: { severity: string }) { return <span className={`severity ${severity.toLowerCase()}`}>{severity}</span>; }
