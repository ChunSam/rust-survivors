# Rust Survivors 작업 계획서

작성일: 2026-05-23
갱신일: 2026-05-24
상태: Phase A~F 구현/문서/로컬 패키징 검증 완료. 남은 게이트는 Windows CI 실제 실행 결과와 수동 GUI QA다.

## 1. 목표

현재 빌드는 Vampire Survivors 클론의 핵심 루프와 주요 콘텐츠가 구현된 상태다. 다음 작업의 목표는 "기능 추가"보다 "상용 빌드에 가까운 체감 품질과 검증 가능성"을 높이는 것이다.

우선순위는 다음 순서로 둔다.

1. 실제 플레이 화면 기준으로 시각 품질과 가독성 검증
2. HUD/타이틀/장비 슬롯 UX 정리
3. 업적 시스템과 해금 연동
4. 한/영 전체 로컬라이제이션
5. 외부 음원 기반 BGM/SFX 연결
6. macOS/Windows 배포 준비

## 2. 현재 기준

- 작업 대상 repo: `rust-survivors`
- 작업 대상 크레이트: `crates/game`
- 엔진 의존성: `engine = { git = "https://github.com/ChunSam/rust-2d-engine" }`
- 로컬 `crates/engine`은 legacy/reference snapshot이며 새 작업 대상이 아니다.
- 검증 기준:
  - `cargo fmt`
  - `cargo test -p game --lib`
  - `cargo build -p game --bin survivor`
  - 필요 시 `cargo run -p game --bin survivor`

## 3. 작업 원칙

- dirty worktree가 많으므로 임의 revert 금지.
- 게임 기능, 밸런스, UI, 콘텐츠, 자산은 현재 repo에서 작업한다.
- 엔진 API, renderer, input, audio, save 자체 수정이 필요하면 별도 `rust-2d-engine` repo에서 작업한다.
- Rust 초급자 학습 목적을 고려해 복잡한 borrow checker 회피 패턴은 짧은 이유 주석 또는 문서 설명을 남긴다.
- 큰 기능은 "작게 구현 -> 테스트 -> 수동 검증 체크리스트 갱신" 순서로 진행한다.
- 현재 개발 중 save 호환성은 유지하지 않아도 된다. save 구조 변경이 필요하면 단순하게 새 구조로 정리한다.

## 4. 확정된 방향

| 항목 | 결정 |
|---|---|
| 최우선 작업 | 화면/HUD 개선, 업적 시스템 |
| HUD 장비 슬롯 | 텍스트 기반 개선 먼저 |
| HUD 표기 | 한글 이름 중심. 예: `채찍 3`, `마늘 2` |
| HUD 정보량 | 설정에서 최소/중간/상세 3단계 전환 |
| 오디오 | 외부 음원 파일 연결 |
| 업적 보상 | 해금과 연결 |
| 업적 보상 방향 | 캐릭터/스테이지/파워업 혼합 |
| 업적 화면 | Shop/Meta 화면 안에 포함 |
| 로컬라이제이션 | 한국어/영어 전체 적용 |
| 기본 언어 | 시스템 언어 감지 |
| 외부 음원 확보 | 사용자가 파일 제공. 코드는 정해진 경로의 파일을 로드 |
| 상세 HUD 스탯 | 핵심 전투 스탯만 표시 |
| 업적 초기 개수 | 20개 내외 |
| Settings 위치 | 별도 Settings 화면 |
| 배포 | 이번 묶음에서는 후순위 |
| save 호환성 | 개발 중이므로 깨져도 됨 |

## 5. Phase A: 화면 품질 검증과 문서 정리

목표: 최근 추가된 타이틀 배경, atlas sprite, HUD 슬롯이 실제 화면에서 읽히는지 확인하고 문서 불일치를 정리한다.

작업 항목:

- `survivor` 실행 화면에서 타이틀 배경 노출, 화면 비율, 텍스트 대비 확인
- InGame HUD의 무기/패시브 슬롯 약어와 레벨 표시 가독성 확인
- 레벨업 카드, 캐릭터 선택, 스테이지 선택, 상점 화면에서 텍스트 겹침 확인
- `docs/beta_test_checklist.html`에 최근 UI 확인 항목 추가
- `crates/game/src/survivor/README.md`의 오래된 Phase 2 상태를 현재 구조에 맞게 갱신하거나 `CLAUDE.md`로 위임하도록 축약

완료 기준:

- `cargo test -p game --lib` 통과
- `cargo build -p game --bin survivor` 통과
- 수동 체크 결과가 문서에 남아 있음

## 6. Phase B: HUD/장비 슬롯 UX 개선

목표: 현재 텍스트 기반 장비 슬롯을 플레이 중 빠르게 읽을 수 있는 형태로 개선한다.

이번 결정:

- 현 엔진 기능을 유지한다.
- 텍스트 약어 + 레벨 + 색상 + 정렬을 개선한다.
- 슬롯 표기는 한글 이름 중심으로 한다. 예: `채찍 3`, `마늘 2`.
- atlas 아이콘 UI와 엔진 UI texture pass 확장은 이번 작업 범위에서 제외한다.

작업 항목:

- 무기 슬롯과 패시브 슬롯의 행/열 구조 고정
- 한글 장비명 표시 규칙 통일
- 레벨 표시 형식 통일
- 빈 슬롯 표시 방식 통일
- HUD 정보량 설정 추가:
  - 최소: HP/XP/Level/Time/Kills + 장비 슬롯
  - 중간: 최소 정보 + 골드/캐릭터/스테이지
  - 상세: 중간 정보 + 핵심 전투 스탯 요약
- 상세 HUD 스탯 범위:
  - 공격력
  - 쿨다운
  - 범위
  - 투사체 수
  - 이동속도
- 낮은 해상도 800x600 기준 텍스트 겹침 방지
- Title / InGame / LevelUp / Shop / CharacterSelect / StageSelect 화면의 텍스트 밀도 조정

완료 기준:

- 800x600 기준 슬롯 텍스트가 겹치지 않음
- 무기/패시브 이름, 레벨, 빈 슬롯 상태가 구분됨
- 설정에서 HUD 정보량 3단계를 바꿀 수 있음
- HUD 변경에 대한 unit test 또는 snapshot이 어렵다면 수동 체크리스트에 명시

## 7. Phase C: 업적 시스템

목표: 메타 진행과 해금을 보강할 업적 기반을 추가한다.

작업 항목:

- `AchievementKind` enum 정의
- save 구조에 업적 달성 상태 추가
- 업적 달성 이벤트 감지 시스템 추가
- 업적 달성 시 캐릭터/스테이지/파워업 해금을 혼합 보상으로 연결
- 업적 목록을 Shop/Meta 화면 안에 포함
- 초기 업적은 20개 내외로 구성
- save 호환성은 유지하지 않고 단순한 새 구조로 정리

완료 기준:

- save/load 후 업적 상태 유지
- 중복 달성 처리 없음
- 업적 보상으로 캐릭터/스테이지/기능 해금 가능
- Shop/Meta 화면에서 업적 목록과 달성 상태 확인 가능
- 최소 5개 업적 테스트 통과

초기 업적 후보:

- 첫 런 시작
- 첫 스테이지 클리어
- 5분 생존
- 10분 생존
- 1000마리 처치
- 첫 무기 Lv8
- 첫 진화 성공
- Mad Forest 클리어
- Inlaid Library 클리어
- 특정 캐릭터로 클리어

## 8. Phase D: 로컬라이제이션

목표: 한국어/영어 전체 UI 문자열을 관리 가능한 구조로 옮긴다.

작업 항목:

- 현재 `locale.rs` 구조 점검
- UI 문자열을 enum key 또는 함수 기반으로 정리
- Title, HUD, LevelUp, Shop, CharacterSelect, StageSelect, GameOver부터 적용
- 업적 화면과 업적 이름/설명도 한/영 대응
- 기본 언어는 시스템 언어 감지로 결정
- 별도 Settings 화면에서 언어와 HUD 정보량을 변경
- 저장 파일에는 사용자가 바꾼 언어 설정만 저장하고, 표시 문자열은 코드/데이터에서 로드

완료 기준:

- 한국어/영어 전환 가능
- 첫 실행 시 시스템 언어가 한국어면 한국어, 그 외에는 영어로 시작
- 새 화면 추가 시 문자열 누락이 드러나는 구조
- hard-coded UI 문자열 감소

## 9. Phase E: 오디오 연결

목표: 외부 음원 파일을 BGM과 주요 SFX에 연결한다.

작업 항목:

- `assets/audio` 구조 확정
- 사용자가 제공할 파일명과 폴더 구조 문서화
- Title / InGame / Boss / StageClear / GameOver BGM 전환 규칙 정의
- 기존 `BgmSystem`과 `AudioManager` 연결 상태 점검
- SFX 우선순위 정의:
  - 공격 hit
  - 적 사망
  - XP 흡수
  - 레벨업
  - 보물상자
  - 보스 등장
  - 플레이어 피격
- CoreAudio 초기화 실패 시 게임이 계속 실행되는 현재 동작 유지

완료 기준:

- 오디오 자산이 있으면 실제 wav/ogg 재생
- 없는 파일은 조용히 실패하거나 절차적 SFX fallback 유지
- 오디오 실패가 게임 루프를 중단하지 않음
- 사용자가 어떤 파일을 어디에 넣어야 하는지 문서로 확인 가능

## 10. Phase F: 배포 준비

목표: 후순위. HUD/업적/로컬라이제이션/오디오가 정리된 뒤 테스트 가능한 로컬 배포물을 만든다.

작업 항목:

- macOS `.app` 패키징 방식 결정
- Windows `.exe` 빌드 방식 결정
- asset path가 개발/배포 환경 모두에서 동작하는지 확인
- release build 실행 체크리스트 작성
- 라이선스 파일 정리:
  - NotoSansKR OFL
  - 외부 texture/audio asset license

완료 기준:

- macOS release build 실행 가능
- Windows 빌드 절차 문서화 또는 실제 산출물 생성
- 외부 자산 라이선스 출처 문서화

현재 문서:

- 배포 체크리스트: `docs/release_checklist.md`
- 오디오 파일명 규격: `docs/audio_assets.md`
- 자산 라이선스 목록: `docs/ASSET_LICENSES.md`
- 수동 QA 체크리스트: `docs/manual_qa_checklist.md`
- macOS/Windows CI smoke: `.github/workflows/release-smoke.yml`

## 11. 권장 실행 순서

1. Phase A: 화면 품질 검증 + 문서 정리
2. Phase B1: HUD 텍스트 가독성 개선
3. Phase C: 업적 시스템 최소 버전 + 해금 연동
4. Phase D: 한/영 전체 로컬라이제이션
5. Phase E: 외부 음원 BGM/SFX 연결
6. Phase F: 배포 준비

## 11.5 현재 완료 상태 (2026-05-24)

- Phase A: `survivor` README와 베타/수동 QA 체크리스트 갱신. macOS 패키지 Title smoke 결과 기록.
- Phase B: Settings 기반 HUD 정보량 3단계, 텍스트 기반 장비 슬롯 6+6 고정 표시, 한/영 장비명/빈 슬롯 표기 정리.
- Phase C: 업적 20종, save 연동, 중복 방지, 캐릭터/스테이지/파워업 보상, Shop 업적 요약 표시.
- Phase D: `UiText` 기반 주요 UI 문자열 정리, 언어 System/Ko/En 설정, 번역 누락 단위 테스트.
- Phase E: BGM/SFX 외부 파일 우선 로드, Boss BGM, 보물상자/보스 등장 SFX, 절차적 fallback, `docs/audio_assets.md` 정리.
- Phase F: macOS 폴더 패키지 + `.app` 번들, Windows 패키징/검증 스크립트, macOS/Windows CI smoke, `PACKAGE_MANIFEST.sha256`, 라이선스 문서 포함.

검증 증거:

- `cargo fmt --check` 통과.
- `cargo test -p game --lib` 통과: 96 passed.
- `cargo build -p game --bin survivor --release` 통과.
- `scripts/package_macos.sh` 통과: `dist/macos/RustSurvivors`, `dist/macos/RustSurvivors.app` 생성.
- `scripts/verify_macos_package.sh` 통과.
- `.github`, `docs`, `scripts`, `assets`, `dist/macos` 내 `._*` / `.DS_Store` 없음.

잔여 외부 게이트:

- GitHub Actions `release-smoke`의 `windows-latest` 실제 실행 결과 확인.
  - 2026-05-24 현재 `gh run list --workflow release-smoke.yml` 결과: 원격 default branch에 workflow가 아직 없어 `HTTP 404: workflow release-smoke.yml not found on the default branch`.
  - 이 게이트는 `.github/workflows/release-smoke.yml` 포함 변경분을 commit/push한 뒤 `gh workflow run release-smoke.yml --ref main`으로 실행해야 닫을 수 있다.
- 사람이 직접 `docs/manual_qa_checklist.md` 기준으로 Settings, 800x600 HUD, 메뉴 화면, 오디오, 저장/업적 QA 수행.

## 12. Settings 화면 구현 상태

별도 Settings 화면은 구현됐다.

설정 항목:

- 언어: 시스템 감지 / 한국어 / 영어
- HUD 정보량: 최소 / 중간 / 상세
- BGM 볼륨
- SFX 볼륨

진입 방식:

- Title 메뉴에서 `O`로 Settings 진입
- `W/S` 또는 방향키로 항목 이동
- `A/D` 또는 좌우 방향키로 값 변경
- 해상도 항목에서 `Enter`로 resize 적용
- `ESC`로 Title 복귀

저장 방식:

- `MetaSave`에 언어/HUD 정보량/BGM 볼륨/SFX 볼륨/해상도 key를 저장한다.
- 언어가 시스템 감지일 때는 실행 시 `LC_ALL`, `LC_MESSAGES`, `LANG` 순서로 확인하고, 한국어/영어가 명시되면 저장값을 우선한다.

## 13. 다음 착수 순서

초기 착수 목록은 완료됐다. 다음 순서는 잔여 검증 게이트를 닫는 것이다.

1. GitHub Actions `release-smoke`를 실행해 `macos-latest`와 `windows-latest` 결과 확인
   - 선행 조건: 현재 변경분 commit/push
   - 실행: `gh workflow run release-smoke.yml --ref main`
   - 조회: `gh run list --workflow release-smoke.yml --limit 5`
2. Windows artifact `dist/windows/RustSurvivors`를 `scripts\verify_windows_package.ps1` 기준으로 검증한 결과 확인
3. macOS `.app`을 실제 GUI로 열어 Title → Settings → InGame → Pause/GameOver 흐름 수동 QA
4. 800x600 해상도에서 HUD/LevelUp/Shop/CharacterSelect/StageSelect 겹침 확인
5. 외부 오디오 파일 교체 시 `docs/ASSET_LICENSES.md`에 출처/라이선스/저작자 표기 요구사항 추가

완료된 기능 착수 목록:

1. 별도 Settings 화면 추가
2. Settings에 언어/HUD 정보량/BGM 볼륨/SFX 볼륨 설정 추가
3. HUD 텍스트 개선과 정보량 3단계 적용
4. 업적 시스템 20개 내외 추가
5. Shop/Meta 화면에 업적 섹션 추가
6. 한/영 전체 로컬라이제이션 적용
7. 외부 음원 파일 경로와 문서 추가
