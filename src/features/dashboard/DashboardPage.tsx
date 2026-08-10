import { useQuery } from "@tanstack/react-query";
import { StatePanel } from "../../shared/components/StatePanel";
import {
  Gauge,
  MetricCard,
  ResourceMeter,
  SectionHeader,
  SeverityBadge,
} from "../../shared/components/Visuals";
import { formatMebibytes } from "../../shared/lib/format";
import { BACKGROUND_REFETCH_MS, LIVE_REFETCH_MS, queryKeys } from "../../shared/lib/query";
import { useLanguage } from "../../shared/lib/use-language";
import { getAppStatus } from "../../shared/lib/tauri";
import { getSecurityReport } from "../security-management/api";
import {
  getSystemSnapshot,
  listProcesses,
  terminateProcess,
  type ProcessSummary,
} from "../system-status/api";
import { estimateNetworkLoadPercent } from "../system-status/metrics";

// A process only shows up in the resource alert list when it is clearly outside normal use.
const RESOURCE_ALERT_CPU_PERCENT = 15;
const RESOURCE_ALERT_MEMORY_BYTES = 1024 * 1024 * 1024;
const RESOURCE_ALERT_LIMIT = 3;

function selectResourceHogs(processes: ProcessSummary[]) {
  return processes
    .filter(
      (process) =>
        process.cpuPercent >= RESOURCE_ALERT_CPU_PERCENT ||
        process.memoryBytes >= RESOURCE_ALERT_MEMORY_BYTES,
    )
    .slice(0, RESOURCE_ALERT_LIMIT);
}

export function DashboardPage() {
  const { language } = useLanguage();
  const ko = language === "ko";
  const status = useQuery({ queryKey: queryKeys.appStatus, queryFn: getAppStatus, retry: false });
  const report = useQuery({
    queryKey: queryKeys.securityReport,
    queryFn: getSecurityReport,
    retry: false,
    refetchInterval: BACKGROUND_REFETCH_MS,
  });
  const system = useQuery({
    queryKey: queryKeys.systemSnapshot,
    queryFn: getSystemSnapshot,
    refetchInterval: LIVE_REFETCH_MS,
  });
  const processes = useQuery({
    queryKey: queryKeys.resourceAlertProcesses,
    queryFn: () => listProcesses(undefined, "cpu"),
    refetchInterval: BACKGROUND_REFETCH_MS,
  });
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
  const score = report.data?.securityScore.score ?? 100;
  const incidents = report.data?.securityScore.openIncidentCount ?? 0;
  const cpu = system.data?.cpuPercent ?? 0;
  const memory = system.data?.memory.percent ?? 0;
  const resourceHogs = selectResourceHogs(processes.data ?? []);
  const requestTermination = (process: ProcessSummary) => {
    const confirmed = window.confirm(
      ko
        ? `${process.name}을(를) 종료할까요? 저장하지 않은 작업이 손실될 수 있습니다.`
        : `Close ${process.name}? Unsaved work may be lost.`,
    );
    if (confirmed) void terminateProcess(process.pid).then(() => processes.refetch());
  };
  return (
    <section>
      <SectionHeader
        eyebrow={ko ? "보안 관제 센터 · 로컬 보호" : "Security command center · local protection"}
        title={ko ? "현재 PC는 안전하게 보호되고 있습니다" : "Your device is protected"}
        action={
          <span className="severity low">● {ko ? "정상 운영 중" : "ALL SYSTEMS NOMINAL"}</span>
        }
      />
      <div className="metric-grid">
        <MetricCard
          accent
          label={ko ? "보안 점수" : "SECURITY SCORE"}
          value={
            <>
              {score}
              <small> / 100</small>
            </>
          }
        >
          <p>
            {incidents
              ? ko
                ? `${incidents}건의 Incident 검토 필요`
                : `${incidents} incidents require review`
              : ko
                ? "활성화된 보안 우려가 없습니다"
                : "No active security concerns"}
          </p>
        </MetricCard>
        <MetricCard label={ko ? "활성 Incident" : "ACTIVE INCIDENTS"} value={incidents}>
          <p>
            {report.data?.securityScore.criticalIncidentCount ?? 0}
            {ko ? "건의 긴급 우선순위" : " critical priority"}
          </p>
        </MetricCard>
        <MetricCard
          label={ko ? "모니터링 상태" : "MONITORING STATUS"}
          value={ko ? "활성" : "Active"}
        >
          <p>{ko ? "파일 무결성 감시가 실행 중입니다" : "File integrity watch is running"}</p>
        </MetricCard>
        <MetricCard
          label={ko ? "보호 파일 이벤트" : "PROTECTED FILES"}
          value={report.data?.fileEventCount ?? 0}
        >
          <p>
            {report.data?.monitoredFolderCount ?? 0}
            {ko ? "개 감시 경로" : " watched locations"}
          </p>
        </MetricCard>
      </div>
      <div className="dashboard-grid">
        <div className="stack">
          <article className="card panel">
            <div className="panel-title">
              <h2>{ko ? "CPU · 메모리 사용 현황" : "CPU & memory usage"}</h2>
              <span>{ko ? "실시간 · 2초" : "LIVE · 2 SEC"}</span>
            </div>
            <div className="resource-grid">
              <ResourceMeter
                label="CPU"
                value={cpu}
                tone="cpu"
                detail={ko ? "처리 성능" : "PROCESSING"}
                values={[18, 25, 22, 41, 30, 53, 39, 48, 33, 42, cpu]}
              />
              <ResourceMeter
                label={ko ? "메모리" : "Memory"}
                value={memory}
                tone="memory"
                detail={ko ? "할당 메모리" : "ALLOCATED"}
                values={[38, 39, 42, 41, 44, 45, 43, 48, 46, 50, memory]}
              />
            </div>
          </article>
          <article className="card panel">
            <div className="panel-title">
              <h2>{ko ? "보호 범위" : "Protection coverage"}</h2>
              <span>{ko ? "현재 상태" : "CURRENT"}</span>
            </div>
            <div className="gauge-row">
              <Gauge label="CPU" value={cpu} tone="blue" />
              <Gauge label={ko ? "메모리" : "Memory"} value={memory} />
              <Gauge
                label={ko ? "디스크" : "Disk"}
                value={system.data?.disk.percent ?? 0}
                tone="amber"
              />
              <Gauge
                label={ko ? "네트워크" : "Network"}
                value={estimateNetworkLoadPercent(cpu)}
                tone="blue"
              />
            </div>
          </article>
        </div>
        <article className="card panel">
          <div className="panel-title">
            <h2>{ko ? "위협 타임라인" : "Threat timeline"}</h2>
            <span>
              {report.data?.recentRiskEvents.length ?? 0} {ko ? "최근 신호" : "RECENT SIGNALS"}
            </span>
          </div>
          <div className="timeline">
            {report.data?.recentRiskEvents.slice(0, 5).map((event) => (
              <div className="timeline-item" key={event.id}>
                <i className="timeline-dot" />
                <div>
                  <strong>{event.title}</strong>
                  <small>{event.description}</small>
                </div>
                <SeverityBadge severity={event.severity} />
              </div>
            )) ?? (
              <p className="empty">
                {ko ? "최근 위협 신호가 없습니다." : "No recent threat signals."}
              </p>
            )}
          </div>
          <div className="detail-key">
            <span>{ko ? "보안 데이터" : "Security data"}</span>
            <strong>{ko ? "이 기기에만 저장" : "Stored on this device"}</strong>
          </div>
          <div className="detail-key">
            <span>{ko ? "마지막 상태 확인" : "Last health check"}</span>
            <strong>{ko ? "방금 전" : "Just now"}</strong>
          </div>
        </article>
      </div>
      <div className="dashboard-grid">
        <article className="card panel">
          <div className="panel-title">
            <h2>{ko ? "리소스 사용 경고" : "Resource usage alerts"}</h2>
            <span>
              {resourceHogs.length} {ko ? "개 확인" : "DETECTED"}
            </span>
          </div>
          {resourceHogs.length ? (
            <div className="resource-alerts">
              {resourceHogs.map((process) => (
                <div className="resource-alert" key={process.pid}>
                  <div>
                    <strong>{process.name}</strong>
                    <small>
                      PID {process.pid} · CPU {process.cpuPercent.toFixed(1)}% ·{" "}
                      {formatMebibytes(process.memoryBytes)}
                    </small>
                  </div>
                  <button className="btn danger" onClick={() => requestTermination(process)}>
                    {ko ? "앱 종료 제안" : "Close app"}
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <p className="empty">
              {ko
                ? "현재 CPU나 메모리를 과도하게 사용하는 앱이 없습니다."
                : "No apps are using an unusual amount of CPU or memory."}
            </p>
          )}
          <p>
            {ko
              ? "종료는 사용자가 직접 확인한 경우에만 요청됩니다."
              : "Closing is only requested after your explicit confirmation."}
          </p>
        </article>
        <article className="card panel">
          <div className="panel-title">
            <h2>{ko ? "빠른 작업" : "Quick actions"}</h2>
            <span>{ko ? "보호 제어" : "PROTECTION"}</span>
          </div>
          <div className="quick-actions">
            <button className="btn">
              <span>01 / {ko ? "제어" : "CONTROL"}</span>
              {ko ? "모니터링 시작" : "Start monitoring"}
            </button>
            <button className="btn secondary">
              <span>02 / {ko ? "제어" : "CONTROL"}</span>
              {ko ? "모니터링 중지" : "Stop monitoring"}
            </button>
            <button className="btn secondary">
              <span>03 / {ko ? "검사" : "CHECK"}</span>
              {ko ? "지금 검사" : "Scan now"}
            </button>
            <button className="btn secondary">
              <span>04 / {ko ? "검토" : "REVIEW"}</span>
              {ko ? "리포트 열기" : "Open reports"}
            </button>
          </div>
        </article>
      </div>
    </section>
  );
}
