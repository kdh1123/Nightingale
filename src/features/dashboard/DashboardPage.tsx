import { useQuery } from "@tanstack/react-query";
import { StatePanel } from "../../shared/components/StatePanel";
import { getAppStatus } from "../../shared/lib/tauri";
import { getSecurityReport } from "../security-management/api";

export function DashboardPage() {
  const status = useQuery({ queryKey: ["app-status"], queryFn: getAppStatus, retry: false });
  const report = useQuery({ queryKey: ["security-report"], queryFn: getSecurityReport, retry: false });
  if (status.isPending)
    return (
      <StatePanel title="Nightingale 준비 중">안전한 상태 정보를 확인하고 있습니다.</StatePanel>
    );
  if (status.isError)
    return (
      <StatePanel title="상태를 불러올 수 없습니다">
        앱을 다시 시작해 보세요. 문제가 계속되면 관리자에게 문의하세요.
      </StatePanel>
    );
  return (
    <section>
      <p className="eyebrow">PHASE 4 · Security Management Platform</p>
      <h1>보안 상태</h1>
      <p>
        모니터링 기능은 아직 준비 중입니다. 이 화면은 앱과 플랫폼 준비 상태를 안전하게 확인합니다.
      </p>
      <div className="status-grid">
        <article>
          <span>Active Incident</span>
          <strong>{report.data?.securityScore.openIncidentCount ?? "—"}</strong>
          <p>해결되지 않은 보안 Incident</p>
        </article>
        <article>
          <span>Security Score</span>
          <strong>{report.data ? `${report.data.securityScore.score} / 100` : "계산 중"}</strong>
          {report.data ? <p>열린 Incident {report.data.securityScore.openIncidentCount}건</p> : null}
        </article>
        <article>
          <span>운영체제</span>
          <strong>{status.data.operatingSystem}</strong>
        </article>
        <article>
          <span>앱 버전</span>
          <strong>{status.data.appVersion}</strong>
        </article>
      </div>
      {report.data ? <><h2>최근 탐지 및 Severity 분포</h2><div className="severity-row">{Object.entries(report.data.severityCounts).map(([severity, count]) => <span key={severity} className={`severity ${severity}`}>{severity}: {count}</span>)}</div>{report.data.recentRiskEvents.length ? <ul className="event-list">{report.data.recentRiskEvents.slice(0, 5).map((event) => <li key={event.id}><strong>{event.title}</strong><span>{event.severity} · {event.occurredAt}</span></li>)}</ul> : <p>최근 위험 이벤트가 없습니다.</p>}</> : null}
      <h2>기능 준비 상태</h2>
      <ul className="capabilities">
        {status.data.capabilities.map((item) => (
          <li key={item.key}>
            <strong>{item.key}</strong>
            <span className={`badge ${item.status}`}>{item.status}</span>
            <p>{item.detail}</p>
          </li>
        ))}
      </ul>
    </section>
  );
}
