import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { StatePanel } from "../../shared/components/StatePanel";
import { getSystemSnapshot, listProcesses } from "./api";

export function SystemStatusPage() {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState("cpu");
  const snapshot = useQuery({
    queryKey: ["snapshot"],
    queryFn: getSystemSnapshot,
    refetchInterval: 2000,
  });
  const processes = useQuery({
    queryKey: ["processes", query, sort],
    queryFn: () => listProcesses(query, sort),
    refetchInterval: 2000,
  });
  if (snapshot.isPending)
    return (
      <StatePanel title="시스템 상태를 불러오는 중">최대 2초마다 안전하게 갱신합니다.</StatePanel>
    );
  if (snapshot.isError)
    return (
      <StatePanel title="시스템 상태를 불러올 수 없습니다">잠시 후 다시 시도하세요.</StatePanel>
    );
  const value = snapshot.data;
  return (
    <section>
      <p className="eyebrow">실시간 모니터링 · 2초 간격</p>
      <h1>시스템 상태</h1>
      <div className="status-grid">
        <article>
          <span>CPU</span>
          <strong>{value.cpuPercent.toFixed(1)}%</strong>
        </article>
        <article>
          <span>메모리</span>
          <strong>{value.memory.percent.toFixed(1)}%</strong>
        </article>
        <article>
          <span>디스크</span>
          <strong>{value.disk.percent.toFixed(1)}%</strong>
        </article>
        <article>
          <span>프로세서</span>
          <strong>{value.logicalCpuCount} 논리 코어</strong>
        </article>
      </div>
      <p>
        {value.operatingSystem} {value.operatingSystemVersion ?? ""} · 마지막 갱신{" "}
        {new Date(value.collectedAtUnix * 1000).toLocaleTimeString()}
      </p>
      <h2>프로세스</h2>
      <input
        aria-label="프로세스 검색"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="이름 또는 PID 검색"
      />
      <select
        aria-label="프로세스 정렬"
        value={sort}
        onChange={(event) => setSort(event.target.value)}
      >
        <option value="cpu">CPU</option>
        <option value="memory">메모리</option>
        <option value="name">이름</option>
      </select>
      {processes.isError ? (
        <StatePanel title="프로세스 목록을 불러올 수 없습니다">다시 시도하세요.</StatePanel>
      ) : (
        <ul>
          {(processes.data ?? []).slice(0, 100).map((item) => (
            <li key={item.pid}>
              {item.name} · PID {item.pid} · CPU {item.cpuPercent.toFixed(1)}% ·{" "}
              {(item.memoryBytes / 1024 / 1024).toFixed(0)} MB
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
