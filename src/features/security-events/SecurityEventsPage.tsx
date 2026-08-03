import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { MiniChart, SectionHeader, SeverityBadge } from "../../shared/components/Visuals";
import { findByIdOrFirst } from "../../shared/lib/collection";
import { useAsyncAction } from "../../shared/lib/use-async-action";
import { useLanguage } from "../../shared/lib/use-language";
import {
  listFileEventsFiltered,
  listIncidents,
  listSecurityEvents,
  markSecurityEventReviewed,
  updateIncidentStatus,
} from "./api";

export function SecurityEventsPage() {
  const { language } = useLanguage();
  const ko = language === "ko";
  const client = useQueryClient();
  const [severity, setSeverity] = useState("");
  const [status, setStatus] = useState("");
  const [selected, setSelected] = useState<number | null>(null);
  const { error, run } = useAsyncAction<number>(
    ko ? "Incident를 갱신할 수 없습니다." : "The incident could not be updated.",
  );
  const events = useQuery({
    queryKey: ["security-events"],
    queryFn: listSecurityEvents,
    refetchInterval: 2000,
  });
  const incidents = useQuery({
    queryKey: ["incidents", severity, status],
    queryFn: () => listIncidents(severity, status),
    refetchInterval: 2000,
  });
  const fileEvents = useQuery({
    queryKey: ["file-events-filtered"],
    queryFn: () => listFileEventsFiltered({}),
  });
  // A resolved or filtered-out selection should not strand the detail panel.
  const active = findByIdOrFirst(incidents.data, selected);
  const review = (id: number) =>
    run(
      id,
      () => markSecurityEventReviewed(id),
      () => client.invalidateQueries({ queryKey: ["security-events"] }),
    );
  const resolve = (id: number) =>
    run(
      id,
      () => updateIncidentStatus(id, "resolved"),
      () => client.invalidateQueries({ queryKey: ["incidents"] }),
    );
  return (
    <section>
      <SectionHeader
        eyebrow={
          ko ? "Incident 대응 · 조사 작업 공간" : "Incident response · investigation workspace"
        }
        title={ko ? "위협 조사" : "Threat investigation"}
        action={
          <span className="severity high">
            {incidents.data?.filter((x) => x.status !== "resolved").length ?? 0}{" "}
            {ko ? "미해결" : "OPEN"}
          </span>
        }
      />
      {error ? <p role="alert">{error}</p> : null}
      <div className="dashboard-grid">
        <article className="card panel chart-panel">
          <div className="panel-title">
            <h2>{ko ? "심각도 분포" : "Severity distribution"}</h2>
            <span>{ko ? "최근 30일" : "LAST 30 DAYS"}</span>
          </div>
          <div className="chart-grid">
            <MiniChart
              label={ko ? "Incident 심각도 분포" : "Incident severity distribution"}
              tone="red"
              values={(incidents.data ?? [])
                .slice(0, 12)
                .map(
                  (x, i) =>
                    (({ critical: 90, high: 70, medium: 48, low: 25, info: 15 })[x.severity] ??
                      20) +
                    (i % 3) * 5,
                )}
            />
          </div>
        </article>
        <article className="card panel">
          <div className="panel-title">
            <h2>{ko ? "위협 타임라인" : "Threat timeline"}</h2>
            <span>{ko ? "최근 항목" : "RECENT"}</span>
          </div>
          <div className="timeline">
            {events.data?.slice(0, 4).map((event) => (
              <div className="timeline-item" key={event.id}>
                <i className="timeline-dot" />
                <div>
                  <strong>{event.title}</strong>
                  <small>{event.occurredAt}</small>
                  {!event.reviewed ? (
                    <button className="btn secondary" onClick={() => void review(event.id)}>
                      검토 완료
                    </button>
                  ) : null}
                </div>
                <SeverityBadge severity={event.severity} />
              </div>
            )) ?? <p className="empty">{ko ? "보안 이벤트가 없습니다." : "No security events."}</p>}
          </div>
        </article>
      </div>
      <div className="page-grid" style={{ marginTop: 14 }}>
        <article className="card panel">
          <div className="panel-title">
            <h2>{ko ? "Incident 목록" : "Incident list"}</h2>
            <div className="filters">
              <select value={severity} onChange={(e) => setSeverity(e.target.value)}>
                <option value="">{ko ? "모든 심각도" : "All severities"}</option>
                <option value="critical">Critical</option>
                <option value="high">High</option>
                <option value="medium">Medium</option>
              </select>
              <select value={status} onChange={(e) => setStatus(e.target.value)}>
                <option value="">{ko ? "모든 상태" : "All status"}</option>
                <option value="open">Open</option>
                <option value="investigating">Investigating</option>
                <option value="resolved">Resolved</option>
              </select>
            </div>
          </div>
          <ul className="event-list incident-list">
            {incidents.data?.map((item) => (
              <li key={item.id}>
                <button className="path-item" onClick={() => setSelected(item.id)}>
                  <SeverityBadge severity={item.severity} />
                  <strong>{item.title}</strong>
                  <span>{item.description}</span>
                  <span>
                    {item.status} · {item.eventCount} {ko ? "개 이벤트" : "events"} ·{" "}
                    {item.lastDetectedAt}
                  </span>
                </button>
              </li>
            )) ?? (
              <li className="empty">
                {ko ? "조건에 맞는 Incident가 없습니다." : "No matching incidents."}
              </li>
            )}
          </ul>
        </article>
        <div className="stack">
          <article className="card panel">
            {active ? (
              <>
                <div className="panel-title">
                  <h2>{ko ? "Incident 상세" : "Incident detail"}</h2>
                  <SeverityBadge severity={active.severity} />
                </div>
                <p>{active.description}</p>
                <div className="detail-key">
                  <span>{ko ? "상태" : "Status"}</span>
                  <strong>{active.status}</strong>
                </div>
                <div className="detail-key">
                  <span>{ko ? "최초 탐지" : "First detected"}</span>
                  <strong>{active.firstDetectedAt}</strong>
                </div>
                <div className="detail-key">
                  <span>{ko ? "연관 이벤트" : "Related events"}</span>
                  <strong>{active.eventCount}</strong>
                </div>
                {active.status !== "resolved" ? (
                  <button className="btn" onClick={() => void resolve(active.id)}>
                    {ko ? "해결됨으로 표시" : "Mark resolved"}
                  </button>
                ) : null}
              </>
            ) : (
              <p className="empty">{ko ? "Incident를 선택하세요." : "Select an incident."}</p>
            )}
          </article>
          <article className="card panel">
            <h2>{ko ? "연관 파일 이벤트" : "Related file events"}</h2>
            <ul className="event-list">
              {fileEvents.data?.slice(0, 3).map((e) => (
                <li key={e.id}>
                  <strong>{e.eventKind}</strong>
                  <span>{e.filePath}</span>
                </li>
              )) ?? <li className="empty">{ko ? "관련 이벤트 없음" : "No related events"}</li>}
            </ul>
          </article>
        </div>
      </div>
    </section>
  );
}
