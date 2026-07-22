import { StatePanel } from "../../shared/components/StatePanel";

export function SystemStatusPage() {
  return (
    <StatePanel title="시스템 모니터링은 아직 준비 중입니다">
      CPU, 메모리, 디스크, 프로세스 수집은 Phase 1에서 최소 권한 원칙으로 추가됩니다.
    </StatePanel>
  );
}
