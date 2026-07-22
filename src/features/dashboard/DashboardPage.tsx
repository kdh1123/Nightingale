import { useQuery } from "@tanstack/react-query";
import { StatePanel } from "../../shared/components/StatePanel";
import { getAppStatus } from "../../shared/lib/tauri";

export function DashboardPage() {
  const status = useQuery({ queryKey: ["app-status"], queryFn: getAppStatus, retry: false });
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
      <p className="eyebrow">PHASE 0 · 기반 준비</p>
      <h1>보안 상태</h1>
      <p>
        모니터링 기능은 아직 준비 중입니다. 이 화면은 앱과 플랫폼 준비 상태를 안전하게 확인합니다.
      </p>
      <div className="status-grid">
        <article>
          <span>운영체제</span>
          <strong>{status.data.operatingSystem}</strong>
        </article>
        <article>
          <span>앱 버전</span>
          <strong>{status.data.appVersion}</strong>
        </article>
      </div>
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
