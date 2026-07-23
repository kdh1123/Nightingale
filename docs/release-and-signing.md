# Release 및 서명 절차

## Release bundle

macOS에서는 `npm run tauri:build:release`로 `.app` 및 `.dmg`를, Windows에서는 `npm run tauri:build:windows`로 MSI와 NSIS 설치 프로그램을 생성합니다. 각 플랫폼에서 실행해야 하며, macOS 환경에서는 Windows 설치 프로그램을 검증하지 않습니다.

GitHub Actions의 `Release bundles` 워크플로는 `v*` 태그 또는 수동 실행에서 두 플랫폼의 번들을 artifact로 보관합니다. 실제 GitHub Release 게시와 서명은 인증서가 준비된 뒤에 추가합니다.

## macOS code signing 및 notarization

1. Apple Developer Program의 `Developer ID Application` 인증서를 CI 키체인 또는 build machine에 설치합니다.
2. `APPLE_SIGNING_IDENTITY`에 인증서 Common Name을 설정하고, `codesign --verify --deep --strict`로 `.app`을 검증합니다.
3. App Store Connect API key 또는 app-specific password를 CI secret으로 저장합니다. 필요한 값은 `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`입니다.
4. `xcrun notarytool submit <dmg> --wait`로 제출한 뒤 `xcrun stapler staple <app-or-dmg>`를 실행합니다.
5. Gatekeeper 환경에서 DMG 설치와 실행을 직접 확인합니다.

## Windows code signing

1. 신뢰할 수 있는 인증 기관에서 코드 서명 인증서를 발급받습니다.
2. 인증서 파일과 비밀번호를 CI secret에 저장하고, Windows runner에서 `signtool sign /fd SHA256 /tr <timestamp-url>`로 MSI와 NSIS 실행 파일을 모두 서명합니다.
3. `signtool verify /pa`와 깨끗한 Windows 11 VM 설치 테스트를 수행합니다.

## 배포 전 확인

- `version`을 `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`에서 동일하게 갱신합니다.
- `cargo fmt --check`, clippy, Rust/React 테스트, release build를 모두 통과시킵니다.
- 릴리스 노트, SHA-256 checksum, 라이선스 고지를 포함합니다.
