# Image Based UI Pass

작성일: 2026-05-28

인게임과 메뉴 UI를 텍스트/단색 사각형 중심에서 이미지 기반 UI로 옮긴 작업 기록이다.
목표는 화면의 첫인상과 슬롯/카드/패널의 시각 밀도를 높이되, 동적 수치와 KO/EN 로컬라이즈 텍스트는 엔진 텍스트 렌더링으로 유지하는 것이다.

## 범위

완료:

- Title 메뉴 이미지 UI
- InGame 상단 HUD 프레임
- 보스 HP 바 프레임
- 무기/패시브 장비 슬롯 프레임
- Level-up 카드 패널/row 프레임
- Shop 선택 row와 전체 패널
- CharacterSelect, StageSelect, PauseMenu, Settings, StageClear, GameOver 패널

의도적으로 텍스트 유지:

- HP/XP/Gold/Kills/Time/Level 같은 동적 수치
- KO/EN 로컬라이즈 라벨
- Level-up 카드 설명
- Shop/Settings/Achievement 텍스트
- 데미지 숫자

이유:

- 생성 이미지 안의 텍스트는 오탈자와 언어 전환 문제가 크다.
- 현재 엔진의 `TextQueue`가 로컬라이즈와 동적 수치를 안정적으로 처리한다.
- 완전한 이미지 폰트/숫자 아틀라스는 별도 시스템 작업이 필요하다.

## 생성 에셋

### Title

저장 위치: `assets/textures/survivor/menu/`

| 파일 | 용도 |
|---|---|
| `title_backdrop_v2.png` | 새 타이틀 풀스크린 배경 |
| `title_logo_plaque.png` | 제목 텍스트 뒤 장식 받침 |
| `menu_button_start.png` | Start 버튼 장식 |
| `menu_button_character.png` | Character 버튼 장식 |
| `menu_button_stage.png` | Stage 버튼 장식 |
| `menu_button_shop.png` | Shop 버튼 장식 |
| `menu_button_settings.png` | Settings 버튼 장식 |

상세 문서: `docs/TITLE_MENU_IMAGE_UI.md`

### InGame / Modal

저장 위치: `assets/textures/survivor/ui/`

| 파일 | 용도 |
|---|---|
| `ui_modal_panel.png` | 큰 모달/결과/설정/선택 화면 패널 |
| `ui_slot_frame.png` | HUD, 장비 슬롯, 카드 row, 선택 row, 보스바 프레임 |

생성 방식:

- `image_gen` built-in tool 사용.
- 투명 처리가 필요한 UI 프레임은 `#00ff00` 크로마키 배경으로 생성.
- 로컬 Node 기반 임시 PNG 변환으로 녹색 배경 제거, 알파 PNG 저장, 프레임 크롭 처리.
- 최종 에셋 SHA-256은 `docs/ASSET_LICENSES.md`에 기록.

대표 프롬프트 의도:

- 텍스트 없는 dark fantasy/gothic roguelite UI panel
- 텍스트 없는 compact slot frame
- front-facing orthographic game UI asset
- aged brass, dark iron, dark wood, red gemstone accents
- no text, no numbers, no logo, no watermark

## 코드 연결

| 파일 | 변경 내용 |
|---|---|
| `crates/game/src/survivor/sprites.rs` | UI 이미지 경로 상수와 `SurvivorTextureHandles` 필드 추가 |
| `crates/game/src/bin/survivor.rs` | 새 UI 이미지를 시작 시 `app.load_image(...)`로 로드 |
| `crates/game/src/survivor/mod.rs` | UI 이미지 경로 상수 재수출 |
| `crates/game/src/survivor/title_visual.rs` | Title logo/button 이미지를 `UiImageQueue`로 제출 |
| `crates/game/src/survivor/ui_icons.rs` | HUD/Level-up/Shop 배경과 아이콘을 `UiImageQueue` 이미지로 제출 |
| `crates/game/src/survivor/hud.rs` | 주요 메뉴/패널/HUD 프레임을 이미지 기반으로 교체 |
| `docs/ASSET_LICENSES.md` | 새 생성 에셋 출처와 해시 기록 |
| `docs/manual_qa_checklist.md` | 이미지 UI 수동 QA 항목 추가 |

## 렌더링 설계

- `UiImageQueue`의 screen-space `DrawImage`를 사용한다.
- 텍스처는 `DrawImage::textured_with_handle(...)`로 제출하고, 테스트/폴백을 위해 path도 유지한다.
- 동적 바(HP, XP, 보스 HP)는 `DrawImage::colored(...)`로 같은 image pass에 제출해 이미지 프레임과 z 정렬을 맞춘다.
- 텍스트는 계속 `TextQueue`로 제출한다. 엔진 렌더 순서상 텍스트가 이미지 UI 위에 온다.
- 기존 title hitbox와 mode transition 입력은 유지한다.

장비 슬롯 변경:

- 기존: 아이콘 + 장비명 + 레벨 + 빈 슬롯 텍스트
- 현재: 아이콘 + `Lv` 중심 표기
- 빈 슬롯은 프레임만 남겨 장비 보유 여부가 시각적으로 드러나게 했다.

## 검증

통과한 자동 검증:

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
- macOS folder package와 `.app` package 모두 새 `assets/textures/survivor/ui/*.png` 포함 확인

## 확인한 제한

- 초기 macOS `screencapture` 결과가 검은 화면으로만 저장되어 신뢰 가능한 시각 smoke 자료로 쓰지 않았다.
- 이후 잠금 해제된 실제 화면에서 `Computer Use`로 패키지 앱 smoke를 수행했다.
- 잠금 화면 해제, 비밀번호/Touch ID 입력, 화면 잠금 우회는 에이전트가 수행하지 않는다.

## 추가 QA 결과

상세 기록: `docs/VISUAL_QA_BACKING_FIX.md`

2026-05-28 실제 앱 smoke에서 발견한 문제:

- Title/menu/UI frame 이미지의 투명 영역이 흰 사각형으로 보임.
- Settings 하단 도움말이 프레임 장식과 겹쳐 읽기 어려움.

수정:

- Title logo/button과 공통 modal/slot frame 이미지를 제출하기 전에 불투명한 어두운 backing `DrawImage::colored(...)`를 먼저 큐에 넣었다.
- Settings 도움말 문구를 프레임 밖 아래쪽으로 이동하고 글자 크기를 줄였다.
- 2026-05-29 로고풍 텍스트 투명 PNG 패스에서 Title logo/button용 backing은 제거되었다. 공통 modal/slot frame backing은 유지한다.

확인:

- 기본 해상도와 800x600에서 Title 이미지 UI가 텍스트와 겹치지 않는지 확인.
- InGame에서 HP/XP/상단 HUD 프레임과 텍스트가 겹치지 않는지 확인.
- `B` boss spawn 후 보스 HP 바 프레임, fill, 라벨이 모두 보이는지 확인.
- Level-up 카드 3개가 프레임 안에서 아이콘/텍스트와 겹치지 않는지 확인.
- Shop에서 power-up icon, 선택 row, 업적 요약이 서로 겹치지 않는지 확인.
- Settings 패널의 텍스트와 하단 도움말이 프레임과 겹치지 않는지 확인.
- QA 중 800x600으로 변경한 해상도는 1280x720 기본값으로 되돌렸다.

당시 남은 수동 QA 메모 (비활성, 현재 계획은 `docs/NEXT_WORK_PLAN.md` 참조):

- 실제 스피커 출력 기준 BGM/SFX 청취.
- 장시간 플레이 기반 StageClear.
- KO/EN 언어 전환 후 긴 문자열이 버튼/패널 밖으로 나가지 않는지 확인.

## 당시 후속 메모 (비활성)

## 2026-05-29 InGame Static Text Image Follow-up

상세 문서: `docs/INGAME_STATIC_TEXT_IMAGE_UI.md`

추가 완료:

- 상단 HUD 고정 라벨 `Lv/HP/XP/Gold/Kills/Passives`를 언어별 PNG 이미지로 교체했다.
- 장비 섹션 `Weapons/Passives` 라벨을 언어별 PNG 이미지로 교체했다.
- 레벨업 제목, 게임오버 제목, 재시작 안내를 언어별 PNG 이미지로 교체했다.
- 시간, HP/XP/Gold/Kills/Passives 수치, 슬롯 `Lv N`, 카드명, 결과 통계는 텍스트로 유지했다.
- 새 PNG 자산은 `SurvivorTextureHandles`로 로드하고, `MetaSave::effective_lang()` 기준으로 `{ko,en}` 경로를 선택한다.

검증:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

결과:

- game lib tests: 157 passed
- release survivor build 통과
- macOS folder package와 `.app` package 모두 새 ingame static text PNG 포함 확인

## 당시 후속 메모 (비활성)

1. 선택/hover/focus 상태용 프레임 변형 이미지를 추가한다.
2. 숫자 전용 bitmap font 또는 digit atlas를 검토해 HP/XP/Gold 같은 수치를 더 이미지 기반으로 전환한다.
3. Settings/Shop/Achievement 화면은 텍스트 밀도가 높으므로 별도 레이아웃 정리를 한다.
4. 최종 출시 전 placeholder 생성 에셋을 사용 가능한 최종 라이선스 에셋으로 교체한다.
