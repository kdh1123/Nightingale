import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { SectionHeader, SeverityBadge } from "../../shared/components/Visuals";
import { findByIdOrFirst } from "../../shared/lib/collection";
import { LIVE_REFETCH_MS, queryKeys } from "../../shared/lib/query";
import { useAsyncAction } from "../../shared/lib/use-async-action";
import { useLanguage } from "../../shared/lib/use-language";
import {
  addMonitoredPath,
  listFileEvents,
  listMonitoredPaths,
  pauseFileMonitoring,
  removeMonitoredPath,
  resumeFileMonitoring,
  startBaselineScan,
} from "./api";

export function FileMonitoringPage() {
  const { language } = useLanguage();
  const ko = language === "ko";
  const [path, setPath] = useState("");
  const [selected, setSelected] = useState<number | null>(null);
  const client = useQueryClient();
  const {
    activeTask: working,
    error,
    run,
  } = useAsyncAction<number>(
    ko ? "요청을 처리할 수 없습니다." : "The request could not be completed.",
  );
  const paths = useQuery({ queryKey: queryKeys.monitoredPaths, queryFn: listMonitoredPaths });
  const events = useQuery({
    queryKey: queryKeys.fileEvents,
    queryFn: listFileEvents,
    refetchInterval: LIVE_REFETCH_MS,
  });
  const refresh = () =>
    Promise.all([
      client.invalidateQueries({ queryKey: queryKeys.monitoredPaths }),
      client.invalidateQueries({ queryKey: queryKeys.fileEvents }),
    ]);
  // Fall back to the first remaining location after a selected location is removed.
  const active = findByIdOrFirst(paths.data, selected);
  return (
    <section>
      <SectionHeader
        eyebrow={ko ? "파일 무결성 모니터링 · 로컬 전용" : "File integrity monitoring · local only"}
        title={ko ? "모니터링 작업 공간" : "Monitoring workspace"}
        action={
          <div className="filters">
            <input
              aria-label="감시 폴더 경로"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder={ko ? "감시할 폴더 경로 추가" : "Add folder path"}
            />
            <button
              className="btn"
              onClick={() =>
                void (path.trim()
                  ? run(
                      -1,
                      async () => {
                        await addMonitoredPath(path.trim());
                        setPath("");
                      },
                      refresh,
                    )
                  : undefined)
              }
            >
              {ko ? "경로 추가" : "Add location"}
            </button>
          </div>
        }
      />
      {error ? <p role="alert">{error}</p> : null}
      <div className="page-grid">
        <article className="card panel">
          <div className="panel-title">
            <h2>{ko ? "감시 중인 경로" : "Watched locations"}</h2>
            <span>
              {paths.data?.length ?? 0} {ko ? "개 경로" : "LOCATIONS"}
            </span>
          </div>
          <div className="path-list">
            {paths.isPending ? (
              <p className="empty">{ko ? "경로를 불러오는 중입니다." : "Loading locations."}</p>
            ) : (
              (paths.data ?? []).map((item) => (
                <button
                  key={item.id}
                  className={`path-item ${active?.id === item.id ? "selected" : ""}`}
                  onClick={() => setSelected(item.id)}
                >
                  <strong>{item.path}</strong>
                  <small>
                    {item.enabled
                      ? ko
                        ? "모니터링 활성화"
                        : "Monitoring active"
                      : ko
                        ? "모니터링 일시정지"
                        : "Monitoring paused"}{" "}
                    · {item.baselineStatus}
                  </small>
                </button>
              ))
            )}
          </div>
        </article>
        <article className="card panel">
          {active ? (
            <>
              <div className="panel-title">
                <h2>{ko ? "경로 상세 정보" : "Location details"}</h2>
                <span>
                  {active.enabled ? (ko ? "활성" : "ACTIVE") : ko ? "일시정지" : "PAUSED"}
                </span>
              </div>
              <div className="detail-key">
                <span>{ko ? "기준선" : "Baseline"}</span>
                <strong>{active.baselineStatus}</strong>
              </div>
              <div className="detail-key">
                <span>{ko ? "마지막 검사" : "Last scan"}</span>
                <strong>{active.lastScanAt ?? (ko ? "검사 기록 없음" : "Not scanned")}</strong>
              </div>
              <div className="detail-key">
                <span>{ko ? "마지막 활동" : "Last activity"}</span>
                <strong>{active.lastEventAt ?? (ko ? "활동 없음" : "None")}</strong>
              </div>
              {active.lastError ? <p role="alert">{active.lastError}</p> : null}
              <div className="quick-actions">
                <button
                  className="btn"
                  disabled={working === active.id}
                  onClick={() => void run(active.id, () => startBaselineScan(active.id), refresh)}
                >
                  {ko ? "기준선 스캔" : "Baseline scan"}
                </button>
                <button
                  className="btn secondary"
                  disabled={working === active.id}
                  onClick={() =>
                    void run(
                      active.id,
                      () =>
                        active.enabled
                          ? pauseFileMonitoring(active.id)
                          : resumeFileMonitoring(active.id),
                      refresh,
                    )
                  }
                >
                  {active.enabled ? (ko ? "일시정지" : "Pause") : ko ? "재개" : "Resume"}
                </button>
                <button
                  className="btn danger"
                  disabled={working === active.id}
                  onClick={() => void run(active.id, () => removeMonitoredPath(active.id), refresh)}
                >
                  {ko ? "삭제" : "Remove"}
                </button>
              </div>
            </>
          ) : (
            <p className="empty">{ko ? "감시 경로를 선택하세요." : "Select a watched location."}</p>
          )}
        </article>
      </div>
      <div className="dashboard-grid">
        <article className="card panel">
          <div className="panel-title">
            <h2>{ko ? "파일 활동 타임라인" : "File activity timeline"}</h2>
            <span>{ko ? "실시간 이벤트" : "LIVE EVENTS"}</span>
          </div>
          <div className="timeline">
            {events.data?.slice(0, 7).map((event) => (
              <div key={event.id} className="timeline-item">
                <i className="timeline-dot" />
                <div>
                  <strong>{event.eventKind}</strong>
                  <small>{event.filePath}</small>
                </div>
                <SeverityBadge severity={event.severity} />
              </div>
            )) ?? (
              <p className="empty">
                {ko ? "최근 파일 활동이 없습니다." : "No recent file activity."}
              </p>
            )}
          </div>
        </article>
        <article className="card panel">
          <h2>{ko ? "조사 맥락" : "Investigation context"}</h2>
          <p>
            {ko
              ? "선택한 경로의 변경 흐름과 연결된 보안 신호를 확인하세요."
              : "Review file changes and their connected security signals."}
          </p>
          <div className="detail-key">
            <span>{ko ? "연관 Incident" : "Related incidents"}</span>
            <strong>{ko ? "위협 탐지에서 검토" : "Review in Threats"}</strong>
          </div>
          <div className="detail-key">
            <span>{ko ? "이벤트 보관" : "Event retention"}</span>
            <strong>{ko ? "로컬 데이터베이스" : "Local database"}</strong>
          </div>
        </article>
      </div>
    </section>
  );
}
