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
- GitHub Actions `release-smoke`는 macOS 전용 검증으로 운영한다.

## 2026-05-25 Verification Result

- `cargo fmt --check` 통과.
- `cargo test -p game --lib -- --test-threads=1` 통과: 98 passed.
- `cargo build -p game --bin survivor --release` 통과.
- `scripts/package_macos.sh` 통과: `dist/macos/RustSurvivors`, `dist/macos/RustSurvivors.app` 생성 및 내부 검증 완료.
- `scripts/verify_macos_package.sh` 통과.
- `open dist/macos/RustSurvivors.app`로 패키지 앱 실행 확인.
- Title 화면 확인: 배경 이미지, 한글 제목, 시작 버튼, Character/Stage/Shop/Settings 버튼 표시.
- Settings 화면 확인: `O` 진입, 한글 전환, 800x600 해상도 적용, 텍스트 화면 내 표시.
- InGame 800x600 HUD 확인: HP/XP/Level/Kills와 무기/패시브 슬롯 2줄 표시, 큰 겹침 없음.
- PauseMenu 800x600 확인: Resume/Return to Title/Quit 표시, 텍스트 겹침 없음.
- Shop 800x600 확인: 파워업 목록과 업적 요약 표시, 상호 겹침 없음.
- 미확인: 실제 스피커 출력 기준 BGM/SFX 청취, 장시간 플레이 기반 GameOver/StageClear.

## 2026-05-25 Additional Manual QA

- 저장 파일 확인: `~/Library/Application Support/rust-vampire-survivors/save.ron`에 Settings 변경값 저장 확인.
- 재실행 유지 확인: `language_setting: Ko`, `resolution_key: "800x600"` 로드 후 Title 화면이 한글/800x600으로 표시됨.
- 보스 smoke 확인: InGame에서 `B` 입력 시 GiantSlime 보스 소환, 보스 HP 바 표시, 보스 줌 동작 확인.
- GameOver 확인: 보스 접촉으로 HP 0 도달 시 사망 패널과 `R 키로 재시작` 표시.
- Restart 확인: GameOver 상태에서 `R` 입력 시 새 런으로 재시작되고 HUD가 초기화됨.
- 발견 이슈: 800x600에서 보스 카메라 줌아웃 시 우측에 큰 보라색 빈 영역이 노출됨. 배경/타일 렌더 범위가 줌아웃 viewport를 충분히 덮지 못하는 것으로 보임.
- 미확인 유지: 실제 스피커 출력 기준 BGM/SFX 청취, 장시간 플레이 기반 StageClear.

## 2026-05-25 Regression Test Added

- 800x600 보스 줌아웃 배경 빈 영역 이슈를 `background_tiles_cover_800x600_boss_zoom_view` 단위 테스트로 고정.
- `BackgroundSystem` 타일 개수 계산을 `viewport / camera.zoom` 기준으로 변경.
- 검증: `cargo fmt --check`, `cargo test -p game --lib -- --test-threads=1`, `cargo build -p game --bin survivor --release` 통과.

## 2026-05-25 Camera / UI Regression

- 보스 소환 시 플레이어가 화면 중앙에서 벗어나는 이슈를 `camera_follow_keeps_player_centered_when_boss_zoom_is_active` 테스트로 고정.
- 카메라 top-left 계산을 `viewport / (2 * zoom)` 기준으로 변경하고, 서바이버 카메라는 플레이어 중앙 고정으로 동작하게 수정.
- UI scale을 저장된 해상도 값이 아니라 실제 `ViewportSize` 기준으로 계산하도록 변경.
- 데미지 숫자 월드→화면 변환에 camera zoom 반영.
- 패키지 앱 smoke: Title → InGame → `B` 보스 소환 후 플레이어 중앙 유지와 보스 HP 바 표시 확인.

## 2026-05-25 Sprite Size / UV Regression

- 플레이어/적 스프라이트의 투명 여백을 UV에서 잘라내고 원본 가로세로 비율을 보존하도록 수정.
- Retina 환경에서 실제 화면상 약 150px 수준으로 보이도록 플레이어/적/보스 기준 크기를 상향.
- 엔진 quad의 UV 방향 차이로 상하 반전되던 atlas sprite와 title 배경에 vertical flipped UV를 적용.
- 패키지 앱 smoke: Title 배경이 정방향으로 보이고, InGame 플레이어/좀비가 커진 크기와 정방향으로 표시되는 것 확인.

## 2026-05-25 Combat Sprite Pass

- `assets/textures/survivor/survivor_effects.png` 추가: Whip slash, Fireball, Cross projectile, HolyWater pool, HolyBook, Lightning strike, Garlic aura, Impact spark.
- `assets/textures/survivor/survivor_actor_frames.png` 추가: 플레이어/적 이동 3프레임, 피격, 사망 pose.
- `assets/textures/survivor/survivor_evolutions.png` 추가: 진화 무기 아이콘 8종.
- `assets/textures/survivor/survivor_passives.png` 추가: 패시브 아이콘 16종.
- `assets/textures/survivor/survivor_powerups.png` 추가: 파워업 아이콘 19종.
- Whip 발화 시 짧은 slash effect가 생성되도록 연결.
- Whip 좌/우 방향 slash effect가 별도 스프라이트로 표시되도록 연결.
- FireWand/Cross/HolyWater/KingBible/Lightning/Garlic이 기존 아이콘 재사용 대신 전용 effect sprite를 사용하도록 변경.
- 패키지 앱 smoke: 새 effect atlas가 패키지 manifest에 포함되고 InGame sprite 렌더가 정상 유지되는 것 확인.

## 2026-05-25 Responsive Icon Layout Pass

- HUD/LevelUp/Shop icon 위치 계산을 `UiIconSystem` 레이아웃 헬퍼로 분리.
- 800x600에서 HUD 슬롯 아이콘이 장비 텍스트를 침범하지 않는지 단위 테스트로 고정.
- LevelUp 카드 아이콘과 Shop 파워업 아이콘이 텍스트 시작 여백을 침범하지 않는지 단위 테스트로 고정.
- 고해상도에서 HUD/LevelUp/Shop 아이콘 크기가 과도하게 커지지 않도록 상한을 적용.

## 2026-05-25 Playtest Fix Pass

- Settings 해상도 변경 시 `PendingResize`뿐 아니라 `ViewportSize`와 `WindowConfig`도 즉시 갱신하도록 보강.
- 해상도 항목에서 좌/우로 값을 변경하는 순간에도 resize 요청을 넣어 Enter 적용 누락을 방지.
- PauseMenu/StageClear에서 Title로 돌아갈 때는 새 Player를 즉시 스폰하지 않는 `reset_to_title_world` 경로로 변경.
- Title 상태에 Player가 남아 CameraFollow가 메뉴 카메라를 움직이던 재진입 리스크를 테스트로 고정.
- 800x600 compact Title에서 제목이 잘리지 않도록 타이틀 글자 크기와 추정 폭을 축소.

## 2026-05-28 skeleton-engine Modernization Pass

- 엔진 의존성은 `skeleton-engine v1.0.0 #78cebcf` 그대로 유지.
- `HitFlash` 만료와 level-up 카드 선택 완료는 sentinel/consumed flag 대신 public ECS removal API를 사용하도록 변경.
- Survivor texture는 시작 시 `app.load_image`로 handle을 등록하고, sprite 생성은 handle 우선 + path fallback 구조로 정리.
- 배경/world/effect/UI sprite에 명시적인 `RenderLayer`를 부여하고, 기존 `Transform.z`는 같은 레이어 안의 정렬값으로 유지.
- 엔진 수정 희망 사항은 `docs/ENGINE_FOLLOWUPS.md`에만 문서화.
- 자동 검증: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (139 passed), `cargo check -p game --all-targets --locked`, `cargo build -p game --bin survivor --release --locked` 통과.
- 수동 smoke 권장: Title backdrop, InGame actor/effect sprites, HUD/level-up/shop icons를 800x600과 기본 해상도에서 확인.

## 2026-05-28 QA Issue Fix Pass

- 발견 이슈: 보스 HP 라벨/바가 기본 해상도에서 상단 HUD 텍스트와 겹침.
- 발견 이슈: HUD/Level-up/Shop 아이콘이 `UiQueue` 배경 rect 뒤에 가려져 색 막대처럼 보이거나 보이지 않음.
- 원인: 엔진 렌더 순서가 world sprite -> `UiQueue` rect -> `TextQueue`라서 `RenderLayer(UI)` 스프라이트도 `UiQueue` rect보다 뒤에 그려짐.
- 수정: 보스 HP 바 위치를 top HUD panel bottom + label padding + label height 기준으로 계산.
- 수정: HUD 슬롯, Level-up dim/panel/card row, Shop 선택 row 배경을 `UiIconSystem`의 screen-space sprite로 전환하고 icon/background를 `Transform.z`로 정렬.
- 수정: Title 화면에 gameplay HUD sprite가 새어 보이지 않도록 HUD slot icon/background spawn을 `SurvivorMode::InGame`으로 제한.
- 자동 검증: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (141 passed), `cargo check -p game --all-targets --locked`, `cargo build -p game --bin survivor --release --locked`, `bash scripts/package_macos.sh` 통과.
- 패키지 앱 smoke: Title HUD 누수 없음, InGame Whip/Passive 아이콘 표시, `B` 보스 소환 시 보스바/HUD 겹침 없음, Level-up 카드 아이콘 표시, Shop power-up 아이콘 표시 확인.
- 미확인 유지: 실제 스피커 출력 기준 BGM/SFX 청취, 장시간 플레이 기반 StageClear.

## 2026-05-28 Engine Migration Pass

- 엔진 의존성을 `skeleton-engine #0e01b0f`로 갱신.
- Survivor sprite UV 생성은 엔진 `UvRect::from_pixels(...).flipped_y()` / `UvRect::from_grid(...).flipped_y()` helper를 사용.
- HUD/Level-up/Shop icon과 관련 배경은 per-frame entity spawn/despawn 대신 `UiImageQueue`의 screen-space `DrawImage`로 제출.
- 자동 검증: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (144 passed), `cargo check -p game --all-targets --locked`, `cargo build -p game --bin survivor --release --locked` 통과.
- 수동 smoke 권장: 기본 해상도와 800x600에서 Title, InGame HUD slots, Level-up cards, Shop icons, pickup/effect sprite 방향 확인.

## 2026-05-28 Title Menu Image UI Pass

- `assets/textures/survivor/menu/`에 AI-generated title backdrop, logo plaque, Start/Character/Stage/Shop/Settings 버튼 이미지를 추가.
- Title 배경은 새 menu backdrop으로 교체하고, 메뉴 장식/버튼은 `UiImageQueue` screen-space image로 제출.
- KO/EN 로컬라이즈 정확도를 위해 메뉴 문구는 엔진 텍스트로 유지하고, 이미지 에셋은 텍스트 없는 장식/아이콘 중심으로 사용.
- 수동 smoke 권장: 기본 해상도와 800x600에서 로고/Start/하단 4버튼 이미지가 텍스트와 겹치지 않는지, 클릭 영역이 기존 버튼 위치와 일치하는지 확인.

## 2026-05-28 InGame Image UI Pass

- `assets/textures/survivor/ui/`에 AI-generated modal panel과 slot frame 이미지를 추가.
- Character/Stage/Shop/Pause/Settings/StageClear/GameOver 패널과 선택 row를 이미지 프레임 중심으로 교체.
- InGame 상단 HUD, 보스 HP, 장비 슬롯, Level-up 카드 row를 `UiImageQueue` 기반 이미지 UI로 표시.
- 장비 슬롯은 이름 텍스트 대신 아이콘 + `Lv` 표기로 축소.
- 수동 smoke 권장: 기본 해상도와 800x600에서 HP/XP/보스바가 프레임 이미지 뒤에 가려지지 않는지, Pause/Settings/Level-up/Shop 텍스트가 프레임 내부에 들어오는지 확인.

## 2026-05-28 Visual QA Backing Fix Pass

- 상세 문서: `docs/VISUAL_QA_BACKING_FIX.md`
- 발견 이슈: 실제 앱 화면에서 Title/menu/UI frame 이미지의 투명 영역이 흰 사각형으로 보임.
- 원인 추정: `UiImageQueue` 이미지의 투명 영역과 macOS/wgpu surface 합성 경로에서 배경 알파가 안정적으로 남지 않는 시각 누수.
- 수정: Title logo/button 이미지와 `ui_modal_panel.png` / `ui_slot_frame.png` 제출 전에 불투명한 어두운 backing `DrawImage::colored(...)`를 먼저 큐에 넣음.
- 수정: Settings 하단 도움말 문구를 모달 프레임 밖 아래쪽으로 내려 장식 프레임과 겹치지 않게 조정.
- 자동 검증: `cargo fmt --check`, `cargo test -p game --lib --locked -- --test-threads=1` (147 passed), `cargo check -p game --all-targets --locked`, `cargo build -p game --bin survivor --release --locked`, `bash scripts/package_macos.sh`, `bash scripts/verify_macos_package.sh` 통과.
- 패키지 앱 smoke: 기본 해상도와 800x600에서 Title, InGame HUD, `B` 보스바, Level-up 카드, Shop, Settings 화면 확인. 흰 사각형 누수 없음.
- QA 중 800x600으로 바꾼 해상도 설정은 1280x720 기본값으로 되돌리고 앱 종료.
- 미확인 유지: 실제 스피커 출력 기준 BGM/SFX 청취, 장시간 플레이 기반 StageClear.

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
- 장비 아이콘/레벨/빈 슬롯 상태가 구분된다.
- 상세 HUD에서 공격력/쿨다운/범위/투사체 수/이동속도 줄이 상단 UI와 겹치지 않는다.

## Menu Screens

- CharacterSelect에서 해금/잠김 상태가 읽힌다.
- StageSelect에서 잠긴 스테이지와 선택 커서가 구분된다.
- Shop에서 파워업 목록과 업적 요약이 서로 겹치지 않는다.
- PauseMenu에서 Resume/Return to Title/Quit 선택이 정상 동작한다.

## Gameplay

- 1분 이상 생존 중 플레이어, 적, XP, 픽업, 투사체가 스프라이트로 보인다.
- 플레이어와 일반 적이 화면에서 충분히 크게 보이고 상하 반전 없이 표시된다.
- 첫 레벨업 카드 3장이 카드 박스 안에서 읽힌다.
- `1/2/3` 선택 뒤 전투가 재개되고 강화가 반영된다.
- 보스 등장 시 카메라 줌/화면 흔들림이 과하지 않다.
- 데미지 숫자, 히트 플래시, 파티클이 보인다.
- 디버그 입력: `H` 테스트 도움말 토글, `F5` Bomb, `F6` Rosary, `B` GiantSlime 보스 소환이 동작한다.

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
