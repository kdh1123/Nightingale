import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { listSecurityEvents, markSecurityEventReviewed } from "./api";

export function SecurityEventsPage() {
  const queryClient = useQueryClient();
  const [workingId, setWorkingId] = useState<number | null>(null);
  const events = useQuery({
    queryKey: ["security-events"],
    queryFn: listSecurityEvents,
    refetchInterval: 2000,
  });
  const review = async (id: number) => {
    setWorkingId(id);
    try {
      await markSecurityEventReviewed(id);
      await queryClient.invalidateQueries({ queryKey: ["security-events"] });
    } finally {
      setWorkingId(null);
    }
  };
  if (events.isPending) return <p>보안 이벤트를 불러오는 중입니다.</p>;
  if (events.isError) return <p role="alert">보안 이벤트를 불러올 수 없습니다.</p>;
  return (
    <section>
      <p className="eyebrow">파일 모니터링 알림</p>
      <h1>보안 이벤트</h1>
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
    </section>
  );
}
