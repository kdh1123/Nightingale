# 아키텍처

React는 `src/features/<feature>`에서 페이지·기능 전용 코드를 함께 찾는 feature-first 구조이고, 실제로 여러 기능이 재사용할 때만 `src/shared`로 승격합니다. `src/app`은 라우터·레이아웃만 책임집니다. 컴포넌트는 한 화면의 독립적 상태/상호작용 단위가 생길 때 분리하며, 단순한 한 번의 마크업을 과도하게 파일로 쪼개지 않습니다.

Rust는 `domain`(플랫폼 무관 정책/모델) → `application`(유스케이스·오류 변환) → `repository`/`platform`(외부 I/O 어댑터) → `tauri_api`(명시적 command)의 방향을 사용합니다. `platform` 내부에서만 `cfg(target_os)`를 사용합니다. 새 수집기는 공통 모델을 반환하는 adapter를 `platform`에 추가하고, detector는 이후 `application`에 입력/출력을 명확히 한 채 추가합니다.

현재 흐름은 `React query → invoke(get_app_status) → Tauri command → application → platform`입니다. 향후에는 bounded Tokio channel로 시스템 이벤트를 정규화하고, debounce/cooldown 뒤 detector가 `SecurityEvent`를 만들며 repository 기록 후 Tauri event로 UI에 최소 정보만 전달합니다. 전역 무제한 event bus는 만들지 않습니다.

`repository`는 SQLite 연결·마이그레이션과 도메인 변환을 소유합니다. Phase 0에서는 초기화만 수행합니다. sqlx보다 동기적이고 작은 로컬 SQLite 작업에 적합한 `rusqlite`를 선택해 DB 접근을 Rust 경계 안에 유지했습니다. 단점은 향후 많은 비동기 DB 작업이 생기면 전용 worker/connection 전략이 필요하다는 점입니다.
