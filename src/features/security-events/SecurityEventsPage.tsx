import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { listFileEventsFiltered, listIncidents, listSecurityEvents, markSecurityEventReviewed, updateIncidentStatus } from "./api";

export function SecurityEventsPage() {
  const queryClient = useQueryClient();
  const [workingId, setWorkingId] = useState<number | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [severity, setSeverity] = useState("");
  const [incidentStatus, setIncidentStatus] = useState("");
  const [logQuery, setLogQuery] = useState("");
  const [logSeverity, setLogSeverity] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [sortDesc, setSortDesc] = useState(true);
  const events = useQuery({
    queryKey: ["security-events"],
    queryFn: listSecurityEvents,
    refetchInterval: 2000,
  });
  const incidents = useQuery({
    queryKey: ["incidents", severity, incidentStatus],
    queryFn: () => listIncidents(severity, incidentStatus),
    refetchInterval: 2000,
  });
  const fileEvents = useQuery({ queryKey: ["file-events", logQuery, logSeverity, from, to, sortDesc], queryFn: () => listFileEventsFiltered({ query: logQuery, severity: logSeverity, from, to, sortDesc }) });
  const review = async (id: number) => {
    setWorkingId(id);
    try {
      setActionError(null);
      await markSecurityEventReviewed(id);
      await queryClient.invalidateQueries({ queryKey: ["security-events"] });
    } catch (error) {
      setActionError(
        error instanceof Error ? error.message : "이벤트를 검토 완료로 표시할 수 없습니다.",
      );
    } finally {
      setWorkingId(null);
    }
  };
  const changeIncidentStatus = async (id: number, status: "open" | "investigating" | "resolved") => {
    await updateIncidentStatus(id, status);
    await queryClient.invalidateQueries({ queryKey: ["incidents"] });
  };
  if (events.isPending) return <p>보안 이벤트를 불러오는 중입니다.</p>;
  if (events.isError) return <p role="alert">보안 이벤트를 불러올 수 없습니다.</p>;
  return (
    <section>
      <p className="eyebrow">파일 모니터링 알림</p>
      <h1>보안 이벤트</h1>
      {actionError ? <p role="alert">{actionError}</p> : null}
      <h2>Incident</h2>
      <div className="filters">
        <label>Severity <select value={severity} onChange={(event) => setSeverity(event.target.value)}><option value="">전체</option><option value="info">info</option><option value="low">low</option><option value="medium">medium</option><option value="high">high</option><option value="critical">critical</option></select></label>
        <label>상태 <select value={incidentStatus} onChange={(event) => setIncidentStatus(event.target.value)}><option value="">전체</option><option value="open">open</option><option value="investigating">investigating</option><option value="resolved">resolved</option></select></label>
      </div>
      {incidents.data?.length ? <ul className="event-list">{incidents.data.map((incident) => <li key={incident.id}><strong>{incident.title}</strong><span>{incident.description}</span><span>{incident.severity} · {incident.status} · 이벤트 {incident.eventCount}건</span><span>{incident.lastDetectedAt}</span>{incident.status !== "resolved" ? <button onClick={() => void changeIncidentStatus(incident.id, "resolved")}>해결 완료</button> : null}</li>)}</ul> : <p>조건에 맞는 Incident가 없습니다.</p>}
      <h2>보안 이벤트</h2>
      {events.data?.length ? (
        <ul className="event-list">
          {events.data.map((event) => (
            <li key={event.id}>
              <strong>{event.title}</strong>
              <span>{event.description}</span>
              <span>
                {event.severity} · {event.occurredAt}
              </span>
              {event.reviewed ? (
                <span>검토 완료</span>
              ) : (
                <button disabled={workingId === event.id} onClick={() => void review(event.id)}>
                  검토 완료
                </button>
              )}
            </li>
          ))}
        </ul>
      ) : (
        <p>기록된 보안 이벤트가 없습니다.</p>
      )}
      <h2>파일 이벤트 로그</h2><div className="filters"><label>경로 검색 <input value={logQuery} onChange={(event) => setLogQuery(event.target.value)} /></label><label>Severity <select value={logSeverity} onChange={(event) => setLogSeverity(event.target.value)}><option value="">전체</option><option value="info">info</option><option value="low">low</option><option value="medium">medium</option><option value="high">high</option></select></label><label>시작일 <input type="date" value={from} onChange={(event) => setFrom(event.target.value)} /></label><label>종료일 <input type="date" value={to} onChange={(event) => setTo(event.target.value)} /></label><button onClick={() => setSortDesc((value) => !value)}>{sortDesc ? "최신순" : "오래된순"}</button></div>
      {fileEvents.isPending ? <p>로그를 불러오는 중입니다.</p> : fileEvents.data?.length ? <ul className="event-list">{fileEvents.data.map((event) => <li key={event.id}><strong>{event.eventKind}</strong><span>{event.filePath}</span><span>{event.severity} · {event.occurredAt}</span></li>)}</ul> : <p>조건에 맞는 파일 이벤트가 없습니다.</p>}
    </section>
  );
}
