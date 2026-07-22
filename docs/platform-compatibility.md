# 플랫폼 호환성

| 기능                   | Windows 11               | macOS                       | 권한·제약                   | Phase 0 검증 |
| ---------------------- | ------------------------ | --------------------------- | --------------------------- | ------------ |
| 앱 상태/알림 UI        | supported                | supported                   | 일반 사용자                 | macOS 구현   |
| CPU·메모리·디스크      | supported (sysinfo 검토) | supported (sysinfo 검토)    | 일부 세부 정보 차이         | 설계만       |
| 프로세스 경로/부모 PID | partially_supported      | partially_supported         | 보호 프로세스·샌드박스 제한 | 설계만       |
| 명령줄·사용자          | permission_required      | permission_required         | 개인정보·권한 제한          | 설계만       |
| 파일 이벤트            | supported (notify)       | supported (FSEvents/notify) | 선택한 경로만               | 설계만       |
| 자동 시작              | supported                | supported                   | 사용자 동의 필요            | 설계만       |
| 보안 저장소            | Credential Manager 검토  | Keychain 검토               | 사용자 인증/잠금 상태       | 설계만       |

공통 모델은 optional 필드와 `supported`, `partially_supported`, `permission_required`, `unsupported`, `temporarily_unavailable` 상태를 사용합니다. Windows 빌드/실행은 이 macOS 환경에서 검증하지 않았으며, Windows CI와 실제 장비에서 확인해야 합니다.
