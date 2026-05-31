# UI Title Style Prompt

Use this prompt when regenerating Rust Survivors title-style UI image assets such as level-up, game-over, restart, section, or menu plaque text.

## Reference Assets

- `assets/textures/survivor/menu/title_logo_plaque_v3.png`
- `assets/textures/survivor/ui/restart_hint_ko.png`
- `assets/textures/survivor/ui/section_weapons_ko.png`
- `assets/textures/survivor/ui/ingame_label_lv_ko.png`

## Prompt Template

```text
Create a fantasy roguelite game UI title plaque asset reading exactly "<TEXT>".

Use case: stylized-concept
Asset type: Rust Survivors UI title plaque
Style reference to match: dark cracked black stone plaque, ornate aged bronze/gold gothic frame, red glowing diamond gems at the sides and center, high-contrast gold beveled lettering with rough hammered metal texture, strong dark drop shadow, premium fantasy action game UI.
Critical typography requirement: the letters must not look horizontally stretched or squashed. Use natural character proportions and readable spacing.
Composition: centered text, enough side padding, horizontal plaque format, transparent outer background if possible. Keep the plaque within a 760x190 frame.
Avoid: flat vector look, plain simple border, modern sans-serif UI, cartoon style, blurry text, distorted text, extra words, watermark.
```

## Text Notes

- Korean level-up title: `레벨 업`
- English level-up title: `LEVEL UP`
- Korean game-over title: `게임 오버`
- English game-over title: `GAME OVER`
- Do not add exclamation marks unless the target asset explicitly needs one.
- Keep generated source files, then crop/remove checker backgrounds and save final game assets as `760x190` RGBA PNGs.
