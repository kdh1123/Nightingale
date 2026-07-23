# Nightingale 프로젝트 요약

## 목적

Nightingale은 macOS와 Windows에서 동작하는 로컬 우선 방어형 보안 모니터입니다. 사용자 지정 폴더의 파일 활동과 시스템 상태를 관찰하고, 설명 가능한 규칙으로 위협 신호·Incident·Security Score를 제공합니다. 자동 삭제, 격리, 프로세스 종료, 클라우드 연동은 수행하지 않습니다.

## 현재 기능

- 시스템·프로세스 상태 조회와 선택한 폴더의 재귀 파일 감시
- SHA-256 기준선 기반 파일 무결성 신호
- 파일 확장자, 기준선 변경, 대량 변경을 사용하는 규칙 기반 Threat Detection
- SecurityEvent를 Incident로 상관관계 묶음 처리 및 Security Score 계산
- 내부 알림, 로그 검색·필터·정렬·보관 기간 정리
- SQLite 영속 설정, 보안 리포트, Dashboard, About 화면

## 기술 구조

- UI: React, TypeScript, Vite, TanStack Query
- Desktop: Tauri 2
- Core: Rust
- Storage: 로컬 SQLite (`rusqlite`), WAL 및 busy timeout 사용
- 계층: `domain` → `application` → `repository`/`platform` → `tauri_api`

## 품질과 배포 상태

- Rust·React 정적 검사와 테스트는 GitHub Actions에서 macOS·Windows 매트릭스로 실행됩니다.
- macOS Release `.app` 및 `.dmg` 생성이 검증되었습니다.
- Windows MSI·NSIS 설정과 CI artifact 생성은 준비되었으며, Windows 실기기 설치 검증은 남아 있습니다.
- Code signing과 notarization 절차는 문서화되었고 실제 인증서는 아직 구성하지 않았습니다.

## 알려진 제한사항

- macOS 보호 폴더 감시에는 파일 접근 권한이 필요합니다.
- 이벤트가 매우 많은 환경에서는 OS watcher 버퍼 포화로 일부 이벤트가 누락될 수 있습니다.
- Threat Detection은 악성 행위 확정이 아니라 사용자의 검토를 돕는 보안 신호입니다.

자세한 설치·빌드·배포 절차는 [README](../README.md)와 [Release 문서](release-and-signing.md)를 참고하세요.
