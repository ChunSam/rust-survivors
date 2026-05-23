# Rust Survivors Release Checklist

작성일: 2026-05-23

배포 준비는 HUD/업적/로컬라이제이션/오디오가 검증된 뒤 진행한다.
현재 목표는 로컬에서 재현 가능한 macOS/Windows 준비 절차를 고정하는 것이다.

## 공통 검증

```bash
cargo fmt
cargo test -p game --lib
cargo build -p game --bin survivor
cargo build -p game --bin survivor --release
```

2026-05-23 자동 검증 결과:

- `cargo fmt`: 통과
- `cargo test -p game --lib`: 통과, 89 passed
- `cargo build -p game --bin survivor`: 통과
- `cargo build -p game --bin survivor --release`: 통과
- macOS 임시 패키지 폴더 `dist/macos/RustSurvivors` 생성 확인
- `dist/macos/RustSurvivors` 내 `._*` / `.DS_Store` 제거 확인
- `scripts/package_macos.sh` 실행 확인
- 현재 로컬 Rust target: `aarch64-apple-darwin`만 설치됨. Windows 산출물은 Windows 환경에서 확인 필요

수동 확인:

- `cargo run -p game --bin survivor --release`로 Title 진입
- Settings에서 언어, HUD 정보량, BGM/SFX 볼륨 변경 후 저장 확인
- 800x600 해상도에서 HUD, LevelUp, Shop, CharacterSelect, StageSelect 텍스트 겹침 확인
- `assets/audio` 파일이 있을 때 BGM/SFX 재생 확인
- 오디오 장치 초기화 실패 시 게임 루프가 계속 동작하는지 확인

## macOS

1. 패키징 스크립트 실행:

   ```bash
   scripts/package_macos.sh
   ```

2. 또는 수동으로 release binary 생성: `cargo build -p game --bin survivor --release`
3. 임시 패키지 폴더 생성:

   ```bash
   mkdir -p dist/macos/RustSurvivors
   cp target/release/survivor dist/macos/RustSurvivors/
   cp -R assets dist/macos/RustSurvivors/
   find dist/macos/RustSurvivors \( -name '._*' -o -name '.DS_Store' \) -delete
   ```

4. `dist/macos/RustSurvivors`에서 실행:

   ```bash
   cd dist/macos/RustSurvivors
   ./survivor
   ```

5. `.app` 번들은 별도 패키징 스크립트를 만들 때 `Contents/MacOS/survivor`와 `Contents/Resources/assets` 구조로 정리한다.

## Windows

Windows 산출물은 Windows 환경 또는 cross toolchain에서 만든다.

권장 절차:

1. Windows에서 repo checkout
2. Rust stable 설치
3. PowerShell에서 패키징 스크립트 실행:

   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts\package_windows.ps1
   ```

4. 또는 수동으로 `cargo build -p game --bin survivor --release`
5. `target/release/survivor.exe`와 `assets/`를 같은 배포 폴더에 복사
6. 실행 위치 기준 상대 경로 `assets/...`가 정상 로드되는지 확인

## Asset Path

현재 게임 코드는 개발/배포 모두 실행 작업 디렉터리 기준 `assets/...` 상대 경로를 사용한다.
따라서 배포물은 binary와 같은 작업 디렉터리 아래에 `assets/`를 포함해야 한다.

## 제외 항목

- `target/`
- `.git/`
- `docs/._*`, `assets/**/._*`
- 개발용 save 파일
- 출처/라이선스가 정리되지 않은 외부 자산

## 라이선스

출시 전 [ASSET_LICENSES.md](ASSET_LICENSES.md)의 TODO를 모두 제거한다.
