# Rust Survivors Asset Licenses

작성일: 2026-05-23
갱신일: 2026-05-30

출시 빌드에는 모든 외부 자산의 출처와 라이선스를 이 문서에 남긴다.
출처가 불명확한 파일은 release package에 포함하지 않는다.

## Fonts

| 파일 | 출처 | 라이선스 | 메모 |
|---|---|---|---|
| `assets/fonts/NotoSansKR-Regular.ttf` | Google Noto Fonts | SIL Open Font License 1.1 | 라이선스 원문: `assets/fonts/OFL.txt` |

## Textures

| 파일 또는 폴더 | 출처 URL | 라이선스 | 저작자 표기 | 메모 |
|---|---|---|---|---|
| `assets/textures/survivor/survivor_atlas.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `5931fcbae8737ba14458d078e8b206a57b8c14318ba4200955f4a8a9a53fdad8` |
| `assets/textures/survivor/survivor_actor_frames.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `8a05278ec7b076359f05ddf000b9854a7dfcae01e0f75bddbaa4f4025822b67a` |
| `assets/textures/survivor/survivor_effects.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `d32ec7a561c3bdc856af7ed534f0d084ae118f7151bd6033c20b5dfec1cba7c5` |
| `assets/textures/survivor/survivor_evolutions.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `8845c7f6a94ba3efec66ddf5b1c3fb5da49a113436e13f4b047a1da9386b889c` |
| `assets/textures/survivor/survivor_icons.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `31d7a1b50c4a214957625e4b3938289147673996332ffaf15fa8ac01e320c372` |
| `assets/textures/survivor/survivor_passives.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `b53e24428528c51350defcb430debdbb018d932ee5ae3933569ef89dc07678e6` |
| `assets/textures/survivor/survivor_powerups.png` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `d266df248806cdf966ba199d30acfaf7867d0978a868f0b0c844936da5d12634` |
| `assets/textures/survivor/menu/title_backdrop_v2.png` | local AI generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `07ed6357d9efc45fd4c32c957809962a0cfd5ccd35cca94d66389070fa9c927e` |
| `assets/textures/survivor/menu/title_logo_plaque_v3.png`, `menu_button_*_{ko,en}.png` | local AI generated asset | 프로젝트 내부 placeholder | 필요 없음 | Text UI alpha matte applied with `scripts/make_text_ui_transparent.py`. SHA-256: `title_logo_plaque_v3.png` `b12dfec922bfb7e4d1e1b87b74dc38ea116bd240fe87dc158c35a3a5164cc3ea`, `menu_button_start_ko.png` `14ebbf2e8b28ef0608f54c7e90a4cda37f0af5d8d386a92628dcf0ffdf2ee811`, `menu_button_start_en.png` `20ac804a2e445e90dac6bafc2565d81689e21ae183e82811bfe192e366cd3342`, `menu_button_character_ko.png` `a8143b7d992a14dc600d40d3629e32dc30b7f0bf888cfcc96d5aa8cf9c8e3cc0`, `menu_button_character_en.png` `707c5a2418a87009925979f7730e7b1365725e24777721dd60f9f6405c15b102`, `menu_button_stage_ko.png` `054d65558346c244f74db9ce59740dd26bb92bf5a963aae182dbe0a8007c7014`, `menu_button_stage_en.png` `b26dc4c76878e7c9afcbe3c1b90979f2de856a1d974f38f12e1c2b6bf02aa841`, `menu_button_shop_ko.png` `203f1400de4d0e0d07224f593564c88657a547ce3347e1b168bfdc1b9098efb1`, `menu_button_shop_en.png` `687763fe015b6bef17f82ee9688904c0dc3f8555c3c2db6a06644b4e20457179`, `menu_button_achievements_ko.png` `4e0884a7c4a51fdbc2b70b115be038d9a9e1f0820a128c98fa3e8ca2bde56579`, `menu_button_achievements_en.png` `f29bb780297778885b0acbdd7cdf7921ffe31b0cd958444163744397e2685a42`, `menu_button_settings_ko.png` `e71721c3b0ef709a7bbd40ff94efd89633313ea411b0d1832bb97e0ef78375cc`, `menu_button_settings_en.png` `daf40164268c0b01ae981ae65c01af05fe5e3cac303995b9ab73ea64c38ef22d` |
| `assets/textures/survivor/ui/*.png` | local AI generated asset | 프로젝트 내부 placeholder | 필요 없음 | Image generation + local chroma-key alpha cleanup. SHA-256: `ui_modal_panel.png` `464cf041bd2565f60a49f8eec73c1dc4b22509675b4584adeae7c3c0446044ec`, `ui_slot_frame.png` `8235dcbcf0155f5a8a1146d039eaabf05316e2551d6c6d155f8f5056a1a5e41b` |
| `assets/textures/survivor/ui/ingame_label_*.png`, `levelup_title_*.png`, `gameover_title_*.png`, `restart_hint_*.png`, `section_*.png` | local AI generated asset | 프로젝트 내부 placeholder | 필요 없음 | Text UI alpha matte applied with `scripts/make_text_ui_transparent.py`. SHA-256: `ingame_label_gold_en.png` `fa9e161277cdd8936452be873c35e340d5cdda3450cb67f928c3245e6a541cb5`, `ingame_label_gold_ko.png` `fcf0ff8c43f884675e70432e7e0b10df2af03aadfbd699fa6afcf89a2da46fbb`, `ingame_label_hp_en.png` `96c99b767eb29abf96655cf04ca7ebb80091d213767f7cca959bfd9c5c02dad8`, `ingame_label_hp_ko.png` `96c99b767eb29abf96655cf04ca7ebb80091d213767f7cca959bfd9c5c02dad8`, `ingame_label_kills_en.png` `ebd45db9d1ea451b1210a57198b1d0f72a7aa5fea15e82d72fb2510f58c61b55`, `ingame_label_kills_ko.png` `67b82ffc1d48c1f558080a1e592d0e07aa839336983c4b8edd51ebdeb9fb39fa`, `ingame_label_lv_en.png` `74b30f2f672172781c41a0aa696e9cbc433fc9c047a8d8bcb0eed099297b462c`, `ingame_label_lv_ko.png` `74b30f2f672172781c41a0aa696e9cbc433fc9c047a8d8bcb0eed099297b462c`, `ingame_label_passives_en.png` `8b1d41f57320712564b388214a2934e999b3113bb6fd90bbd6b1ed72324b16af`, `ingame_label_passives_ko.png` `9a22b4c7c1bb89d1e1618bad60cb43e1962461451d564a65b66af09f3504c67f`, `ingame_label_xp_en.png` `debe9749c541d49bb3b657c039ba08a9fbb1f9b67b4c3996246309081c8455bb`, `ingame_label_xp_ko.png` `debe9749c541d49bb3b657c039ba08a9fbb1f9b67b4c3996246309081c8455bb`, `levelup_title_en.png` `0713c1b2ff80909c37d601488cf310b7cf198fa8566a8910aaea05f00c8f72aa`, `levelup_title_ko.png` `9777e4db731483ffcb7d53b0dbde72eefc889119ac8ca0f4f3b5234a8754a573`, `gameover_title_en.png` `ee0449a1009ca67483697072ca539cbd2aa75eb30502ab016d66d4bb731658c0`, `gameover_title_ko.png` `5a57ab233be388943f406120d3bc4e10d3e8f645a36063ba5338e99c1cb78838`, `restart_hint_en.png` `000a16c950cddebc7f89b21af9e6769354fdfa5fe13c973229859424b23c15b7`, `restart_hint_ko.png` `8b055230cfe34be8927fc1eb484d82f01bfcc8f546e06010e90c8a9648f87d5b`, `section_passives_en.png` `15c2a1d92205e002d5f538942b2c11ea69c401d15e8f3b7971c19270a6896eec`, `section_passives_ko.png` `d75ef8be1e369d15bbc950913db723717d7796a3b5452d9617323c853ccfe007`, `section_weapons_en.png` `c3d6145932f28933142fbf6030d783d552812b0983fa4fc1568dc08f2d69418c`, `section_weapons_ko.png` `271b37e85ec11e73ef327a217f8b0005c751d330334056b980dec0fac7b378b5` |

## Audio

| 파일 | 출처 URL | 라이선스 | 저작자 표기 | 수정 여부 |
|---|---|---|---|---|
| `assets/audio/rustsurvivors title1.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `71e5c0441065a411900a27f24a4ee880d822e2ae333ef4cb9b6395da5191358f` |
| `assets/audio/rustsurvivors title2.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `1ed02ac93e51183d74bcddd2f27b02d87ea4e9a20ba1e3a7bf8319607686e15c` |
| `assets/audio/rustsurvivors ingame1.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `ac45999702e683f70a0fc958d7decc7ae57f3aae265356ad8dc9b5cf7f03b40e` |
| `assets/audio/rustsurvivors ingame2.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `f77ed39980e2d4541763ffbfeb69e14009450b9bd9936151ebb52033d4cfde92` |
| `assets/audio/rustsurvivors boss1.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `4d68b7650009212c896ebf26fe3201f7ef6f17bcd96890aea28c6a5f6ab025dd` |
| `assets/audio/rustsurvivors boss2.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `a7b8f455deb36221a0ec81e641746e27b2d5f805f23f106ba5e1f47670d56da8` |
| `assets/audio/rustsurvivors stageclear1.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `13f00234a7c5538614726411aaa22e565fb49721277cd9275f1efdfb269dacd6` |
| `assets/audio/rustsurvivors stageclear2.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `828ae2ff135a3368bed6e1d0a0158c09b0ebd2c024538d55d1692491d1570c78` |
| `assets/audio/rustsurvivors gameover1.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `bd31f60dac478e40a7a8afeb85dbb8d824c34b1974fd4a9283f4f074c58f2353` |
| `assets/audio/rustsurvivors gameover2.mp3` | user supplied generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `0fbf25938092856756d13a3627a3b823167c3fbb918a317b701fdb5729ffe087` |
| `assets/audio/sfx_enemy_hit.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `0c98145123dfb1e3309871307473cce15a5b34de74839568687f38f55ff7758a` |
| `assets/audio/sfx_enemy_die.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `0f3b07df8c9325030d16c17a7451f416a818a54ef12a0c8586d1e5cab2fe82a3` |
| `assets/audio/sfx_player_hit.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `7eef147bf165f74ad07cd690c06c8cfb0bb3d21867f1968865c198cbf4562ae4` |
| `assets/audio/sfx_levelup.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `052734d582b77d58847678cec2c65a07896a0d71e9d970e19280674610903e27` |
| `assets/audio/sfx_xp.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `c60eecd723be6abbc7f7d4d313743d72062d456472f50a88342a54fb3555e81c` |
| `assets/audio/sfx_pickup.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `591a50277d8f142fbad35708422c32d53949b852b22765902247e2ddf521dcca` |
| `assets/audio/sfx_bomb.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `b82315b6eeb2749bcdac45b38818e3c937b3f2fa053a7e408b5ba5a528450826` |
| `assets/audio/sfx_chest_open.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `7e2f929838ad69bbb5414a9851fe76ab07ba2b855c489143cee9c0722f50e75f` |
| `assets/audio/sfx_boss_appear.wav` | local generated asset | 프로젝트 내부 placeholder | 필요 없음 | SHA-256 `994ad77646b09d4abb4bded3f90f1a1fdab4d34d75f9e18abd0aa64ede4ad0d2` |

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
