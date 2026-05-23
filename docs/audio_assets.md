# Rust Survivors Audio Assets

작성일: 2026-05-23

오디오는 `assets/audio/` 아래에서 먼저 `.ogg`, 그 다음 `.wav` 순서로 찾는다.
파일이 없으면 게임은 계속 실행된다. BGM은 조용히 정지하고, SFX는 절차적 톤 fallback을 사용한다.

## BGM

| 파일명 | 사용 시점 | 반복 |
|---|---|---|
| `bgm_title.ogg` 또는 `bgm_title.wav` | Title, CharacterSelect, StageSelect, Shop, Settings | 반복 |
| `bgm_ingame.ogg` 또는 `bgm_ingame.wav` | InGame | 반복 |
| `bgm_boss.ogg` 또는 `bgm_boss.wav` | InGame 중 보스가 살아 있을 때 | 반복 |
| `bgm_stageclear.ogg` 또는 `bgm_stageclear.wav` | StageClear | 1회 |
| `bgm_gameover.ogg` 또는 `bgm_gameover.wav` | GameOver | 1회 |

## SFX

| 파일명 | 이벤트 | fallback |
|---|---|---|
| `sfx_enemy_hit.ogg` 또는 `sfx_enemy_hit.wav` | 적 피격 | 짧은 고음 톤 |
| `sfx_enemy_die.ogg` 또는 `sfx_enemy_die.wav` | 적 사망 | 짧은 저음 톤 |
| `sfx_player_hit.ogg` 또는 `sfx_player_hit.wav` | 플레이어 피격 | 중간 충격 톤 |
| `sfx_levelup.ogg` 또는 `sfx_levelup.wav` | 레벨업 | 상승 2음 톤 |
| `sfx_xp.ogg` 또는 `sfx_xp.wav` | XP 젬 흡수 | 짧은 고음 핑 |
| `sfx_pickup.ogg` 또는 `sfx_pickup.wav` | 일반 픽업 | 밝은 핑 |
| `sfx_bomb.ogg` 또는 `sfx_bomb.wav` | 폭탄/광역 폭발 | 저주파 톤 |
| `sfx_chest_open.ogg` 또는 `sfx_chest_open.wav` | 보물상자 열림 | 2음 반짝임 |
| `sfx_boss_appear.ogg` 또는 `sfx_boss_appear.wav` | 보스 등장 | 저음 경고 톤 |

## 볼륨

Settings 화면에서 BGM/SFX 볼륨을 0~100%로 저장한다.
저장 위치는 엔진 save 경로의 `rust-vampire-survivors/save.ron`이다.

## 라이선스 메모

외부 음원을 넣을 때는 다음 정보를 별도 라이선스 문서에 남긴다.

- 파일명
- 원본 출처 URL
- 라이선스 이름
- 저작자 표기 요구 여부
- 수정 여부
