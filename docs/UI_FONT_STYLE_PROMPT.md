# UI Font Style Prompt

작성일: 2026-05-30

이 문서는 현재 Rust Survivors PNG UI 이미지에 들어간 글자 스타일을 분석해, 게임 전용 TTF/display font 제작 프롬프트로 옮기기 위한 기준이다.
목표는 PNG 플라크나 금속 질감을 다시 만드는 것이 아니라, 같은 분위기를 가진 깨끗한 벡터 폰트 외곽선을 정의하는 것이다.

## Reference Assets

Primary references:

- `assets/textures/survivor/menu/title_logo_plaque_v3.png`
- `assets/textures/survivor/menu/menu_button_start_ko.png`
- `assets/textures/survivor/menu/menu_button_start_en.png`
- `assets/textures/survivor/ui/levelup_title_ko.png`
- `assets/textures/survivor/ui/ingame_label_passives_en.png`

Secondary references:

- `assets/textures/survivor/menu/menu_button_*_{ko,en}.png`
- `assets/textures/survivor/ui/levelup_title_en.png`
- `assets/textures/survivor/ui/gameover_title_{ko,en}.png`
- `assets/textures/survivor/ui/ingame_label_*_{ko,en}.png`
- `assets/textures/survivor/ui/section_*_{ko,en}.png`
- `assets/textures/survivor/ui/restart_hint_*_{ko,en}.png`

## Observed Letter Style

Font skeleton to keep:

- Heavy display typeface for dark fantasy roguelite UI, closer to carved metal title lettering than body text.
- High-contrast strokes: thick vertical/main strokes with thinner internal cuts and sharp highlight-like counters.
- Sharp wedge serifs and blade-like stroke terminals; many ends feel chiseled, pointed, or axe-cut rather than rounded.
- Compact, plaque-friendly proportions with wide title presence, tight but readable spacing, and strong centered silhouettes.
- Hangul glyphs are blocky and monumental, with angular brush/blackletter influence, narrow internal whitespace, and strong vertical weight.
- Latin glyphs are gothic fantasy roman capitals with flared serifs, engraved-book-title proportions, and medieval metal-signage character.
- Numerals and symbols should follow the Latin style: high contrast, sharp terminals, strong silhouette, readable in HUD sizes.

Image rendering effects to exclude from the TTF:

- Gold metallic fill, bevel, inner glow, top highlight, drop shadow, black outline, cracks, scratches, hammered metal texture.
- Stone plaque, bronze frame, red gems, background panels, transparent PNG matting, baked lighting.
- Any raster-only distortion, fake depth, or texture noise that should be applied later by the game renderer or image post-process.

## Unified TTF Prompt

```text
Design a clean vector TTF display font for a dark fantasy roguelite game UI font, inspired by gothic metal plaque lettering.

The font should have a heavy display typeface skeleton with sharp wedge serifs / blade-like stroke terminals, chiseled angular cuts, high-contrast strokes, and a carved medieval metal title-lettering feel. It must support both Hangul and Latin characters with a coherent shared style. Hangul glyphs should be blocky, monumental, angular, and Hangul-compatible, readable at small HUD sizes, with clear syllable structure and not overly compressed counters. Latin uppercase, lowercase, numerals, and punctuation should feel like gothic fantasy roman lettering with flared wedge serifs, strong vertical weight, and readable spacing.

Create not texture, not logo rendering, clean vector font outlines. Do not bake in color, bevels, shadows, highlights, cracks, gems, stone plaques, or background frames. The font should work as a plain monochrome outline font that can later receive gold material, outline, shadow, and UI effects in the game renderer.
```

의도:

- 한글/영문을 같은 폰트 패밀리로 묶기 위한 통합 지시문이다.
- PNG의 금속 질감은 제외하고, 획 모양과 글자 실루엣만 TTF로 옮긴다.
- 실제 게임 HUD에서도 쓸 수 있도록 작은 크기 가독성을 명시한다.

## Hangul Glyph Prompt

```text
Design Hangul glyphs for a dark fantasy roguelite game UI font, matching gothic metal plaque lettering.

The Hangul should be a heavy display typeface with blocky monumental syllable shapes, angular brush-like construction, sharp blade-like stroke terminals, chiseled wedge cuts, and strong vertical weight. Keep each syllable clearly readable and structurally correct. Preserve enough inner whitespace for small HUD labels while maintaining a dense carved-metal title feel. Avoid soft rounded terminals; use pointed, beveled-looking outline shapes only as clean vector contours.

Create not texture, not logo rendering, clean vector font outlines. No baked gold material, bevel, shadow, cracks, highlights, gems, stone plaque, or background frame.
```

의도:

- `시작`, `레벨 업`, `사망`, `무기`, `패시브` 같은 한글 UI에 맞춘 지시문이다.
- 한글은 장식성을 높이면 쉽게 읽기 어려워지므로, "syllable structure"와 "inner whitespace"를 강하게 요구한다.
- 붓글씨처럼 흐르는 서체보다 각지고 조각된 게임 타이틀 글자를 목표로 한다.

## Latin, Numerals, And Symbols Prompt

```text
Design Latin letters, numerals, and UI punctuation for a dark fantasy roguelite game UI font, matching gothic metal plaque lettering.

The Latin alphabet should be a heavy display typeface with gothic fantasy roman proportions, sharp wedge serifs, blade-like terminals, high-contrast strokes, chiseled cuts, and a carved medieval metal signage feel. Uppercase should be especially strong for menu buttons and title plaques. Lowercase should remain readable for labels if needed. Numerals should be compact, clear, and HUD-readable, matching the same sharp serif construction. Punctuation and symbols should be simple enough for UI use but share the same angular carved style.

Create not texture, not logo rendering, clean vector font outlines. No baked gold material, bevel, shadow, cracks, highlights, gems, stone plaque, or background frame.
```

의도:

- `RUST SURVIVORS`, `START`, `PASSIVES`, `LEVEL UP`, 숫자 HUD에 맞춘 지시문이다.
- 대문자 타이틀감은 유지하되, 숫자와 기호는 게임 UI에서 즉시 읽히도록 단순하게 제한한다.

## Negative Prompt

```text
Avoid overly thin strokes, cute/rounded sans-serif, modern flat UI font, messy pseudo letters, fake unreadable Hangul, excessive calligraphy, unreadable blackletter, overly condensed glyphs, stretched or squashed letters, inconsistent Hangul and Latin style, baked-in shadows, gradients, bevels, cracks, jewels, stone texture, metal texture, background frames, logo composition, raster texture, noisy outlines, decorative details that break small-size readability.
```

의도:

- 생성 모델이 PNG 이미지를 그대로 따라 하면서 질감/그림자/프레임까지 폰트에 넣는 것을 막는다.
- 특히 한글 오탈자, 가짜 문자, 과한 블랙레터화를 방지한다.

## Production Notes

- TTF 제작 단계에서는 흑백 벡터 외곽선만 확정한다.
- 금색 금속 채움, 검은 외곽선, 그림자, 균열, 하이라이트는 별도 렌더링 스타일 또는 후처리로 적용한다.
- Regular 하나만 만들기보다 `Display Heavy`를 기본으로 두고, HUD용으로 내부 공간을 더 확보한 `Display Medium` 변형을 고려한다.
- 우선 검수 문자열은 `Rust Survivors`, `START`, `PASSIVES`, `LEVEL UP`, `시작`, `레벨 업`, `사망`, `무기`, `패시브`, `0123456789`로 잡는다.
