import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { StatePanel } from "../../shared/components/StatePanel";
import { cleanupSecurityLogs, getApplicationSettings, getSecurityReport, listNotifications, markNotificationRead, updateApplicationSettings, type ApplicationSettings } from "./api";

export function SecurityManagementPage() {
  const client = useQueryClient();
  const settings = useQuery({ queryKey: ["settings"], queryFn: getApplicationSettings });
  const report = useQuery({ queryKey: ["security-report"], queryFn: getSecurityReport });
  const notifications = useQuery({ queryKey: ["notifications"], queryFn: listNotifications, refetchInterval: 5000 });
  const [message, setMessage] = useState<string | null>(null);
  useEffect(() => { if (settings.data) document.documentElement.dataset.theme = settings.data.uiTheme; }, [settings.data]);
  if (settings.isPending || report.isPending) return <StatePanel title="보안 관리 준비 중">설정과 리포트를 불러오고 있습니다.</StatePanel>;
  if (settings.isError || report.isError || !settings.data || !report.data) return <StatePanel title="보안 관리 정보를 불러올 수 없습니다">잠시 후 다시 시도해 주세요.</StatePanel>;
  const save = async (next: ApplicationSettings) => { await updateApplicationSettings(next); await client.invalidateQueries({ queryKey: ["settings"] }); setMessage("설정을 저장했습니다."); };
  const toggle = (key: keyof ApplicationSettings) => { const value = settings.data[key]; if (typeof value === "boolean") void save({ ...settings.data, [key]: !value }); };
  return <section>
    <p className="eyebrow">PHASE 4 · 보안 관리</p><h1>보안 관리</h1>{message ? <p role="status">{message}</p> : null}
    <h2>Security Report</h2><div className="status-grid"><article><span>Security Score</span><strong>{report.data.securityScore.score} / 100</strong></article><article><span>전체 Incident</span><strong>{report.data.totalIncidents}</strong></article><article><span>감시 폴더</span><strong>{report.data.monitoredFolderCount}</strong></article><article><span>파일 이벤트</span><strong>{report.data.fileEventCount}</strong></article></div>
    <div className="severity-row">{Object.entries(report.data.severityCounts).map(([severity, count]) => <span key={severity} className={`severity ${severity}`}>{severity}: {count}</span>)}</div>
    <h2>애플리케이션 설정</h2><div className="settings-list">{([['monitoringEnabled','감시 활성화'], ['threatDetectionEnabled','Threat Detection'], ['autoBaselineRefresh','자동 Baseline 갱신'], ['securityScoreEnabled','Security Score 계산']] as const).map(([key,label]) => <label key={key}><input type="checkbox" checked={settings.data[key]} onChange={() => toggle(key)} /> {label}</label>)}<label>로그 보관 기간 <input aria-label="로그 보관 기간" type="number" min="1" max="3650" value={settings.data.logRetentionDays} onChange={(event) => void save({ ...settings.data, logRetentionDays: Number(event.target.value) })} />일</label><label>테마 <select value={settings.data.uiTheme} onChange={(event) => void save({ ...settings.data, uiTheme: event.target.value as ApplicationSettings["uiTheme"] })}><option value="system">시스템</option><option value="light">라이트</option><option value="dark">다크</option></select></label></div>
    <button onClick={() => void cleanupSecurityLogs().then((count) => setMessage(`${count}개의 오래된 알림을 정리했습니다.`))}>오래된 로그 정리</button>
    <h2>최근 알림</h2>{notifications.data?.length ? <ul className="event-list">{notifications.data.map((item) => <li key={item.id}><strong>{item.title}</strong><span>{item.message}</span><span>{item.severity} · {item.createdAt}</span>{!item.read ? <button onClick={() => void markNotificationRead(item.id).then(() => client.invalidateQueries({ queryKey: ["notifications"] }))}>읽음 처리</button> : <span>읽음</span>}</li>)}</ul> : <p>최근 알림이 없습니다.</p>}
  </section>;
}
