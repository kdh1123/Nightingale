# Phase 0 기술 결정

| 결정            | 선택                        | 이유                                                                                             |
| --------------- | --------------------------- | ------------------------------------------------------------------------------------------------ |
| 데스크톱        | Tauri 2 + Vite + React      | Tauri 공식 Vite 구성을 사용하며 Rust를 보안 경계로 유지한다.                                     |
| 로컬 DB         | `rusqlite` + bundled SQLite | 작은 로컬 동기 초기화에 알맞고, JS에 범용 SQL 권한을 주지 않는다. migration은 Rust에서 실행한다. |
| UI 서버 상태    | TanStack Query              | Tauri command의 비동기 결과/로딩/오류 상태를 일관되게 표현한다.                                  |
| 클라이언트 상태 | Zustand 미사용              | Phase 0에는 Query와 React state로 충분해 불필요한 전역 store를 피한다.                           |
| 차트            | 미선택                      | 실제 시스템 시계열이 Phase 1 전에는 없어 Recharts/ECharts를 설치하지 않는다.                     |
| 입력 검증       | Zod                         | UI 정책 입력이 생길 때 사용할 검증 경계로 준비하되 현재 화면에는 미노출이다.                     |
| 이벤트          | 향후 bounded Tokio channel  | 현재 event bus를 만들지 않고, 폭주·취소·backpressure 요구가 생기는 수집 단계에서 도입한다.       |
| 로깅            | tracing                     | Rust 내부 진단용이며 민감 경로·명령줄·비밀번호를 기록하지 않는다.                                |

Tauri는 SPA에 Vite를 권장하고 `frontendDist: ../dist` 구성을 안내한다. SQL plugin은 frontend에 SQL API를 노출하므로, Phase 0의 최소 권한 경계에는 선택하지 않았다. TanStack Query는 React 18 이상과 호환되고 object-form `useQuery` API를 제공한다.

- [Tauri Vite 구성 공식 문서](https://v2.tauri.app/start/frontend/vite/)
- [Tauri SQL plugin 공식 문서](https://v2.tauri.app/plugin/sql/)
- [TanStack Query 설치 문서](https://tanstack.com/query/latest/docs/framework/react/installation)
