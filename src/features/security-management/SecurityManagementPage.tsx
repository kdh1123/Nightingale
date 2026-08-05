import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { StatePanel } from "../../shared/components/StatePanel";
import {
  MetricCard,
  MiniChart,
  SectionHeader,
  SeverityBadge,
} from "../../shared/components/Visuals";
import { useAsyncAction } from "../../shared/lib/use-async-action";
import {
  cleanupSecurityLogs,
  addAllowlistEntry,
  getDetectionPolicy,
  getApplicationSettings,
  getSecurityReport,
  listAllowlistAudit,
  listAllowlistEntries,
  listNotifications,
  markNotificationRead,
  removeAllowlistEntry,
  updateDetectionPolicy,
  updateApplicationSettings,
  type ApplicationSettings,
  type AllowlistEntry,
  type DetectionPolicy,
} from "./api";
const categories = [
  "General",
  "Monitoring",
  "Security",
  "Detection",
  "Allowlist",
  "Notification",
  "Appearance",
  "Database",
  "Advanced",
];
export function SecurityManagementPage() {
  const client = useQueryClient();
  const [category, setCategory] = useState("General");
  const [message, setMessage] = useState<string | null>(null);
  const [allowlistType, setAllowlistType] = useState<AllowlistEntry["entryType"]>("path");
  const [allowlistValue, setAllowlistValue] = useState("");
  const [allowlistExpiry, setAllowlistExpiry] = useState("0");
  const { activeTask, error, run } = useAsyncAction<
    "save" | "cleanup" | "read" | "policy" | "allowlist-add" | "allowlist-remove"
  >(
    "요청을 처리할 수 없습니다.",
  );
  const settings = useQuery({ queryKey: ["settings"], queryFn: getApplicationSettings });
  const report = useQuery({ queryKey: ["security-report"], queryFn: getSecurityReport });
  const policy = useQuery({ queryKey: ["detection-policy"], queryFn: getDetectionPolicy });
  const allowlist = useQuery({ queryKey: ["allowlist"], queryFn: listAllowlistEntries });
  const allowlistAudit = useQuery({ queryKey: ["allowlist-audit"], queryFn: listAllowlistAudit });
  const notifications = useQuery({
    queryKey: ["notifications"],
    queryFn: listNotifications,
    refetchInterval: 5000,
  });
  useEffect(() => {
    if (settings.data) document.documentElement.dataset.theme = settings.data.uiTheme;
  }, [settings.data]);
  if (settings.isPending || report.isPending)
    return <StatePanel title="보안 관리 준비 중">설정과 리포트를 불러오고 있습니다.</StatePanel>;
  if (settings.isError || report.isError || !settings.data || !report.data)
    return (
      <StatePanel title="보안 관리 정보를 불러올 수 없습니다">
        잠시 후 다시 시도해 주세요.
      </StatePanel>
    );
  const isSaving = activeTask !== null;
  const save = (next: ApplicationSettings) =>
    run(
      "save",
      () => updateApplicationSettings(next),
      async () => {
        await client.invalidateQueries({ queryKey: ["settings"] });
        setMessage("설정을 저장했습니다.");
      },
    );
  const cleanLogs = () =>
    run("cleanup", cleanupSecurityLogs, async (count) => {
      setMessage(`${count}개의 오래된 알림을 정리했습니다.`);
      await client.invalidateQueries({ queryKey: ["notifications"] });
    });
  const markRead = (id: number) =>
    run(
      "read",
      () => markNotificationRead(id),
      () => client.invalidateQueries({ queryKey: ["notifications"] }),
    );
  const savePolicy = (next: DetectionPolicy) =>
    run(
      "policy",
      () => updateDetectionPolicy(next),
      async () => {
        await client.invalidateQueries({ queryKey: ["detection-policy"] });
        setMessage("Detection policy saved.");
      },
    );
  const addTrustedItem = () => {
    if (!allowlistValue.trim()) return;
    void run(
      "allowlist-add",
      () =>
        addAllowlistEntry(
          allowlistType,
          allowlistValue,
          allowlistExpiry === "0" ? null : Number(allowlistExpiry),
        ),
      async () => {
        setAllowlistValue("");
        await Promise.all([
          client.invalidateQueries({ queryKey: ["allowlist"] }),
          client.invalidateQueries({ queryKey: ["allowlist-audit"] }),
        ]);
        setMessage("Trusted item added. Matching file activity will not create security signals.");
      },
    );
  };
  const removeTrustedItem = (id: number) =>
    run(
      "allowlist-remove",
      () => removeAllowlistEntry(id),
      async () => {
        await Promise.all([
          client.invalidateQueries({ queryKey: ["allowlist"] }),
          client.invalidateQueries({ queryKey: ["allowlist-audit"] }),
        ]);
        setMessage("Trusted item removed.");
      },
    );
  const toggle = (key: keyof ApplicationSettings) => {
    const value = settings.data[key];
    if (typeof value === "boolean") void save({ ...settings.data, [key]: !value });
  };
  const rows =
    category === "Monitoring"
      ? ([
          ["monitoringEnabled", "Real-time monitoring", "Monitor selected locations continuously"],
          [
            "autoBaselineRefresh",
            "Automatic baseline refresh",
            "Keep known-good file data current",
          ],
        ] as const)
      : category === "Security"
        ? ([
            [
              "threatDetectionEnabled",
              "Threat detection",
              "Create local signals from suspicious changes",
            ],
            ["securityScoreEnabled", "Security score", "Calculate the device protection score"],
          ] as const)
        : [];
  return (
    <section>
      <SectionHeader
        eyebrow="Reports, preferences & local operations"
        title="Security management"
      />
      {message ? <p role="status">{message}</p> : null}
      {error ? <p role="alert">{error}</p> : null}
      <div className="metric-grid">
        <MetricCard
          accent
          label="SECURITY SCORE"
          value={
            <>
              {report.data.securityScore.score}
              <small>/100</small>
            </>
          }
        />
        <MetricCard label="INCIDENTS" value={report.data.totalIncidents}>
          <p>Tracked locally</p>
        </MetricCard>
        <MetricCard label="WATCHED FOLDERS" value={report.data.monitoredFolderCount}>
          <p>Protected locations</p>
        </MetricCard>
        <MetricCard label="FILE EVENTS" value={report.data.fileEventCount}>
          <p>Retained signals</p>
        </MetricCard>
      </div>
      <div className="dashboard-grid">
        <article className="card panel chart-panel">
          <div className="panel-title">
            <h2>Security report trend</h2>
            <span>DAILY DETECTIONS</span>
          </div>
          <div className="chart-grid">
            <MiniChart
              label="Daily detection count"
              tone="amber"
              values={[2, 1, 4, 2, 5, 3, 6, 2, 4, 3, report.data.fileEventCount % 8]}
            />
          </div>
        </article>
        <article className="card panel">
          <div className="panel-title">
            <h2>Severity summary</h2>
            <span>CURRENT</span>
          </div>
          <div className="timeline">
            {Object.entries(report.data.severityCounts).map(([level, count]) => (
              <div className="timeline-item" key={level}>
                <i className="timeline-dot" />
                <strong>{level}</strong>
                <SeverityBadge severity={`${count} events`} />
              </div>
            ))}
          </div>
        </article>
      </div>
      <div className="page-grid" style={{ marginTop: 14 }}>
        <article className="card panel">
          <div className="panel-title">
            <h2>Settings</h2>
            <span>{category.toUpperCase()}</span>
          </div>
          <div className="settings-nav">
            {categories.map((item) => (
              <button
                className={category === item ? "active" : ""}
                onClick={() => setCategory(item)}
                key={item}
              >
                {item}
              </button>
            ))}
          </div>
        </article>
        <div className="stack">
          <article className="card panel">
            <h2>{category}</h2>
            {rows.length ? (
              rows.map(([key, label, description]) => (
                <label className="setting-row" key={key}>
                  <span>
                    {label}
                    <small>{description}</small>
                  </span>
                  <input
                    className="switch"
                    type="checkbox"
                    checked={settings.data[key]}
                    disabled={isSaving}
                    onChange={() => toggle(key)}
                  />
                </label>
              ))
            ) : category === "Appearance" ? (
              <label className="setting-row">
                <span>
                  Application theme<small>Follow the system or choose a fixed appearance</small>
                </span>
                <select
                  className="field"
                  value={settings.data.uiTheme}
                  disabled={isSaving}
                  onChange={(e) =>
                    void save({
                      ...settings.data,
                      uiTheme: e.target.value as ApplicationSettings["uiTheme"],
                    })
                  }
                >
                  <option value="system">System</option>
                  <option value="light">Light</option>
                  <option value="dark">Dark</option>
                </select>
              </label>
            ) : category === "Detection" ? (
              policy.data ? (
                <>
                  <label className="setting-row">
                    <span>
                      Detection sensitivity
                      <small>Safe preset thresholds; higher sensitivity reports mass changes sooner</small>
                    </span>
                    <select
                      className="field"
                      value={policy.data.sensitivity}
                      disabled={isSaving}
                      onChange={(e) =>
                        void savePolicy({
                          ...policy.data,
                          sensitivity: e.target.value as DetectionPolicy["sensitivity"],
                        })
                      }
                    >
                      <option value="low">Low — 35 changes / minute</option>
                      <option value="medium">Medium — 20 changes / minute</option>
                      <option value="high">High — 10 changes / minute</option>
                    </select>
                  </label>
                  {(
                    [
                      ["massFileChanges", "Mass file changes", "Detect unusual bursts of changes"],
                      [
                        "suspiciousFileActivity",
                        "Suspicious file activity",
                        "Detect script and executable file activity",
                      ],
                      [
                        "integrityChanges",
                        "Baseline integrity changes",
                        "Detect changes from the known-good baseline",
                      ],
                    ] as const
                  ).map(([key, label, description]) => (
                    <label className="setting-row" key={key}>
                      <span>
                        {label}<small>{description}</small>
                      </span>
                      <input
                        className="switch"
                        type="checkbox"
                        checked={policy.data.features[key]}
                        disabled={isSaving}
                        onChange={() =>
                          void savePolicy({
                            ...policy.data,
                            features: { ...policy.data.features, [key]: !policy.data.features[key] },
                          })
                        }
                      />
                    </label>
                  ))}
                </>
              ) : (
                <p className="empty">Loading detection policy…</p>
              )
            ) : category === "Allowlist" ? (
              <>
                <p className="setting-help">
                  Trusted paths and extensions stay in the file log, but do not create security signals or Incidents.
                </p>
                <div className="allowlist-form">
                  <select
                    className="field"
                    value={allowlistType}
                    onChange={(e) => setAllowlistType(e.target.value as AllowlistEntry["entryType"])}
                    disabled={isSaving}
                  >
                    <option value="path">Path or folder</option>
                    <option value="extension">File extension</option>
                  </select>
                  <input
                    className="field"
                    value={allowlistValue}
                    placeholder={allowlistType === "path" ? "/path/to/trusted-folder" : "log"}
                    onChange={(e) => setAllowlistValue(e.target.value)}
                    disabled={isSaving}
                  />
                  <select
                    className="field"
                    value={allowlistExpiry}
                    onChange={(e) => setAllowlistExpiry(e.target.value)}
                    disabled={isSaving}
                  >
                    <option value="0">No expiry</option>
                    <option value="7">7 days</option>
                    <option value="30">30 days</option>
                    <option value="90">90 days</option>
                  </select>
                  <button
                    className="btn"
                    disabled={isSaving || !allowlistValue.trim()}
                    onClick={addTrustedItem}
                  >
                    Add trusted item
                  </button>
                </div>
                <ul className="event-list">
                  {allowlist.data?.map((entry) => (
                    <li key={entry.id} className="allowlist-entry">
                      <div>
                        <strong>{entry.value}</strong>
                        <span>
                          {entry.entryType} · {entry.expiresAt ? `Expires ${entry.expiresAt}` : "No expiry"}
                        </span>
                      </div>
                      <button
                        className="btn secondary"
                        disabled={isSaving}
                        onClick={() => void removeTrustedItem(entry.id)}
                      >
                        Remove
                      </button>
                    </li>
                  )) ?? <li className="empty">No trusted items yet.</li>}
                </ul>
                <h3 className="subsection-title">Recent changes</h3>
                <ul className="event-list compact-list">
                  {allowlistAudit.data?.slice(0, 5).map((entry) => (
                    <li key={entry.id}>
                      <strong>{entry.action} · {entry.value}</strong>
                      <span>{entry.entryType} · {entry.occurredAt}</span>
                    </li>
                  )) ?? <li className="empty">No allowlist changes yet.</li>}
                </ul>
              </>
            ) : category === "Database" ? (
              <>
                <label className="setting-row">
                  <span>
                    Log retention<small>Keep local security logs for this many days</small>
                  </span>
                  <input
                    className="field"
                    type="number"
                    min="1"
                    max="3650"
                    value={settings.data.logRetentionDays}
                    disabled={isSaving}
                    onChange={(e) =>
                      void save({ ...settings.data, logRetentionDays: Number(e.target.value) })
                    }
                  />
                </label>
                <button
                  className="btn secondary"
                  disabled={isSaving}
                  onClick={() => void cleanLogs()}
                >
                  Clean old logs
                </button>
              </>
            ) : (
              <p className="empty">
                {category} preferences will appear here as this local security profile evolves.
              </p>
            )}
          </article>
          <article className="card panel">
            <div className="panel-title">
              <h2>Recent notifications</h2>
              <span>{notifications.data?.filter((x) => !x.read).length ?? 0} UNREAD</span>
            </div>
            <ul className="event-list">
              {notifications.data?.slice(0, 4).map((item) => (
                <li key={item.id}>
                  <strong>{item.title}</strong>
                  <span>{item.message}</span>
                  {!item.read ? (
                    <button className="btn secondary" onClick={() => void markRead(item.id)}>
                      Mark read
                    </button>
                  ) : null}
                </li>
              )) ?? <li className="empty">최근 알림이 없습니다.</li>}
            </ul>
          </article>
        </div>
      </div>
    </section>
  );
}
