# UI Icon Spacing Pass

Date: 2026-05-29

## Summary

이번 작업은 survivor UI 아이콘의 작은 창/고해상도 배치 안정성을 높이는 pass다. 대상은 HUD 슬롯, Level-up 카드, Shop 파워업 목록이다.

핵심 문제는 Level-up 카드와 Shop 파워업 row에서 아이콘과 텍스트 사이 여백이 고해상도에서 2px 수준까지 줄어들 수 있다는 점이었다. 실제 충돌은 자동 테스트상 발생하지 않았지만, 800x600과 HiDPI/고해상도 창에서 읽기 여유를 확보하기 위해 아이콘 좌표를 조금 더 왼쪽으로 옮기고 회귀 테스트를 보강했다.

엔진 수정은 하지 않았다. 이번 변경은 `rust-survivors` 게임 코드의 screen-space UI 좌표 계산과 테스트만 조정했다.

## Changes

| File | Change |
|---|---|
| `crates/game/src/survivor/ui_icons.rs` | Level-up 아이콘 X 오프셋을 `-206.0`에서 `-214.0`으로 조정. |
| `crates/game/src/survivor/ui_icons.rs` | Shop 아이콘 X 오프셋을 `-326.0`에서 `-334.0`으로 조정. |
| `crates/game/src/survivor/ui_icons.rs` | Level-up/Shop 아이콘 X 오프셋을 상수화해 테스트와 런타임 계산이 같은 기준을 쓰게 함. |
| `crates/game/src/survivor/ui_icons.rs` | HUD 슬롯 테스트의 텍스트 inset 기준을 실제 HUD 렌더 좌표와 맞춤. |
| `crates/game/src/survivor/ui_icons.rs` | 800x600과 3840x2160에서 Level-up/Shop 아이콘이 텍스트 여백을 침범하지 않는지 테스트. |
| `docs/manual_qa_checklist.md` | UI Icon Spacing Pass 기록과 수동 QA 권장 항목 추가. |

## Layout Rules

- HUD 슬롯은 800x600에서도 6칸 x 2행 구조를 유지한다.
- HUD 아이콘은 레벨 텍스트 시작점보다 왼쪽에 남고, 텍스트와 최소 간격을 둔다.
- Level-up 카드 아이콘은 카드 row 내부에 남으며, 카드 텍스트 시작점까지 최소 10px 여백을 둔다.
- Shop 파워업 아이콘은 선택 row 내부에 남으며, 파워업 텍스트 시작점까지 최소 10px 여백을 둔다.
- 3840x2160처럼 UI 스케일이 커지는 환경에서도 아이콘 크기는 기존 cap을 유지한다.

## Validation

실행한 자동 검증:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
git diff --check
```

결과:

- `cargo fmt --check`: passed
- `cargo test -p game --lib --locked -- --test-threads=1`: 151 passed
- `cargo build -p game --bin survivor --release --locked`: passed
- `git diff --check`: passed

Targeted check:

```bash
cargo test -p game --lib survivor::ui_icons --locked -- --test-threads=1
```

결과: 10 passed.

## GUI Check Attempt

`target/release/survivor`를 실제 실행해 앱 초기화 로그를 확인했고, 실행 후 프로세스는 종료했다.

다만 macOS `screencapture` 결과가 전면 포커스 시도 후에도 검은 화면으로만 저장되어, 이번 실행은 신뢰 가능한 시각 QA 증거로 보지 않았다.

확인된 로그:

- `cosmic_text` fallback font warning
- `egui_wgpu` framebuffer format warning

두 로그는 기존 실행에서도 보이는 warning 성격이며, 이번 아이콘 좌표 변경과 직접 관련된 오류로 보이지 않는다.

## Remaining Manual QA

- 800x600 창에서 Level-up 카드 3장의 아이콘/텍스트 간격 확인.
- 800x600 창에서 Shop 파워업 목록의 아이콘/텍스트 간격 확인.
- HiDPI 또는 3840x2160급 고해상도 창에서 Level-up/Shop 아이콘이 텍스트와 붙어 보이지 않는지 확인.
- 하단 HUD 슬롯의 아이콘과 `Lv` 텍스트가 겹치지 않는지 확인.

