# Nightingale

Nightingale은 Windows 11과 macOS용 방어 중심 보안 모니터링 애플리케이션입니다. 상용 백신을 대체하지 않으며, 위험 징후를 이해하기 쉬운 설명으로 알려주는 것을 목표로 합니다.

## 현재 단계 — Phase 2

Phase 0의 기반과 Phase 1의 시스템·프로세스 모니터링을 완료했습니다. Phase 2에서는 사용자가 선택한 폴더의 파일 활동을 재귀적으로 감시하고, 이벤트·기준선 SHA-256 해시를 로컬 SQLite에 기록합니다. 감시 경로는 일시정지·재개·삭제할 수 있으며 앱 재시작 시 활성 경로를 복구합니다. 이 기능은 관찰과 기록만 수행하며 파일을 변경·삭제·차단하지 않습니다.

## 개발

필수 도구: Node.js 22+ 및 공식 rustup의 Rust stable (1.77.2 이상), macOS는 Xcode Command Line Tools, Windows는 MSVC Build Tools와 WebView2. npm을 사용합니다. macOS에서 전체 Xcode는 이 Phase 0의 컴파일·개발 실행에 필요하지 않았습니다.

```sh
npm install
npm run tauri dev
npm run typecheck && npm run lint && npm run test && npm run build
cd src-tauri && cargo fmt --check && cargo check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

rustup 설치 뒤에는 새 셸을 열거나 `. "$HOME/.cargo/env"`로 Cargo 환경을 반영합니다. `npm run tauri dev`로 개발 앱을 실행하고 `npm run tauri build`로 macOS 앱 번들을 만듭니다. 이 환경에서는 `.app` 번들 생성까지 확인했으며, Finder를 조작하는 DMG 레이아웃 단계는 GUI 자동화 대기로 완료되지 않았습니다.

개발 중에는 사용자의 실제 파일을 변경하지 않습니다. 파일 감시는 사용자가 명시적으로 추가한 폴더에서만 시작됩니다. 자세한 구조와 보안·플랫폼 제약은 [docs](docs/)를 참고하세요. 기술 선택은 [기술 결정](docs/technology-decisions.md)에 기록했습니다.

## Phase 2 파일 모니터링 제한

감시를 일시 중지하면 watcher를 해제하므로 그 동안의 변경은 기록하지 않습니다. 재개하거나 앱을 다시 시작하면 활성 경로의 감시를 다시 시도합니다. rename과 metadata 이벤트는 활동으로 기록하지만 이전 경로 비교 또는 기준선 불일치 판정에는 사용하지 않습니다. 이벤트는 같은 경로·유형의 500ms 이내 중복을 억제하고, watcher 전달 버퍼가 가득 차면 최신 이벤트를 버릴 수 있습니다. macOS에서는 보호된 폴더에 대해 사용자가 파일 접근 권한을 허용해야 하며 Windows는 아직 실제 기기에서 검증하지 않았습니다. 자세한 동작과 검증 명령은 [Phase 2 문서](docs/phase2-file-monitoring.md)를 참고하세요.
