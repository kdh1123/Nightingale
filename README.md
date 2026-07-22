# Nightingale

Nightingale은 Windows 11과 macOS용 방어 중심 보안 모니터링 애플리케이션입니다. 상용 백신을 대체하지 않으며, 위험 징후를 이해하기 쉬운 설명으로 알려주는 것을 목표로 합니다.

## 현재 단계 — Phase 2

Phase 0의 기반과 Phase 1의 시스템·프로세스 모니터링을 완료했습니다. Phase 2에서는 사용자가 선택한 폴더의 파일 활동을 재귀적으로 감시하고, 이벤트·기준선 SHA-256 해시를 로컬 SQLite에 기록합니다. 감시 경로는 일시정지·재개·삭제할 수 있으며 앱 재시작 시 활성 경로를 복구합니다. 이 기능은 관찰과 기록만 수행하며 파일을 변경·삭제·차단하지 않습니다.

## 처음 실행하기

### 필요한 프로그램

- Node.js 22 이상과 npm
- Rust stable 1.77.2 이상 — [rustup](https://rustup.rs/)로 설치
- macOS: Xcode Command Line Tools (`xcode-select --install`)
- Windows 11: Visual Studio C++ Build Tools(MSVC)와 WebView2 Runtime

Rust 설치 뒤에는 새 터미널을 열거나 `. "$HOME/.cargo/env"`를 실행하고, `rustup component add rustfmt clippy`로 검사 도구를 설치합니다. macOS의 전체 Xcode는 개발·빌드에 필요하지 않습니다.

### 설치와 데이터베이스 초기화

```sh
git clone https://github.com/kdh1123/Nightingale.git
cd Nightingale
npm install
```

별도의 데이터베이스 설치나 수동 migration은 필요하지 않습니다. 첫 Tauri 실행 시 운영체제의 앱 데이터 디렉터리에 `nightingale.sqlite3`를 만들고 migration을 자동 적용합니다. 이 파일에는 사용자가 추가한 감시 경로와 이벤트가 보존됩니다.

### 개발 실행

Tauri 앱 전체를 실행하려면 다음 명령을 사용합니다.

```sh
npm run tauri dev
```

`npm run dev`는 React/Vite 개발 서버만 시작합니다. Tauri command가 필요한 파일 모니터링 기능은 `npm run tauri dev`에서만 동작합니다.

### 테스트와 빌드

```sh
npm run typecheck && npm run lint && npm run test && npm run build
cd src-tauri && cargo fmt --check && cargo check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

배포용 앱 번들은 `npm run tauri build`로 생성합니다. macOS에서는 `.app` 번들 생성을 확인했으며, Finder를 조작하는 DMG 레이아웃 단계는 자동화하지 않습니다.

### 자주 발생하는 오류

| 증상                                   | 해결 방법                                                                    |
| -------------------------------------- | ---------------------------------------------------------------------------- |
| `cargo` 또는 `rustc`를 찾을 수 없음    | rustup 설치 후 새 터미널을 열거나 `. "$HOME/.cargo/env"` 실행                |
| `rustfmt` 또는 `clippy` 오류           | `rustup component add rustfmt clippy` 실행                                   |
| macOS 컴파일러·링커 오류               | `xcode-select --install` 실행 후 터미널 재시작                               |
| Windows에서 빌드 실패                  | MSVC C++ Build Tools와 WebView2 Runtime 설치 확인                            |
| 보호된 macOS 폴더를 감시할 수 없음     | 시스템 설정에서 Nightingale에 파일 접근 권한 부여 또는 권한이 있는 폴더 선택 |
| `npm run tauri dev`에서 포트 1420 충돌 | 해당 Vite 프로세스를 종료한 뒤 다시 실행                                     |
| 감시 경로가 중복되었다는 오류          | 기존 목록에서 해당 경로를 삭제하거나 재개                                    |

### 디스크 용량 정리

Rust 빌드 캐시를 삭제하려면 다음 명령을 실행합니다.

```sh
cargo clean --manifest-path src-tauri/Cargo.toml
```

다음 실행 시에는 다시 컴파일되므로 첫 빌드는 시간이 조금 더 걸릴 수 있습니다.

개발 중에는 사용자의 실제 파일을 변경하지 않습니다. 파일 감시는 사용자가 명시적으로 추가한 폴더에서만 시작됩니다. 자세한 구조와 보안·플랫폼 제약은 [docs](docs/)를 참고하세요. 기술 선택은 [기술 결정](docs/technology-decisions.md)에 기록했습니다.

## Phase 2 파일 모니터링 제한

감시를 일시 중지하면 watcher를 해제하므로 그 동안의 변경은 기록하지 않습니다. 재개하거나 앱을 다시 시작하면 활성 경로의 감시를 다시 시도합니다. rename과 metadata 이벤트는 활동으로 기록하지만 이전 경로 비교 또는 기준선 불일치 판정에는 사용하지 않습니다. 이벤트는 같은 경로·유형의 500ms 이내 중복을 억제하고, watcher 전달 버퍼가 가득 차면 최신 이벤트를 버릴 수 있습니다. macOS에서는 보호된 폴더에 대해 사용자가 파일 접근 권한을 허용해야 하며 Windows는 아직 실제 기기에서 검증하지 않았습니다. 자세한 동작과 검증 명령은 [Phase 2 문서](docs/phase2-file-monitoring.md)를 참고하세요.
