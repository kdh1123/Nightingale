# Nightingale

Nightingale은 Windows 11과 macOS용 방어 중심 보안 모니터링 애플리케이션입니다. 상용 백신을 대체하지 않으며, 위험 징후를 이해하기 쉬운 설명으로 알려주는 것을 목표로 합니다.

## 현재 단계 — Phase 0

작동하는 Tauri + React 기반, 앱 상태 확인 command, capability 모델, SQLite 초기화/마이그레이션, 기본 정책 모델, 기본 화면과 테스트만 구현했습니다. 시스템/프로세스 수집, 파일 감시, 랜섬웨어 탐지, 무결성 검사, 피싱 판정, 비밀번호 보관함은 구현하지 않았습니다.

## 개발

필수 도구: Node.js 22+ 및 공식 rustup의 Rust stable (1.77.2 이상), macOS는 Xcode Command Line Tools, Windows는 MSVC Build Tools와 WebView2. npm을 사용합니다. macOS에서 전체 Xcode는 이 Phase 0의 컴파일·개발 실행에 필요하지 않았습니다.

```sh
npm install
npm run tauri dev
npm run typecheck && npm run lint && npm run test && npm run build
cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo check
```

rustup 설치 뒤에는 새 셸을 열거나 `. "$HOME/.cargo/env"`로 Cargo 환경을 반영합니다. `npm run tauri dev`로 개발 앱을 실행하고 `npm run tauri build`로 macOS 앱 번들을 만듭니다. 이 환경에서는 `.app` 번들 생성까지 확인했으며, Finder를 조작하는 DMG 레이아웃 단계는 GUI 자동화 대기로 완료되지 않았습니다.

개발 중에는 사용자의 실제 파일을 감시하거나 변경하지 않습니다. 자세한 구조와 보안·플랫폼 제약은 [docs](docs/)를 참고하세요. 기술 선택은 [기술 결정](docs/technology-decisions.md)에 기록했습니다.
