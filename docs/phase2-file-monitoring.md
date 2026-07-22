# Phase 2 파일 무결성 모니터링

사용자가 명시적으로 선택한 폴더만 재귀 감시합니다. 감시 경로는 추가·삭제·일시 중지·재개할 수 있으며, `enabled` 상태인 경로는 앱 시작 시 watcher 복구를 시도합니다. 일시 중지 중에는 watcher를 해제하므로 그 동안의 변경을 수집하지 않습니다.

기준선 스캔은 일반 파일의 SHA-256, 크기, 수정 시각을 SQLite에 저장합니다. 해시는 64KiB 버퍼로 스트리밍해 계산하며 파일 내용을 저장하지 않습니다. 읽기·권한 오류는 스캔을 실패 상태로 기록하고 앱 전체를 종료하지 않습니다. 재스캔은 기존에 발견된 파일의 기준선을 갱신합니다.

파일 생성·수정·삭제는 `file_events`에 기록합니다. 기준선에 없던 생성, 기준선 파일의 삭제, 또는 기준선 SHA-256과 다른 수정은 `integrity_changed` 보안 이벤트로 분류해 `medium` 심각도를 사용합니다. 그 외 활동은 `informational`입니다. 이것은 파일 변화의 설명 가능한 무결성 신호일 뿐 악성코드 확정이 아닙니다. high·critical 판정, 중요 경로 분류, 대량 변경 탐지는 Phase 3 이후의 과제입니다.

rename과 metadata 이벤트는 활동으로 기록하지만 이전 경로를 보존하거나 기준선과 비교하지 않습니다. notify 이벤트는 원시 타입을 UI에 노출하지 않고 내부 `FileEventKind`로 정규화합니다. 같은 경로·유형의 500ms 이내 이벤트는 중복 제거하며, 256개 bounded 전달 버퍼가 가득 찬 경우 최신 이벤트는 기록되지 않을 수 있습니다.

데이터는 앱 데이터 디렉터리의 SQLite에 유지됩니다. migration `0002_file_monitoring.sql`은 감시 경로 상태, 기준선, 파일 이벤트, 보안 이벤트와 조회 index를 추가합니다. schema metadata 버전을 확인해 migration을 한 번만 적용하며, 이전에 열이 이미 생성된 데이터베이스도 version 2로 안전하게 표시합니다. 보안 이벤트는 UI에서 검토 완료로 표시할 수 있습니다.

macOS에서 Desktop, Documents 등 보호된 경로를 감시·스캔하려면 운영체제의 파일 접근 권한이 필요할 수 있습니다. Windows 11 지원은 notify의 플랫폼 watcher를 사용하지만 실제 Windows 장비에서 아직 검증하지 않았습니다.

검증 명령:

```sh
npm run typecheck
npm run lint
npm run test
npm run build
cd src-tauri && cargo fmt --check && cargo check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```
