import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
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
  const [path, setPath] = useState("");
  const [actionError, setActionError] = useState<string | null>(null);
  const [workingId, setWorkingId] = useState<number | null>(null);
  const queryClient = useQueryClient();
  const paths = useQuery({ queryKey: ["monitored-paths"], queryFn: listMonitoredPaths });
  const events = useQuery({
    queryKey: ["file-events"],
    queryFn: listFileEvents,
    refetchInterval: 2000,
  });
  const refresh = () => queryClient.invalidateQueries({ queryKey: ["monitored-paths"] });
  const add = async () => {
    if (!path.trim()) return;
    try {
      setActionError(null);
      await addMonitoredPath(path);
      setPath("");
      await refresh();
    } catch (error) {
      setActionError(error instanceof Error ? error.message : "감시 폴더를 추가할 수 없습니다.");
    }
  };
  const run = async (id: number, action: () => Promise<unknown>) => {
    try {
      setActionError(null);
      setWorkingId(id);
      await action();
      await refresh();
    } catch (error) {
      setActionError(error instanceof Error ? error.message : "요청을 처리할 수 없습니다.");
    } finally {
      setWorkingId(null);
    }
  };
  return (
    <section>
      <p className="eyebrow">파일 무결성 모니터링</p>
      <h1>감시 폴더</h1>
      <p>선택한 폴더만 로컬에서 감시합니다.</p>
      <input
        aria-label="감시 폴더 경로"
        value={path}
        onChange={(event) => setPath(event.target.value)}
        placeholder="폴더 경로"
      />
      <button onClick={() => void add()}>폴더 추가</button>
      {actionError ? <p role="alert">{actionError}</p> : null}
      {paths.isPending ? (
        <p>경로를 불러오는 중입니다.</p>
      ) : (
        <ul>
          {(paths.data ?? []).map((item) => (
            <li key={item.id} className="monitoring-item">
              <strong>{item.path}</strong>
              <span>
                감시: {item.monitoringStatus} · 기준선: {item.baselineStatus}
              </span>
              {item.lastScanAt ? <span>마지막 스캔: {item.lastScanAt}</span> : null}
              {item.lastError ? <span role="alert">오류: {item.lastError}</span> : null}
              <div className="actions">
                <button
                  disabled={workingId === item.id}
                  onClick={() => void run(item.id, () => startBaselineScan(item.id))}
                >
                  기준선 스캔
                </button>
                {item.enabled ? (
                  <button
                    disabled={workingId === item.id}
                    onClick={() => void run(item.id, () => pauseFileMonitoring(item.id))}
                  >
                    일시정지
                  </button>
                ) : (
                  <button
                    disabled={workingId === item.id}
                    onClick={() => void run(item.id, () => resumeFileMonitoring(item.id))}
                  >
                    재개
                  </button>
                )}
                <button
                  disabled={workingId === item.id}
                  onClick={() => void run(item.id, () => removeMonitoredPath(item.id))}
                >
                  삭제
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
      <h2>최근 파일 활동</h2>
      {events.isPending ? (
        <p>이벤트를 불러오는 중입니다.</p>
      ) : (
        <ul>
          {(events.data ?? []).map((event) => (
            <li key={event.id}>
              {event.eventKind} · {event.filePath} · {event.severity} · {event.occurredAt}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
