# Phase 5 성능 및 운영 준비

## 적용한 개선

- 파일 감시는 마지막 이벤트 하나가 아니라 경로별 최근 이벤트를 기준으로 중복을 억제한다. 1초가 지난 항목은 즉시 제거해 장시간 실행 시 메모리가 누적되지 않는다.
- Threat Detection 입력(설정, 기준선 비교, 최근 변경 수)을 하나의 SQLite 읽기 연결에서 가져와 이벤트당 연결·쿼리 수를 줄였다.
- SQLite는 WAL, `busy_timeout=5000`, `synchronous=NORMAL`을 사용한다. 이벤트·Incident·알림의 주요 필터 열에 인덱스를 추가했다.
- 대시보드는 리포트에 포함된 Security Score를 재사용해 별도 점수 조회를 없앴다. 리포트 조회는 더 이상 매번 `report_history`를 쓰지 않는다.

## 운영 권장 사항

로그 보관 기간은 Settings에서 조정한다. 앱 시작 시 보관 기간을 넘긴 파일 이벤트·보안 이벤트·알림을 정리한다. 대량 감시 환경에서는 SQLite 파일 크기와 이벤트 유실 경고를 관찰하고, 향후 전용 writer worker 및 명시적인 backpressure 지표를 추가하는 것이 좋다.

Release 전에는 macOS notarization, Windows 코드 서명 및 CI 실기기 파일 watcher 검증을 권장한다.
