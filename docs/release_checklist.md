# Rust Survivors Release Checklist

작성일: 2026-05-23
갱신일: 2026-05-24

배포 준비는 HUD/업적/로컬라이제이션/오디오가 검증된 뒤 진행한다.
현재 목표는 로컬에서 재현 가능한 macOS/Windows 준비 절차를 고정하는 것이다.

## CI 검증

GitHub Actions workflow: `.github/workflows/release-smoke.yml`

주의: workflow 파일은 GitHub default branch에 올라간 뒤에만 `gh workflow run`이나 Actions UI에서 실행할 수 있다. 로컬에만 있는 상태에서는 GitHub API가 `workflow release-smoke.yml not found on the default branch`를 반환한다.

실행/조회:

```bash
gh workflow run release-smoke.yml --ref main
gh run list --workflow release-smoke.yml --limit 5
```

검증 대상:

- `macos-latest`
- `windows-latest`

각 OS에서 실행:

- `cargo fmt --check`
- `cargo test -p game --lib`
- `cargo build -p game --bin survivor`
- `cargo build -p game --bin survivor --release`
- macOS: `scripts/package_macos.sh` (`RustSurvivors` 폴더 + `RustSurvivors.app`)
- Windows: `scripts\package_windows.ps1`
- macOS 검증: `scripts/verify_macos_package.sh`
- Windows 검증: `scripts\verify_windows_package.ps1`
- 각 패키지에 `assets/`, `ASSET_LICENSES.md`, `audio_assets.md` 포함
- 각 패키지에 `PACKAGE_MANIFEST.sha256` 생성
- CI에서 `PACKAGE_MANIFEST.sha256` 해시 검증
- 패키지 폴더 artifact 업로드

## 공통 검증

```bash
cargo fmt
cargo test -p game --lib
cargo build -p game --bin survivor
cargo build -p game --bin survivor --release
```

2026-05-24 자동 검증 결과:

- `cargo fmt`: 통과
- `cargo test -p game --lib`: 통과, 96 passed
- `cargo build -p game --bin survivor`: 통과
- `cargo build -p game --bin survivor --release`: 통과
- macOS 임시 패키지 폴더 `dist/macos/RustSurvivors` 생성 확인
- macOS `.app` 번들 `dist/macos/RustSurvivors.app` 생성 확인
- `dist/macos/RustSurvivors`와 `.app` 내 `._*` / `.DS_Store` 제거 확인
- macOS 패키지에 `ASSET_LICENSES.md`, `audio_assets.md` 포함 확인
- macOS 패키지에 `PACKAGE_MANIFEST.sha256` 생성 확인
- macOS 패키지를 `scripts/verify_macos_package.sh`로 검증
- `scripts/package_macos.sh` 실행 확인
- 현재 로컬 Rust target: `aarch64-apple-darwin`만 설치됨. Windows 산출물은 Windows 환경에서 확인 필요
- Windows 산출물 검증 명령: `scripts\verify_windows_package.ps1`
- Windows 산출물 실제 검증 경로: GitHub Actions `release-smoke`의 `windows-latest game checks`

2026-05-23 GUI smoke 결과:

- packaged macOS binary 실행 시 Title 창 표시 확인
- Settings 이후 키보드 조작 자동화는 macOS Accessibility 권한 제한으로 미확인
- 남은 수동 항목은 `docs/manual_qa_checklist.md` 기준으로 사람이 직접 확인

수동 확인:

- 상세 항목은 [manual_qa_checklist.md](manual_qa_checklist.md)를 따른다.
- `cargo run -p game --bin survivor --release`, packaged folder, 또는 `dist/macos/RustSurvivors.app`에서 Title 진입
- Settings에서 언어, HUD 정보량, BGM/SFX 볼륨 변경 후 저장 확인
- 800x600 해상도에서 HUD, LevelUp, Shop, CharacterSelect, StageSelect 텍스트 겹침 확인
- `assets/audio` 파일이 있을 때 BGM/SFX 재생 확인
- 보스 등장 시 `bgm_boss` 전환과 `sfx_boss_appear` 재생 확인
- 보물상자 획득 시 `sfx_chest_open` 재생 확인
- 오디오 장치 초기화 실패 시 게임 루프가 계속 동작하는지 확인

## macOS

1. 패키징 스크립트 실행:

   ```bash
   scripts/package_macos.sh
   ```

   이 스크립트는 마지막에 `scripts/verify_macos_package.sh`를 호출해 필수 파일, `Info.plist`, macOS metadata 파일, manifest hash를 함께 검증한다.

2. 스크립트 산출물:

   - 폴더 배포: `dist/macos/RustSurvivors`
   - 앱 번들: `dist/macos/RustSurvivors.app`
   - 자산/라이선스 문서: `assets/`, `ASSET_LICENSES.md`, `audio_assets.md`
   - 파일 검증 manifest: `PACKAGE_MANIFEST.sha256`

3. `.app` 구조:

   ```text
   RustSurvivors.app/
   └── Contents/
       ├── Info.plist
       ├── MacOS/
       │   ├── RustSurvivors      # launcher: Resources 로 cd 후 survivor-bin 실행
       │   └── survivor-bin       # 실제 Rust binary
       └── Resources/
           └── assets/
   ```

4. 또는 수동으로 release binary 생성: `cargo build -p game --bin survivor --release`
5. 임시 패키지 폴더 생성:

   ```bash
   mkdir -p dist/macos/RustSurvivors
   cp target/release/survivor dist/macos/RustSurvivors/
   cp -R assets dist/macos/RustSurvivors/
   find dist/macos/RustSurvivors \( -name '._*' -o -name '.DS_Store' \) -delete
   ```

6. `dist/macos/RustSurvivors`에서 실행:

   ```bash
   cd dist/macos/RustSurvivors
   ./survivor
   ```

7. 패키지를 별도로 재검증:

   ```bash
   scripts/verify_macos_package.sh
   ```

8. `.app` 직접 실행:

   ```bash
   open dist/macos/RustSurvivors.app
   ```

## Windows

Windows 산출물은 Windows 환경 또는 cross toolchain에서 만든다.

권장 절차:

1. Windows에서 repo checkout
2. Rust stable 설치
3. PowerShell에서 패키징 스크립트 실행:

   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts\package_windows.ps1
   ```

   이 스크립트는 마지막에 `scripts\verify_windows_package.ps1`를 호출해 필수 파일, macOS metadata 파일, manifest hash를 함께 검증한다.

4. 패키지를 별도로 재검증:

   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts\verify_windows_package.ps1
   ```

5. 또는 수동으로 `cargo build -p game --bin survivor --release`
6. `target/release/survivor.exe`와 `assets/`를 같은 배포 폴더에 복사
7. `ASSET_LICENSES.md`, `audio_assets.md`가 배포 폴더에 포함되는지 확인
8. `PACKAGE_MANIFEST.sha256`가 생성되는지 확인
9. `PACKAGE_MANIFEST.sha256`의 각 항목이 실제 파일 SHA-256과 일치하는지 확인
10. `._*`, `.DS_Store`가 배포 폴더에 남지 않는지 확인
11. 실행 위치 기준 상대 경로 `assets/...`가 정상 로드되는지 확인

## Asset Path

현재 게임 코드는 개발/배포 모두 실행 작업 디렉터리 기준 `assets/...` 상대 경로를 사용한다.
폴더 배포물은 binary와 같은 작업 디렉터리 아래에 `assets/`를 포함한다.
macOS `.app`은 launcher가 `Contents/Resources`로 `cd`한 뒤 실제 binary를 실행하므로 `Contents/Resources/assets`를 같은 방식으로 읽는다.

## 제외 항목

- `target/`
- `.git/`
- `docs/._*`, `assets/**/._*`
- 개발용 save 파일
- 출처/라이선스가 정리되지 않은 외부 자산

## 라이선스

[ASSET_LICENSES.md](ASSET_LICENSES.md)는 현재 포함된 placeholder 자산과 교체 예정 외부 자산의 라이선스 추적 기준 문서다. 외부 자산으로 교체하면 출처 URL, 라이선스, 저작자 표기 요구사항을 추가한다.
