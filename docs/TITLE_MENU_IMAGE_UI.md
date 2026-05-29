# Title Menu Image UI Pass

작성일: 2026-05-28

메인 메뉴를 텍스트와 단색 사각형 중심 UI에서 이미지 기반 UI로 전환한 작업 기록이다.

## 목표

- 타이틀 화면의 첫인상을 강화한다.
- 기존 `Title` 입력 흐름과 클릭 히트박스는 유지한다.
- 한국어/영어 로컬라이제이션 정확도를 위해 메뉴 문구는 엔진 텍스트로 유지한다.
- 생성 이미지는 텍스트 없는 배경, 로고 받침, 버튼 프레임, 기능 아이콘 중심으로 사용한다.

## 생성 에셋

저장 위치: `assets/textures/survivor/menu/`

| 파일 | 용도 |
|---|---|
| `title_backdrop_v2.png` | 새 타이틀 풀스크린 배경 |
| `title_logo_plaque.png` | 제목 텍스트 뒤 로고 받침 |
| `menu_button_start.png` | 큰 시작 버튼 장식 |
| `menu_button_character.png` | Character 버튼 장식 + 인물 아이콘 |
| `menu_button_stage.png` | Stage 버튼 장식 + 지도 아이콘 |
| `menu_button_shop.png` | Shop 버튼 장식 + 상자/코인 아이콘 |
| `menu_button_settings.png` | Settings 버튼 장식 + 기어 아이콘 |

생성 방식:

- `image_gen` built-in 도구로 새 래스터 이미지를 생성했다.
- 배경은 그대로 PNG로 저장했다.
- 로고/버튼 이미지는 `#00ff00` 크로마키 배경으로 생성한 뒤, 로컬 임시 Rust 도구로 녹색 배경 제거와 알파 기준 크롭을 처리했다.
- 2026-05-29 로고풍 텍스트 재작업에서 기존 로고의 금속 세리프/고딕 프레임 느낌을 기준으로 `menu_button_*_{ko,en}.png`를 다시 생성했다.
- 먼저 `menu_button_start_ko.png` 샘플을 사용자 확인받은 뒤 같은 톤으로 나머지 버튼에 확장했다.
- `scripts/make_text_ui_transparent.py`로 로고 받침과 메뉴 버튼 PNG의 캔버스 배경을 투명 처리했다.
- 최종 에셋 SHA-256은 `docs/ASSET_LICENSES.md`에 기록했다.

## 코드 연결

| 파일 | 변경 내용 |
|---|---|
| `crates/game/src/survivor/sprites.rs` | 메뉴 이미지 경로 상수 추가, `SurvivorTextureHandles`에 새 핸들 필드 추가 |
| `crates/game/src/bin/survivor.rs` | 새 메뉴 이미지들을 시작 시 `app.load_image(...)`로 로드 |
| `crates/game/src/survivor/title_visual.rs` | `UiImageQueue`로 로고 받침과 버튼 이미지를 screen-space에 제출 |
| `crates/game/src/survivor/hud.rs` | 새 버튼 이미지 위에 맞도록 타이틀 메뉴 텍스트 크기와 위치 조정 |
| `crates/game/src/survivor/mod.rs` | 새 메뉴 이미지 경로 상수 재수출 |
| `docs/ASSET_LICENSES.md` | 새 생성 에셋 출처와 해시 기록 |
| `docs/manual_qa_checklist.md` | 수동 QA 항목 추가 |

## 렌더링 설계

- 타이틀 배경은 기존 `TitleBackdrop` 월드 스프라이트 경로를 유지하고, 이미지 경로만 새 `title_backdrop_v2.png`로 교체했다.
- 로고 받침과 버튼 이미지는 `UiImageQueue`의 `DrawImage::textured_with_handle(...)`로 제출한다.
- 좌표 계산은 기존 `title_button_layout(vw, vh)`를 기준으로 한다.
- 클릭 판정은 `meta.rs`의 기존 `title_action_at(...)`을 그대로 사용한다.
- 이미지 버튼은 기존 히트박스보다 약간 크게 그려서 장식 가장자리가 버튼 영역을 감싸도록 했다.
- 이미지 생성 텍스트 오류를 피하기 위해 버튼 이미지 안에는 글자를 넣지 않았다.

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
- 새 타이틀 메뉴 이미지 레이아웃/투명 PNG 큐잉 테스트 3개 추가
- macOS 폴더 패키지와 `.app` 번들에 `assets/textures/survivor/menu/*.png` 포함 확인

제한:

- 초기 `screencapture` 기반 수동 캡처는 검은 화면으로만 잡혀 실제 타이틀 시각 smoke는 신뢰할 수 없었다.
- 이후 잠금 해제된 실제 화면에서 `Computer Use`로 기본 해상도와 800x600을 확인했다.
- 2026-05-29 투명 배경 패스에서 Title logo/button PNG 자체에 알파를 적용하고, Title 전용 불투명 backing 큐잉은 제거했다.

## 수동 QA 결과

- 기본 해상도에서 새 배경, 로고 받침, Start 버튼, 하단 4버튼이 모두 보인다.
- 800x600에서 로고/Start/하단 버튼이 서로 겹치지 않는다.
- Title logo/button PNG의 캔버스 배경이 투명하게 보인다.
- `Enter`, `S`, `O` 입력으로 InGame, Shop, Settings 진입을 확인했다.
- Settings 진입 후 `ESC`로 Title에 돌아와도 메뉴 이미지가 정상 유지된다.

남은 확인:

- Korean/English 전환 후 버튼 라벨이 이미지 위에서 읽히는지 확인.
- 마우스 클릭 영역과 이미지 버튼 위치가 체감상 어긋나지 않는지 확인.
- `C`, `T` 입력 흐름 확인.

## 후속 작업

1. 필요하면 버튼 이미지의 가로 비율과 텍스트 위치를 미세 조정한다.
2. 선택/hover 상태용 글로우 이미지를 별도 생성해 현재 커서/마우스 위치 피드백을 강화한다.
3. CharacterSelect, StageSelect, Shop 화면도 같은 방식으로 카드/패널 이미지를 단계적으로 적용한다.
