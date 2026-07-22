# 보안 모델과 위협 모델

보호 대상은 로컬 정책, 이벤트 기록, 선택한 감시 경로의 무결성 기준, 향후 암호화된 보관함입니다. 신뢰 경계는 React 입력 ↔ Tauri command ↔ Rust 도메인 ↔ SQLite/OS API입니다. 모든 UI 입력은 신뢰하지 않으며, 이후 경로는 정규화·allowlist·traversal 검사를 거칩니다.

현재 통제는 최소 Tauri capability(`core:default`), 명시 command 하나, frontend에 DB/키 접근을 주지 않는 구조, SQLite parameter binding 원칙, WAL/foreign key, 민감 정보를 로그에 남기지 않는 정책입니다. DB는 OS 앱 데이터 디렉터리의 `nightingale.sqlite3`에 저장됩니다.

남은 위험: 로컬 DB 탈취, 과도한 파일 경로/프로세스 메타데이터 수집, 향후 command 입력, 손상된 DB, 코드 서명되지 않은 배포물입니다. 비밀번호 기능은 이후 검증된 AEAD와 Argon2id를 사용하고 nonce를 재사용하지 않으며, 키 재료는 Keychain/Credential Manager 사용 가능성을 먼저 검토합니다. 앱은 자동 종료·삭제·차단·관리자 권한 획득을 하지 않습니다.
