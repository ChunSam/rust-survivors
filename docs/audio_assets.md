# Rust Survivors Audio Assets

작성일: 2026-05-23

오디오는 `assets/audio/` 아래 파일을 사용한다.
BGM은 상황별 2곡 MP3 variant만 사용한다.
상용 음원으로 교체할 때도 아래 파일명을 유지하면 코드 변경 없이 반영된다.
파일이 없으면 게임은 계속 실행된다. BGM은 조용히 정지하고, SFX는 절차적 톤 fallback을 사용한다.

## 경로 규칙

BGM 경로는 현재 작업 디렉터리(`cwd`)가 아니라 실행 파일 기준으로 해석한다.

- macOS `.app`: `Contents/Resources/assets/audio`
- 일반 실행 파일: 실행 파일 옆 `assets/audio`, 없으면 상위 1단계의 `assets/audio`
- 개발 빌드(`debug_assertions`): 위 경로에 없을 때만 워크스페이스의 `assets/audio`

의도:

- 임의 cwd 에서 실행해도 같은 음원을 읽는다.
- 패키지 외부의 우연한 상대경로 MP3를 잘못 집어오지 않는다.
- 자산이 없으면 잘못된 fallback 대신 BGM 을 정지하고 경고 로그를 남긴다.

## BGM

| 파일명 | 사용 시점 | 반복 |
|---|---|---|
| `rustsurvivors title1.mp3`, `rustsurvivors title2.mp3` | Title, CharacterSelect, StageSelect, Shop, Settings | 반복 |
| `rustsurvivors ingame1.mp3`, `rustsurvivors ingame2.mp3` | InGame | 반복 |
| `rustsurvivors boss1.mp3`, `rustsurvivors boss2.mp3` | InGame 중 보스가 살아 있을 때 | 반복 |
| `rustsurvivors stageclear1.mp3`, `rustsurvivors stageclear2.mp3` | StageClear | 1회 |
| `rustsurvivors gameover1.mp3`, `rustsurvivors gameover2.mp3` | GameOver | 1회 |

각 상황에 진입할 때마다 1번과 2번이 번갈아 선택된다. Title/InGame/Boss처럼 반복형 BGM에 2곡 variant가 있으면, 현재 곡이 자연 종료될 때 다음 variant로 이어서 재생된다. StageClear/GameOver는 짧은 1회 cue로 유지된다.

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
