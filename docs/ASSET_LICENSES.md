# Rust Survivors Asset Licenses

작성일: 2026-05-23
갱신일: 2026-05-24

출시 빌드에는 모든 외부 자산의 출처와 라이선스를 이 문서에 남긴다.
출처가 불명확한 파일은 release package에 포함하지 않는다.

## Fonts

| 파일 | 출처 | 라이선스 | 메모 |
|---|---|---|---|
| `assets/fonts/NotoSansKR-Regular.ttf` | Google Noto Fonts | SIL Open Font License 1.1 | 라이선스 원문: `assets/fonts/OFL.txt` |

## Textures

| 파일 또는 폴더 | 출처 URL | 라이선스 | 저작자 표기 | 메모 |
|---|---|---|---|---|
| `assets/textures/survivor/survivor_atlas.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `04b2c722fb8173a928fae389fab2cb20124eccef1bd025ed783125f7a35f6d40` |
| `assets/textures/survivor/survivor_atlas_source.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `9c80f543eceafb690ad3bbcae4cd878b3b4dc5774e497d5309916057891d7a6e` |
| `assets/textures/survivor/title_backdrop.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `ff42551fbfb38339d2c02e0dd8cf650d8908ee59050ec08853498c637e3120a1` |

## Audio

| 파일 | 출처 URL | 라이선스 | 저작자 표기 | 수정 여부 |
|---|---|---|---|---|
| `assets/audio/bgm_title.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `148e61107c1b0b84a9a4a371c0f92d7228327d634fbbb474ef013dd5cd590de6` |
| `assets/audio/bgm_ingame.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `df9e054ecf1cb71c06c05c59f68948b654b59b83459306842fc5a29bddf7e647` |
| `assets/audio/bgm_stageclear.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `9e8efc0a18cafb74d78a5ef8ba7048c65530b87dc1eae25cd67d94ab04f2f6ea` |
| `assets/audio/bgm_gameover.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `bb6a7c10c4279eae7834e9dfd749c575fff51812a304da9bb141b4988d099364` |

## Game Data / Shaders

| 파일 또는 폴더 | 출처 | 라이선스 | 메모 |
|---|---|---|---|
| `assets/data/*.ron` | 프로젝트 내부 제작 | 프로젝트 코드와 동일하게 관리 | wave/weapon data |
| `assets/shaders/sprite.wgsl` | 프로젝트 내부 제작 | 프로젝트 코드와 동일하게 관리 | wgpu sprite shader |

## Release Gate

- `local generated asset` 항목은 현재 저장소 내부 placeholder로 관리한다. 외부 자산으로 교체하면 출처 URL, 라이선스, 저작자 표기 요구사항을 이 문서에 추가한다.
- macOS resource fork 파일인 `._*`는 배포물에 포함하지 않는다.
- CC0, permissive, 직접 제작, 구매 라이선스 등 상용 배포 가능 여부를 확인한다.
- 저작자 표기가 필요한 자산은 credits 문서나 게임 내 credits 화면에 반영한다.
