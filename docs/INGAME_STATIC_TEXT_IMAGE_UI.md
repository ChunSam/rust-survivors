# InGame Static Text Image UI Pass

작성일: 2026-05-29

인게임 HUD와 오버레이에서 고정 문구로 출력되던 일부 텍스트를 PNG 이미지 자산으로 교체한 작업 기록이다.
동적으로 바뀌는 값은 계속 `TextQueue` 텍스트로 유지한다.

## 목표

- 인게임 고정 라벨의 시각 스타일을 기존 dark fantasy UI 이미지 톤에 맞춘다.
- 한국어/영어 언어 설정에 따라 고정 문구 이미지도 함께 전환한다.
- 숫자, 시간, 카드명, 통계처럼 매 프레임 또는 상태에 따라 바뀌는 값은 텍스트 렌더링으로 유지한다.
- DebugOverlay는 개발용 UI라 이번 범위에서 제외한다.

## 교체 범위

이미지로 교체:

- 상단 HUD 라벨: `Lv`, `HP`, `XP`, `Gold`, `Kills`, `Passives`
- 장비 섹션 라벨: `Weapons`, `Passives`
- 레벨업 오버레이 제목: `LEVEL UP!`, `레벨 업!`
- 게임오버 제목: `YOU DIED`, `사망`
- 재시작 안내: `PRESS R TO RESTART`, `R 키로 재시작`

텍스트로 유지:

- 시간: `MM:SS`
- 레벨 숫자
- HP/XP 수치
- Gold/Kills/Passives 수치
- 장비 슬롯 내부 `Lv N`, `Lv N E`
- 레벨업 카드명과 선택 번호
- 게임오버 결과 통계
- 보스명, 데미지 숫자, DebugOverlay

## 생성 에셋

저장 위치: `assets/textures/survivor/ui/`

| 파일 패턴 | 용도 |
|---|---|
| `ingame_label_{lv,hp,xp,gold,kills,passives}_{ko,en}.png` | 상단 HUD 라벨 이미지 |
| `section_weapons_{ko,en}.png` | 장비 슬롯 무기 섹션 라벨 |
| `section_passives_{ko,en}.png` | 장비 슬롯 패시브 섹션 라벨 |
| `levelup_title_{ko,en}.png` | 레벨업 오버레이 제목 |
| `gameover_title_{ko,en}.png` | 게임오버 오버레이 제목 |
| `restart_hint_{ko,en}.png` | 게임오버 재시작 안내 |

생성 방식:

- `imagegen`으로 텍스트 없는 gothic/dark fantasy UI plaque와 banner 베이스 이미지를 생성했다.
- 이미지 생성 모델이 텍스트를 직접 그릴 경우 오탈자와 가짜 문자가 섞일 위험이 커서, 최종 문구는 로컬 display font로 래스터라이즈했다.
- 확정본은 가독성을 우선해 중간 굵기, 얇은 외곽선, 약한 중앙 광원, 한 방향 그림자만 적용했다.
- 2026-05-29 로고풍 텍스트 재작업에서 승인된 `시작` 버튼 샘플 톤을 기준으로 인게임 고정 텍스트 PNG도 다시 생성했다.
- 작은 HUD 라벨은 같은 톤을 유지하되 800x600 가독성을 위해 더 단순한 mini plaque로 맞췄다.
- `scripts/make_text_ui_transparent.py`로 검은 석재 플레이트는 유지하고 캔버스 배경만 투명 처리했다.
- 최종 PNG는 한/영 문구를 눈으로 확인한 뒤 workspace에 배치했다.
- SHA-256 해시는 `docs/ASSET_LICENSES.md`에 기록했다.

## 코드 연결

| 파일 | 변경 내용 |
|---|---|
| `crates/game/src/survivor/sprites.rs` | 새 UI 이미지 경로 상수 추가, `SurvivorTextureHandles` 필드 추가, `handle_for` resolve 추가 |
| `crates/game/src/bin/survivor.rs` | 새 PNG 자산을 `app.load_image(...)`로 시작 시 로드 |
| `crates/game/src/survivor/mod.rs` | 새 이미지 경로 상수 재수출 |
| `crates/game/src/survivor/hud.rs` | 고정 문구 `TextQueue` push 제거, `UiImageQueue` 이미지 제출로 교체 |
| `docs/ASSET_LICENSES.md` | 새 UI PNG 출처와 SHA-256 기록 |
| `docs/manual_qa_checklist.md` | 인게임 고정 텍스트 이미지화 확인 항목 추가 |

## 렌더링 설계

- 이미지는 `queue_ingame_ui_image(...)` helper를 통해 `UiImageQueue`에 제출한다.
- `MetaSave::effective_lang()`으로 계산된 `Lang`을 기준으로 `{ko,en}` 이미지 경로를 선택한다.
- 상단 HUD는 기존 하나의 긴 문자열을 분해했다.
  - 시간은 텍스트로 먼저 출력한다.
  - 각 라벨은 PNG 이미지로 출력한다.
  - 라벨 오른쪽의 값은 텍스트로 출력한다.
- 800x600과 1280x720에서 라벨 이미지와 값 텍스트가 겹치지 않도록 폭 추정 helper와 회귀 테스트를 추가했다.
- 기존 HP/XP 바, 장비 슬롯 프레임, 카드/모달 프레임의 이미지 기반 UI 흐름은 유지했다.

## 테스트

추가/보강:

- 언어별 고정 UI 이미지 경로 선택 테스트
- 800x600, 1280x720 상단 HUD 라벨/값 레이아웃 폭 테스트
- `SurvivorTextureHandles::handle_for`가 새 UI 이미지 경로를 resolve하는지 테스트

검증 완료:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
bash scripts/package_macos.sh
bash scripts/verify_macos_package.sh
```

결과:

- lib tests: 157 passed
- survivor release build: passed
- macOS folder package와 `.app` package 모두 새 `assets/textures/survivor/ui/*.png` 포함 확인

## 수동 QA 포인트

- 한국어/영어 설정에서 HUD 라벨, 레벨업 제목, 게임오버 제목/재시작 안내가 언어에 맞게 표시되는지 확인한다.
- 800x600에서 상단 HUD 라벨 이미지와 값 텍스트가 겹치지 않는지 확인한다.
- 장비 섹션 라벨은 이미지로 보이고 슬롯 내부 `Lv N`/`Lv N E`는 계속 텍스트로 표시되는지 확인한다.
- 레벨업 카드명과 게임오버 결과 통계는 기존처럼 텍스트로 읽히는지 확인한다.

## 후속 작업

1. 실제 플레이 화면에서 KO/EN 전환 후 모든 고정 이미지 문구의 선명도를 확인한다.
2. 필요하면 HUD 라벨 이미지 크기와 값 텍스트 간격을 해상도별로 미세 조정한다.
3. 숫자까지 이미지화하려면 별도 bitmap digit atlas 또는 signed distance font 경로를 검토한다.
4. Shop/Settings/Achievements처럼 텍스트 밀도가 높은 화면은 별도 UI 정리 pass에서 다룬다.
