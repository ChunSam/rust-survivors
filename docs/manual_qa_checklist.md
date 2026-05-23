# Rust Survivors Manual QA Checklist

작성일: 2026-05-24

자동 검증은 통과했지만, wgpu 창에서만 확인 가능한 항목은 수동 QA로 남긴다.
각 항목은 문제가 있으면 증상, 화면, 누른 키, 대략 시간을 기록한다.

## 2026-05-23 Smoke Result

- `scripts/package_macos.sh`로 만든 `dist/macos/RustSurvivors/survivor` 실행 확인.
- Title 창 표시 확인: 배경 이미지, `RUST SURVIVORS`, `Press ENTER to start`, 메뉴 도움말, 메타 정보가 보임.
- 화면 캡처 파일: `/tmp/rust_survivors_title.png`
- 자동 키 입력으로 Settings 진입을 시도했으나 macOS Accessibility 권한 제한으로 `System Events` keystroke가 차단됨. Settings 이후 항목은 사람이 직접 확인해야 함.

## 2026-05-24 Package Smoke Result

- `scripts/package_macos.sh`로 `dist/macos/RustSurvivors`와 `dist/macos/RustSurvivors.app` 생성 확인.
- `scripts/verify_macos_package.sh`로 `PACKAGE_MANIFEST.sha256` 해시 검증 확인.
- `ASSET_LICENSES.md`와 `audio_assets.md`가 폴더 패키지와 `.app` 번들에 포함됨.
- `.github`, `docs`, `scripts`, `assets`, `dist/macos` 내 `._*` / `.DS_Store` 없음.
- GitHub Actions `release-smoke` run `26340730147`에서 macOS/Windows package와 manifest 검증 통과.
- 다운로드한 `RustSurvivors-windows` artifact의 `PACKAGE_MANIFEST.sha256` 로컬 재검증 통과.
- Windows artifact 실행 smoke는 실제 Windows GUI 환경에서 사람이 확인해야 함.

## 실행

```bash
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
cd dist/macos/RustSurvivors
./survivor
```

macOS 앱 번들 실행:

```bash
open dist/macos/RustSurvivors.app
```

개발 실행:

```bash
cargo run -p game --bin survivor --release
```

## Title / Settings

- Title 배경 이미지가 보이고 제목/메뉴 텍스트가 배경 위에서 읽힌다.
- `O`로 Settings에 진입한다.
- 언어를 System/한국어/English로 바꾸면 화면 문자열이 즉시 바뀐다.
- HUD 정보량 최소/중간/상세가 저장되고 게임 화면에 반영된다.
- BGM/SFX 볼륨 0~100% 변경이 저장된다.
- 해상도 800x600 적용 후 텍스트가 화면 밖으로 나가지 않는다.

## InGame HUD

- 800x600에서 HP/XP/Level/Time/Kills가 겹치지 않는다.
- 무기 슬롯 6칸과 패시브 슬롯 6칸이 두 줄로 고정 표시된다.
- 장비명/레벨/빈 슬롯 상태가 구분된다.
- 상세 HUD에서 공격력/쿨다운/범위/투사체 수/이동속도 줄이 상단 UI와 겹치지 않는다.

## Menu Screens

- CharacterSelect에서 해금/잠김 상태가 읽힌다.
- StageSelect에서 잠긴 스테이지와 선택 커서가 구분된다.
- Shop에서 파워업 목록과 업적 요약이 서로 겹치지 않는다.
- PauseMenu에서 Resume/Return to Title/Quit 선택이 정상 동작한다.

## Gameplay

- 1분 이상 생존 중 플레이어, 적, XP, 픽업, 투사체가 스프라이트로 보인다.
- 첫 레벨업 카드 3장이 카드 박스 안에서 읽힌다.
- `1/2/3` 선택 뒤 전투가 재개되고 강화가 반영된다.
- 보스 등장 시 카메라 줌/화면 흔들림이 과하지 않다.
- 데미지 숫자, 히트 플래시, 파티클이 보인다.

## Audio

- `assets/audio/bgm_title.wav`가 Title에서 들린다.
- InGame/Boss/StageClear/GameOver 전환 시 BGM이 바뀐다.
- 보스 등장 시 `sfx_boss_appear`가 재생된다.
- 보물상자 획득 시 `sfx_chest_open`이 재생된다.
- SFX 파일이 없을 때 절차적 톤 fallback이 들린다.
- 오디오 장치 초기화 실패 환경에서도 게임 루프가 멈추지 않는다.

## Save / Meta

- Settings 변경 후 재실행해도 언어/HUD/볼륨/해상도 설정이 유지된다.
- FirstRun 업적은 한 번만 저장된다.
- 업적 보상으로 캐릭터/스테이지/파워업 해금이 중복 적용되지 않는다.
