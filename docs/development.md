# 개발·배포 메모

macOS: Xcode Command Line Tools, Node.js, 공식 rustup의 Rust stable을 설치합니다. rustup 설치 뒤에는 새 셸을 열거나 `. "$HOME/.cargo/env"`를 실행하고 `rustup component add rustfmt clippy`로 검사 도구를 설치합니다. 이 프로젝트는 전체 Xcode 없이 macOS에서 `cargo check`, `cargo test`, `npm run tauri dev` 및 `.app` 번들 생성까지 확인했습니다.

Windows: Rust MSVC target, Visual Studio C++ Build Tools, WebView2가 필요합니다. `npm install`, `npm run tauri dev`로 실행하고 README의 검사 명령을 사용합니다. `npm run tauri build`는 macOS 앱 번들과 DMG를 생성합니다. Finder 자동화가 제한된 환경에서는 DMG 레이아웃 단계가 대기할 수 있으므로 `.app` 번들 생성 여부와 DMG 생성 단계를 구분해 확인합니다.

GitHub Actions는 PR과 `main` push에서 React·Rust 정적 검사와 테스트를 macOS·Windows로 실행합니다. `v*` 태그와 수동 실행은 플랫폼별 bundle artifact를 생성합니다. 로컬 release build, code signing, notarization 및 배포 전 점검은 [Release 문서](release-and-signing.md)를 참고하세요.

Tauri/Vite 구성은 [Tauri Vite 공식 문서](https://v2.tauri.app/start/frontend/vite/)를 따르며, SQL은 frontend에 노출하지 않고 Rust `rusqlite` 경계로 유지합니다.
