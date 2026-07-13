# survivor — Vampire Survivors 클론

`crates/game/src/survivor/`는 `skeleton-engine` 위에서 동작하는 탑다운 자동공격 로그라이트 게임 로직이다.
전체 로드맵과 최신 진행 상황은 루트의 `AGENTS.md`, `CLAUDE.md`를 참고하되, 예정 작업은 `docs/NEXT_WORK_PLAN.md` 하나만 기준으로 본다.

## 현재 상태

- Phase 1~10 완료: 코어 루프, 무기/패시브/적/보스/진화/픽업/메타 진행/캐릭터/스테이지가 들어가 있다.
- Phase 11-A~F 완료: 데미지 숫자, 히트 플래시, 화면 흔들기, 절차적 SFX, 파티클, 동적 카메라 줌이 적용됐다.
- 최근 추가: Settings 화면, HUD 정보량 3단계, 업적 20종과 해금 보상, 이미지 기반 Title/HUD 패스, 엔진 v2 대응, SFX bus 볼륨, Luck/Greed/Curse/Revival/Reroll/Skip/Banish 런타임 효과.
- 오디오는 `assets/audio/`의 BGM/SFX 파일을 우선 사용하고, SFX 파일이 없으면 절차적 톤으로 fallback한다. 파일명은 `docs/audio_assets.md` 기준.
- 예정 작업은 `docs/NEXT_WORK_PLAN.md`에만 기록한다.

## 실행

```bash
cargo run -p game --bin survivor
cargo run -p game --bin survivor --release
cargo run -p game --bin game
```

검증 명령:

```bash
cargo fmt --check
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

## 조작

| 키 | 동작 |
|---|---|
| `W/A/S/D`, 방향키 | 이동 또는 메뉴 커서 이동 |
| `Enter` | 타이틀/메뉴 선택 |
| `1/2/3` | 레벨업 카드 선택 |
| `R` | 레벨업 카드 reroll 또는 GameOver 후 재시작 |
| `S` | 레벨업 보상 skip |
| `4/5/6` | 레벨업 카드 1/2/3 banish |
| `ESC` | 일시정지 메뉴 또는 이전 화면 |
| `F5` | 시각 검증용 Bomb 픽업 스폰 |
| `F6` | 시각 검증용 Rosary 픽업 스폰 |
| `B` | 시각 검증용 보스 소환 |

## 작업 경계

이 저장소의 작업 대상은 `crates/game`이다. 엔진 소스는 별도 `skeleton-engine` 저장소에서 관리한다.
엔진 API, 렌더러, 입력, 오디오, 저장, 충돌 모듈 자체 수정이 필요하면 이 저장소에서 엔진 checkout을 직접 수정하지 말고 `docs/ENGINE_CHANGE_REQUESTS.md`에 요청을 남긴다.

## 모듈 개요

| 파일 | 내용 |
|---|---|
| `mod.rs` | 하위 모듈 선언, 재수출, 충돌 레이어 상수 |
| `world_setup.rs` | 초기 리소스와 플레이어/메타 상태 설정 |
| `meta.rs` | `SurvivorMode`, 타이틀/상점/일시정지/설정/클리어 전환 |
| `hud.rs` | 타이틀, HUD, 레벨업, 상점, 선택 화면, GameOver 텍스트 출력 |
| `locale.rs` | 한국어/영어 문자열 헬퍼 |
| `achievement.rs` | 업적 20종, 달성 감지, 캐릭터/스테이지/파워업 보상 |
| `player.rs`, `health.rs`, `stats.rs` | 플레이어 이동, 체력/회복, 전투 스탯 재계산 |
| `inventory.rs`, `passive.rs`, `powerup.rs` | 무기 슬롯, 패시브 슬롯, 메타 파워업 |
| `weapon.rs`, `projectile.rs`, `area.rs`, `bible.rs`, `lightning.rs` | 무기 10종과 투사체/오라/필드/회전책/번개 처리 |
| `enemy.rs`, `spawn.rs`, `director.rs` | 적 종류, 스폰 헬퍼, 시간축 웨이브 감독 |
| `boss.rs` | 보스 3종, 페이즈, StageClear 진행 상태 |
| `xp.rs`, `levelup.rs` | XP 보석 흡수, 레벨업 카드 생성/선택 |
| `pickup.rs`, `chest.rs` | 픽업 5종, 보물상자, 진화 레시피 |
| `character.rs`, `stage.rs` | 캐릭터 6종과 스테이지 3종 선택/해금 |
| `damage.rs`, `damage_number.rs` | 데미지 적용 헬퍼와 데미지 숫자 |
| `particle.rs`, `sfx.rs`, `bgm.rs` | 파티클, 절차적 SFX 큐, BGM 전환 |
| `sprites.rs`, `title_visual.rs` | atlas sprite 매핑, 타이틀 배경과 이미지 기반 메뉴 장식 |
| `debug_input.rs` | 수동 시각 검증용 입력 |
| `data.rs` | 데이터 로딩 헬퍼 |

## 시스템 등록 흐름

실제 등록 순서는 `crates/game/src/bin/survivor.rs`가 권위 있는 기준이다. 큰 흐름은 다음과 같다.

1. 디버그 입력, 모드 전환, BGM, 메뉴 입력
2. 스탯 재계산, 레벨업, 플레이어/적/스폰/보스
3. 접촉 데미지, 체력 회복, 사망/재시작, 카메라/타이틀 비주얼
4. 무기 10종, 투사체, 자석, 보물상자, 픽업
5. 히트 플래시, 데미지 숫자, 파티클
6. HUD 출력, SFX 큐 처리

메뉴나 일시정지 중에는 `SurvivorMode`와 `GameState`를 통해 전투 시스템이 동작하지 않도록 막는다.

## 다음 작업

현재 예정 작업은 `docs/NEXT_WORK_PLAN.md` 하나로 통합한다. 이 README에는 별도 계획 목록을 두지 않는다.
