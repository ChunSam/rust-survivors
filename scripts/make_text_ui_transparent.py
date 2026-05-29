#!/usr/bin/env python3
"""Apply transparent mattes to generated text UI PNGs.

The generated logo-style PNGs intentionally keep their dark stone plaque, but
the surrounding black canvas should be transparent in-game. This script uses
the known asset dimensions to keep the plaque body and visible metal/gem detail
while clearing the outer canvas.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter


ROOT = Path(__file__).resolve().parents[1]
TARGETS = [
    ROOT / "assets/textures/survivor/menu/title_logo_plaque_v3.png",
    *sorted((ROOT / "assets/textures/survivor/menu").glob("menu_button_*_ko.png")),
    *sorted((ROOT / "assets/textures/survivor/menu").glob("menu_button_*_en.png")),
    *sorted((ROOT / "assets/textures/survivor/ui").glob("ingame_label_*.png")),
    *sorted((ROOT / "assets/textures/survivor/ui").glob("levelup_title_*.png")),
    *sorted((ROOT / "assets/textures/survivor/ui").glob("gameover_title_*.png")),
    *sorted((ROOT / "assets/textures/survivor/ui").glob("restart_hint_*.png")),
    *sorted((ROOT / "assets/textures/survivor/ui").glob("section_*.png")),
]


def draw_plaque_polygon(draw: ImageDraw.ImageDraw, box: tuple[int, int, int, int], cut: int) -> None:
    left, top, right, bottom = box
    draw.polygon(
        [
            (left + cut, top),
            (right - cut, top),
            (right, top + cut),
            (right, bottom - cut),
            (right - cut, bottom),
            (left + cut, bottom),
            (left, bottom - cut),
            (left, top + cut),
        ],
        fill=255,
    )


def plaque_mask(size: tuple[int, int]) -> Image.Image:
    width, height = size
    mask = Image.new("L", size, 0)
    draw = ImageDraw.Draw(mask)

    if size == (2172, 724):
        draw_plaque_polygon(draw, (35, 105, width - 35, height - 105), 95)
    elif size == (1600, 520):
        draw_plaque_polygon(draw, (25, 86, width - 25, height - 86), 78)
    elif size == (760, 190):
        draw_plaque_polygon(draw, (6, 36, width - 6, height - 36), 38)
    elif size == (620, 126):
        draw_plaque_polygon(draw, (-4, 20, width + 4, height - 20), 28)
    elif size == (340, 86):
        draw_plaque_polygon(draw, (14, 10, width - 14, height - 10), 22)
    elif size == (240, 78):
        draw_plaque_polygon(draw, (23, 15, width - 23, height - 15), 17)
    else:
        inset_x = max(0, width // 24)
        inset_y = max(0, height // 6)
        cut = max(4, min(width, height) // 5)
        draw_plaque_polygon(draw, (inset_x, inset_y, width - inset_x, height - inset_y), cut)

    return mask


def keep_visible_outer_detail(image: Image.Image, mask: Image.Image) -> Image.Image:
    pixels = image.convert("RGB").load()
    keep = mask.load()
    width, height = image.size

    for y in range(height):
        for x in range(width):
            if keep[x, y]:
                continue

            red, green, blue = pixels[x, y]
            luma = 0.299 * red + 0.587 * green + 0.114 * blue
            spread = max(red, green, blue) - min(red, green, blue)
            is_gold = red > 90 and green > 52 and blue < 58 and spread > 34
            is_red_gem = red > 85 and red > green * 1.25 and red > blue * 1.25
            is_highlight = luma > 98 and spread > 24

            if is_gold or is_red_gem or is_highlight:
                keep[x, y] = 255

    return mask


def apply_alpha(path: Path) -> None:
    image = Image.open(path).convert("RGBA")
    mask = keep_visible_outer_detail(image, plaque_mask(image.size))
    mask = mask.filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.GaussianBlur(0.5))
    image.putalpha(mask)
    image.save(path)


def main() -> None:
    for target in TARGETS:
        apply_alpha(target)
        print(target.relative_to(ROOT))


if __name__ == "__main__":
    main()
