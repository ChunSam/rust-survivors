# Visual QA Backing Fix

작성일: 2026-05-28

업데이트: 2026-05-29 로고풍 텍스트 투명 PNG 패스에서 Title logo/button PNG는 자체 알파를 사용하도록 바뀌었고, Title 전용 backing 큐잉은 제거되었다. 이 문서는 2026-05-28 당시 문제와 조치 기록으로 유지한다. 공통 `ui_modal_panel.png` / `ui_slot_frame.png` backing은 계속 별도 UI 프레임 안정화 용도로 남아 있다.

실제 macOS 패키지 앱에서 이미지 기반 UI를 확인하며 발견한 투명 영역 렌더링 문제와 후속 검증 기록이다.
이미지 UI 도입의 전체 배경은 `docs/IMAGE_BASED_UI_PASS.md`, 긴 변경 이력은 `docs/PHASE_LOG.md`를 참고한다.

## 목표

- Title/menu/InGame UI 이미지가 실제 앱 화면에서 의도한 색과 레이어 순서로 보이는지 확인한다.
- 800x600과 기본 해상도에서 HUD, 보스바, Level-up, Shop, Settings가 겹치지 않는지 확인한다.
- 자동 테스트로 잡기 어려운 시각 문제를 문서화하고, 남은 수동 QA를 분리한다.

## 발견 이슈

1. Title logo/button, modal panel, slot frame 이미지의 투명 영역이 실제 앱에서 흰 사각형처럼 보였다.
2. Settings 화면 하단 도움말이 장식 프레임과 겹쳐 읽기 어려웠다.

PNG 자체는 알파 채널을 가지고 있었다. 따라서 에셋 누락보다는 `UiImageQueue` 이미지와 macOS/wgpu surface 합성 경로에서 투명 배경이 안정적으로 합성되지 않는 문제로 판단했다.

## 수정 내용

| 파일 | 내용 |
|---|---|
| `crates/game/src/survivor/title_visual.rs` | Title logo/button 이미지를 큐에 넣기 전에 불투명한 어두운 backing 이미지를 먼저 제출한다. |
| `crates/game/src/survivor/title_visual.rs` | `title_menu_images_queue_opaque_backings` 테스트를 추가해 backing 제출을 고정한다. |
| `crates/game/src/survivor/hud.rs` | 공통 modal panel과 slot frame 이미지 앞에 어두운 backing을 추가한다. |
| `crates/game/src/survivor/hud.rs` | Settings 도움말을 모달 프레임 아래로 내리고 글자 크기를 줄인다. |
| `crates/game/src/survivor/ui_icons.rs` | HUD/Level-up/Shop에서 큐잉되는 slot/modal frame texture 앞에 어두운 backing을 추가한다. |
| `docs/manual_qa_checklist.md` | 실제 앱 smoke 결과와 남은 수동 QA를 기록한다. |
| `docs/IMAGE_BASED_UI_PASS.md` | 이미지 UI pass의 제한과 추가 QA 결과를 갱신한다. |
| `docs/NEXT_WORK_PLAN.md` | 단일 예정 작업 계획을 갱신한다. |
| `docs/PHASE_LOG.md` | 2026-05-28 Visual QA Backing Fix Pass를 추가한다. |

## 설계 메모

- 이번 수정은 게임 레이어에서 UI 이미지 뒤에 불투명 backing을 깔아 시각 누수를 막는 접근이다.
- 엔진 API 변경은 하지 않았다. 현재 문제는 특정 UI 에셋과 게임 레이아웃의 합성 안정성 문제로 좁혀 처리할 수 있었다.
- 텍스트는 계속 `TextQueue`에 남겼다. KO/EN 전환, 동적 수치, 카드 설명은 이미지 안에 굽지 않는 것이 안전하다.
- 프레임 이미지는 장식 역할을 하고, 실제 판독성은 backing과 engine text가 보장한다.

## 자동 검증

최종 코드 기준으로 아래 검증을 통과했다.

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo check -p game --all-targets --locked
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

결과:

- game lib tests: 147 passed
- release survivor build 통과
- macOS folder package와 `.app` package 검증 통과

## 수동 QA

패키지 앱 `dist/macos/RustSurvivors.app` 기준으로 확인했다.

확인한 화면:

- 기본 해상도 Title: logo/button 주변 흰 사각형 누수 없음.
- 기본 해상도 InGame: 상단 HUD, HP/XP, 장비 슬롯 프레임 표시 정상.
- 기본 해상도 `B` boss spawn: 보스 HP 바, fill, label 표시 정상.
- 기본 해상도 Level-up: 카드 3개, 아이콘, 텍스트가 프레임과 큰 겹침 없이 표시.
- 기본 해상도 Shop: power-up icon, 선택 row, 업적 요약 표시 정상.
- 기본 해상도 Settings: 하단 도움말이 프레임 장식과 겹치지 않음.
- 800x600 Title/InGame/Boss/Level-up: 흰 사각형 누수 없이 화면 내 배치 확인.

QA 중 800x600으로 바꾼 해상도는 다시 `1280x720 HD 16:9 [기본값]`으로 되돌렸고, 앱은 종료했다.

## 당시 남은 확인 메모 (비활성)

- 실제 스피커 출력 기준 BGM/SFX 청취.
- 장시간 플레이 기반 StageClear 흐름.
- KO/EN 전환 후 긴 문자열이 Title, Settings, Shop, card, result 화면에서 넘치지 않는지 확인.
- Title image button의 mouse hitbox 체감 확인.
- CharacterSelect와 StageSelect 이미지 패널 가독성 확인.
- 디버그 키 `C`/`T`가 현재 의도한 동작과 맞는지 확인.
