# Logo Text Image Pass

작성일: 2026-05-29

로고 이미지의 질감 있는 금속 텍스트 스타일을 메뉴 버튼과 인게임 고정 텍스트 PNG에 확장한 작업 기록이다.

## 범위

- 유지: `title_logo_plaque_v3.png`
- 교체: `menu_button_*_{ko,en}.png`
- 교체: `ingame_label_*.png`, `levelup_title_*.png`, `gameover_title_*.png`, `restart_hint_*.png`, `section_*.png`
- 유지: `title_backdrop_v2.png`, `ui_modal_panel.png`, `ui_slot_frame.png`

## 생성 방식

- `imagegen`으로 먼저 `menu_button_start_ko.png` 샘플을 생성하고 사용자 확인을 받았다.
- 승인된 방향: 기존 로고에 가까운 금색 금속 텍스트, 낡은 고딕 프레임, 균열 있는 검은 석재 플레이트, 붉은 보석 포인트.
- 프레임은 첫 샘플보다 덜 화려하게 조정해 버튼/라벨 가독성을 우선했다.
- 작은 HUD 라벨은 같은 분위기의 mini plaque로 단순화했다.
- 생성 결과는 기존 런타임 치수에 맞게 리사이즈했다.

## 투명 배경 처리

- `scripts/make_text_ui_transparent.py`로 최종 PNG에 알파 마스크를 적용했다.
- 검은 석재 플레이트는 유지하고, 캔버스 바깥의 검은 배경만 투명하게 처리했다.
- 치수는 기존 런타임 치수와 동일하게 유지했다.
- Title logo/button은 PNG 알파를 직접 사용하도록 `title_visual.rs`의 전용 backing 큐잉을 제거했다.

## QA

- 한국어/영어 버튼 문구가 오탈자 없이 읽히는지 확인한다.
- 800x600에서 HUD mini label이 값 텍스트와 겹치지 않는지 확인한다.
- 알파 적용 후 각 PNG의 네 모서리가 투명하고, 석재 플레이트 내부가 투명하게 파먹히지 않았는지 확인한다.
- `Lv` 라벨은 생성 모델 특성상 약간 대문자처럼 보일 수 있으므로 실제 게임 화면에서 가독성을 확인한다.

## 이어받기 컨텍스트

- 사용자가 승인한 최종 방향은 두 번째 `시작` 버튼 샘플이다. 첫 샘플은 프레임이 너무 화려해 축소 방향으로 조정했다.
- 로고 이미지는 스타일 기준으로 유지하되, 이번 패스에서는 `title_logo_plaque_v3.png`에도 투명 배경 알파를 적용했다.
- 레거시 `menu_button_start.png`, `menu_button_character.png`, `menu_button_stage.png`, `menu_button_shop.png`, `menu_button_settings.png`, `title_logo_plaque.png`는 현재 패키지에 남아 있다. 런타임은 `{ko,en}` 파일을 사용한다.
- Title 화면에서 다시 사각 배경이 보이면 PNG 알파보다 `title_visual.rs`의 큐잉 로직을 먼저 확인한다. 이번 패스 기준 Title logo/button에는 별도 backing을 두지 않는다.
- 공통 `ui_modal_panel.png` / `ui_slot_frame.png` backing은 이번 텍스트 PNG 작업 대상이 아니며 유지한다.
- 새 텍스트 PNG를 다시 생성하거나 리사이즈한 뒤에는 `scripts/make_text_ui_transparent.py`를 다시 실행하고 `docs/ASSET_LICENSES.md`의 SHA-256을 갱신한다.

## 최종 검증

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

- game lib tests: 167 passed
- 패키지 앱 `dist/macos/RustSurvivors.app` Title 화면에서 로고/버튼 캔버스 배경이 투명하게 보이는 것을 확인했다.
