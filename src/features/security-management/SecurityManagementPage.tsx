import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { StatePanel } from "../../shared/components/StatePanel";
import { SectionHeader } from "../../shared/components/Visuals";
import { BACKGROUND_REFETCH_MS, queryKeys } from "../../shared/lib/query";
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
  type DetectionPolicy,
} from "./api";
import type { AllowlistDraft } from "./components/AllowlistSettings";
import { ReportOverview } from "./components/ReportOverview";
import { SettingsPanel } from "./components/SettingsPanel";
import { SETTINGS_CATEGORIES, type SettingsCategory } from "./settings-catalog";

const NOTIFICATION_PREVIEW_LIMIT = 4;
const NO_EXPIRY = "0";
const EMPTY_ALLOWLIST_DRAFT: AllowlistDraft = {
  entryType: "path",
  value: "",
  expiry: NO_EXPIRY,
};

type ManagementTask = "save" | "cleanup" | "read" | "policy" | "allowlist-add" | "allowlist-remove";

export function SecurityManagementPage() {
  const client = useQueryClient();
  const [category, setCategory] = useState<SettingsCategory>("General");
  const [message, setMessage] = useState<string | null>(null);
  // The draft lives here rather than in the allowlist panel so a half-typed entry survives
  // switching to another settings category and back.
  const [allowlistDraft, setAllowlistDraft] = useState<AllowlistDraft>(EMPTY_ALLOWLIST_DRAFT);
  const { activeTask, error, run } = useAsyncAction<ManagementTask>("요청을 처리할 수 없습니다.");
  const settings = useQuery({ queryKey: queryKeys.settings, queryFn: getApplicationSettings });
  const report = useQuery({ queryKey: queryKeys.securityReport, queryFn: getSecurityReport });
  const policy = useQuery({ queryKey: queryKeys.detectionPolicy, queryFn: getDetectionPolicy });
  const allowlist = useQuery({ queryKey: queryKeys.allowlist, queryFn: listAllowlistEntries });
  const allowlistAudit = useQuery({
    queryKey: queryKeys.allowlistAudit,
    queryFn: listAllowlistAudit,
  });
  const notifications = useQuery({
    queryKey: queryKeys.notifications,
    queryFn: listNotifications,
    refetchInterval: BACKGROUND_REFETCH_MS,
  });
  useEffect(() => {
    if (settings.data) document.documentElement.dataset.theme = settings.data.uiTheme;
  }, [settings.data]);

  const invalidate = (...keys: (readonly unknown[])[]) =>
    Promise.all(keys.map((queryKey) => client.invalidateQueries({ queryKey })));

  const save = (next: ApplicationSettings) =>
    run(
      "save",
      () => updateApplicationSettings(next),
      async () => {
        await invalidate(queryKeys.settings);
        setMessage("설정을 저장했습니다.");
      },
    );
  const cleanLogs = () =>
    run("cleanup", cleanupSecurityLogs, async (count) => {
      setMessage(`${count}개의 오래된 알림을 정리했습니다.`);
      await invalidate(queryKeys.notifications);
    });
  const markRead = (id: number) =>
    run(
      "read",
      () => markNotificationRead(id),
      () => invalidate(queryKeys.notifications),
    );
  const savePolicy = (next: DetectionPolicy) =>
    run(
      "policy",
      () => updateDetectionPolicy(next),
      async () => {
        await invalidate(queryKeys.detectionPolicy);
        setMessage("Detection policy saved.");
      },
    );
  const addTrustedItem = () => {
    if (!allowlistDraft.value.trim()) return;
    void run(
      "allowlist-add",
      () =>
        addAllowlistEntry(
          allowlistDraft.entryType,
          allowlistDraft.value,
          allowlistDraft.expiry === NO_EXPIRY ? null : Number(allowlistDraft.expiry),
        ),
      async () => {
        setAllowlistDraft((draft) => ({ ...draft, value: "" }));
        await invalidate(queryKeys.allowlist, queryKeys.allowlistAudit);
        setMessage("Trusted item added. Matching file activity will not create security signals.");
      },
    );
  };
  const removeTrustedItem = (id: number) =>
    void run(
      "allowlist-remove",
      () => removeAllowlistEntry(id),
      async () => {
        await invalidate(queryKeys.allowlist, queryKeys.allowlistAudit);
        setMessage("Trusted item removed.");
      },
    );

  if (settings.isPending || report.isPending)
    return <StatePanel title="보안 관리 준비 중">설정과 리포트를 불러오고 있습니다.</StatePanel>;
  if (settings.isError || report.isError || !settings.data || !report.data)
    return (
      <StatePanel title="보안 관리 정보를 불러올 수 없습니다">
        잠시 후 다시 시도해 주세요.
      </StatePanel>
    );

  const isSaving = activeTask !== null;
  const unreadCount = notifications.data?.filter((item) => !item.read).length ?? 0;
  return (
    <section>
      <SectionHeader
        eyebrow="Reports, preferences & local operations"
        title="Security management"
      />
      {message ? <p role="status">{message}</p> : null}
      {error ? <p role="alert">{error}</p> : null}
      <ReportOverview report={report.data} />
      <div className="page-grid" style={{ marginTop: 14 }}>
        <article className="card panel">
          <div className="panel-title">
            <h2>Settings</h2>
            <span>{category.toUpperCase()}</span>
          </div>
          <div className="settings-nav">
            {SETTINGS_CATEGORIES.map((item) => (
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
          <SettingsPanel
            category={category}
            settings={settings.data}
            policy={policy.data}
            disabled={isSaving}
            onSettingsChange={(next) => void save(next)}
            onPolicyChange={(next) => void savePolicy(next)}
            onCleanupLogs={() => void cleanLogs()}
            allowlist={{
              entries: allowlist.data,
              auditEntries: allowlistAudit.data,
              draft: allowlistDraft,
              onDraftChange: (patch) => setAllowlistDraft((draft) => ({ ...draft, ...patch })),
              onAdd: addTrustedItem,
              onRemove: removeTrustedItem,
            }}
          />
          <article className="card panel">
            <div className="panel-title">
              <h2>Recent notifications</h2>
              <span>{unreadCount} UNREAD</span>
            </div>
            <ul className="event-list">
              {notifications.data?.slice(0, NOTIFICATION_PREVIEW_LIMIT).map((item) => (
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
