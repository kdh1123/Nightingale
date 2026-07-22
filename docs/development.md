# 개발·배포 메모

macOS: Xcode Command Line Tools, Rust stable, Node.js를 설치합니다. Windows: Rust MSVC target, Visual Studio C++ Build Tools, WebView2가 필요합니다. `npm install`, `npm run tauri dev`로 실행하고 README의 검사 명령을 사용합니다.

배포 전에는 macOS notarization과 Windows 코드 서명, 업데이트 서명·검증, 권한 설명, Windows CI 빌드를 추가로 설계해야 합니다. Tauri/Vite 구성은 [Tauri Vite 공식 문서](https://v2.tauri.app/start/frontend/vite/)를 따르며, SQL은 frontend에 노출하지 않고 Rust `rusqlite` 경계로 유지합니다.
