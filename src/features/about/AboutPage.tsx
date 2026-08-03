import { useQuery } from "@tanstack/react-query";
import { StatePanel } from "../../shared/components/StatePanel";
import { getAppStatus } from "../../shared/lib/tauri";

export function AboutPage() {
  const status = useQuery({ queryKey: ["app-status"], queryFn: getAppStatus, retry: false });
  if (status.isPending)
    return <StatePanel title="Nightingale 정보">제품 정보를 불러오는 중입니다.</StatePanel>;
  if (status.isError)
    return (
      <StatePanel title="정보를 불러올 수 없습니다">
        앱을 다시 시작한 뒤 다시 시도해 주세요.
      </StatePanel>
    );
  return (
    <section>
      <p className="eyebrow">ABOUT NIGHTINGALE</p>
      <h1>Nightingale</h1>
      <p>로컬 우선 방식으로 파일 활동과 시스템 상태를 분석하는 방어형 보안 모니터입니다.</p>
      <div className="status-grid">
        <article>
          <span>버전</span>
          <strong>{status.data.appVersion}</strong>
        </article>
        <article>
          <span>운영체제</span>
          <strong>{status.data.operatingSystem}</strong>
        </article>
        <article>
          <span>라이선스</span>
          <strong>MIT</strong>
        </article>
        <article>
          <span>제품 식별자</span>
          <strong>com.nightingale.securitymonitor</strong>
        </article>
      </div>
      <p>
        파일을 자동으로 삭제·격리·종료하지 않으며, 감지 결과는 사용자의 검토를 돕기 위한 보안
        신호입니다.
      </p>
    </section>
  );
}
