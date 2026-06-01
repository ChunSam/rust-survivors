use glam::Vec2;

use super::meta::HudDetail;

#[cfg(test)]
pub(crate) const UI_PRESET_RESOLUTIONS: [(f32, f32); 4] = [
    (800.0, 600.0),
    (1280.0, 720.0),
    (1600.0, 900.0),
    (1920.0, 1080.0),
];

const SLOT_COLS: usize = 6;
const HUD_SLOT_W: f32 = 152.0;
const HUD_SLOT_H: f32 = 38.0;
const HUD_SLOT_GAP: f32 = 6.0;
const XP_BAR_H: f32 = 14.0;
const HUD_ICON_SIZE: f32 = 28.0;
const HUD_SLOT_TEXT_INSET: f32 = 46.0;
const LEVEL_ICON_SIZE: f32 = 34.0;
const SHOP_ICON_SIZE: f32 = 22.0;
const LEVELUP_ICON_X_OFFSET: f32 = -214.0;
const LEVELUP_TEXT_X_OFFSET: f32 = -178.0;
const MODAL_PANEL_ASPECT: f32 = 1478.0 / 756.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScreenRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

impl ScreenRect {
    pub(crate) const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub(crate) fn from_tuple(rect: (f32, f32, f32, f32)) -> Self {
        Self::new(rect.0, rect.1, rect.2, rect.3)
    }

    pub(crate) fn right(self) -> f32 {
        self.x + self.w
    }

    pub(crate) fn bottom(self) -> f32 {
        self.y + self.h
    }

    pub(crate) fn center(self) -> Vec2 {
        Vec2::new(self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    pub(crate) fn inflated(self, x: f32, y: f32) -> Self {
        Self {
            x: self.x - x,
            y: self.y - y,
            w: self.w + x * 2.0,
            h: self.h + y * 2.0,
        }
    }

    pub(crate) fn inset(self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            w: (self.w - x * 2.0).max(0.0),
            h: (self.h - y * 2.0).max(0.0),
        }
    }

    pub(crate) fn aspect_fit(self, aspect: f32) -> Self {
        let rect_aspect = self.w / self.h.max(1.0);
        if rect_aspect > aspect {
            let w = self.h * aspect;
            Self {
                x: self.x + (self.w - w) * 0.5,
                w,
                ..self
            }
        } else {
            let h = self.w / aspect.max(0.001);
            Self {
                y: self.y + (self.h - h) * 0.5,
                h,
                ..self
            }
        }
    }

    pub(crate) fn aspect_fill(self, aspect: f32) -> Self {
        let rect_aspect = self.w / self.h.max(1.0);
        if rect_aspect > aspect {
            let h = self.w / aspect.max(0.001);
            Self {
                y: self.y + (self.h - h) * 0.5,
                h,
                ..self
            }
        } else {
            let w = self.h * aspect;
            Self {
                x: self.x + (self.w - w) * 0.5,
                w,
                ..self
            }
        }
    }

    pub(crate) fn contains_point(self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    #[cfg(test)]
    pub(crate) fn contains_rect(self, rect: Self) -> bool {
        rect.x >= self.x
            && rect.y >= self.y
            && rect.right() <= self.right()
            && rect.bottom() <= self.bottom()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct UiLayout {
    pub(crate) viewport_w: f32,
    pub(crate) viewport_h: f32,
    pub(crate) compact: bool,
}

impl UiLayout {
    pub(crate) fn new(viewport_w: f32, viewport_h: f32) -> Self {
        Self {
            viewport_w,
            viewport_h,
            compact: viewport_w <= 900.0 || viewport_h <= 640.0,
        }
    }

    pub(crate) fn viewport_rect(self) -> ScreenRect {
        ScreenRect::new(0.0, 0.0, self.viewport_w, self.viewport_h)
    }

    pub(crate) fn centered_rect(self, w: f32, h: f32) -> ScreenRect {
        ScreenRect::new(
            self.viewport_w * 0.5 - w * 0.5,
            self.viewport_h * 0.5 - h * 0.5,
            w,
            h,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ModalLayout {
    pub(crate) panel: ScreenRect,
    pub(crate) content: ScreenRect,
    pub(crate) title_baseline: Vec2,
    pub(crate) footer_baseline: Vec2,
}

impl ModalLayout {
    pub(crate) fn centered(
        viewport_w: f32,
        viewport_h: f32,
        max_w: f32,
        max_h: f32,
        top_bias: f32,
    ) -> Self {
        let margin_x = if viewport_w <= 900.0 { 28.0 } else { 72.0 };
        let margin_y = if viewport_h <= 640.0 { 18.0 } else { 34.0 };
        let bounds = ScreenRect::new(
            0.0,
            0.0,
            max_w.min(viewport_w - margin_x * 2.0).max(360.0),
            max_h.min(viewport_h - margin_y * 2.0).max(260.0),
        );
        let fitted = bounds.aspect_fit(MODAL_PANEL_ASPECT);
        let panel_w = fitted.w;
        let panel_h = fitted.h;
        let x = viewport_w * 0.5 - panel_w * 0.5;
        let centered_y = viewport_h * 0.5 - panel_h * 0.5;
        let y = (centered_y + top_bias).clamp(margin_y, viewport_h - margin_y - panel_h);
        let panel = ScreenRect::new(x, y, panel_w, panel_h);
        let content = panel.inset(56.0, 58.0);
        Self {
            panel,
            content,
            title_baseline: Vec2::new(content.x, panel.y + 42.0),
            footer_baseline: Vec2::new(content.x, panel.bottom() - 28.0),
        }
    }

    pub(crate) fn title_center_x(self) -> f32 {
        self.panel.center().x
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MenuListLayout {
    pub(crate) modal: ModalLayout,
    pub(crate) row_count: usize,
    pub(crate) row_step: f32,
    pub(crate) row_h: f32,
    pub(crate) font_size: f32,
    pub(crate) first_row_y: f32,
    pub(crate) footer_baseline: Vec2,
}

impl MenuListLayout {
    pub(crate) fn new(
        viewport_w: f32,
        viewport_h: f32,
        row_count: usize,
        max_panel_w: f32,
        max_panel_h: f32,
        top_bias: f32,
        preferred_row_step: f32,
        preferred_font_size: f32,
    ) -> Self {
        let modal =
            ModalLayout::centered(viewport_w, viewport_h, max_panel_w, max_panel_h, top_bias);
        let compact = viewport_w <= 900.0 || viewport_h <= 640.0;
        let list_top = modal.content.y + 34.0;
        let list_bottom = if compact {
            modal.panel.bottom() - 116.0
        } else {
            modal.panel.bottom() - 84.0
        };
        let available_h = (list_bottom - list_top).max(row_count as f32 * 24.0);
        let row_step = preferred_row_step
            .min(available_h / row_count.max(1) as f32)
            .max(if compact { 20.0 } else { 24.0 });
        let row_h = (row_step - 4.0).max(20.0);
        let font_size =
            preferred_font_size
                .min(row_h * 0.62)
                .max(if compact { 13.0 } else { 14.0 });
        let footer_baseline = Vec2::new(
            modal.content.x,
            (modal.panel.bottom() + 22.0).min(viewport_h - 20.0),
        );
        Self {
            modal,
            row_count,
            row_step,
            row_h,
            font_size,
            first_row_y: list_top,
            footer_baseline,
        }
    }

    pub(crate) fn row_rect(self, index: usize) -> ScreenRect {
        ScreenRect::new(
            self.modal.content.x,
            self.first_row_y + index.min(self.row_count.saturating_sub(1)) as f32 * self.row_step,
            self.modal.content.w,
            self.row_h,
        )
    }

    #[cfg(test)]
    pub(crate) fn selection_rect(self, index: usize) -> ScreenRect {
        self.row_rect(index).inflated(10.0, 5.0)
    }

    pub(crate) fn text_pos(self, index: usize) -> Vec2 {
        let row = self.row_rect(index);
        Vec2::new(row.x + 16.0, row.y + (row.h - self.font_size) * 0.5)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TopHudLayout {
    pub(crate) ui_scale: f32,
    pub(crate) compact: bool,
    pub(crate) detail: HudDetail,
    pub(crate) panel: ScreenRect,
    pub(crate) hp_bar: ScreenRect,
    pub(crate) hp_text_pos: Vec2,
    pub(crate) primary_y: f32,
    pub(crate) secondary_y: f32,
    pub(crate) stats_y: f32,
    pub(crate) info_font_size: f32,
    pub(crate) stat_font_size: f32,
}

impl TopHudLayout {
    pub(crate) fn new(viewport_w: f32, viewport_h: f32, detail: HudDetail) -> Self {
        let ui_scale = responsive_ui_scale(viewport_w, viewport_h);
        let compact = viewport_w <= 900.0 || viewport_h <= 640.0;
        let x = 16.0 * ui_scale;
        let y = 14.0 * ui_scale;
        let hp_w = if compact { 210.0 } else { 280.0 } * ui_scale;
        let hp_h = if compact { 22.0 } else { 24.0 } * ui_scale;
        let rows = match (compact, detail) {
            (true, HudDetail::Detailed) => 4.0,
            (_, HudDetail::Detailed) => 3.0,
            _ => 2.0,
        };
        let panel_w = match (compact, detail) {
            (true, HudDetail::Detailed) => 520.0,
            (true, _) => 560.0,
            (_, HudDetail::Detailed) => 840.0,
            _ => 760.0,
        } * ui_scale;
        let panel_h = (26.0 + rows * 25.0) * ui_scale;
        let panel = ScreenRect::new(
            x - 14.0 * ui_scale,
            y - 12.0 * ui_scale,
            panel_w.min(viewport_w - 18.0 * ui_scale),
            panel_h,
        );
        let hp_bar = ScreenRect::new(x, y, hp_w.min(panel.w - 150.0 * ui_scale), hp_h);
        let info_font_size = match (compact, detail) {
            (true, HudDetail::Detailed) => 19.0,
            (true, _) => 18.0,
            _ => 18.0,
        } * ui_scale;
        let stat_font_size = if compact { 15.5 } else { 16.5 } * ui_scale;
        Self {
            ui_scale,
            compact,
            detail,
            panel,
            hp_bar,
            hp_text_pos: Vec2::new(hp_bar.right() + 10.0 * ui_scale, hp_bar.y),
            primary_y: hp_bar.bottom() + 9.0 * ui_scale,
            secondary_y: hp_bar.bottom() + 33.0 * ui_scale,
            stats_y: hp_bar.bottom() + if compact { 56.0 } else { 55.0 } * ui_scale,
            info_font_size,
            stat_font_size,
        }
    }

    pub(crate) fn bottom(self) -> f32 {
        self.panel.bottom()
    }

    pub(crate) fn boss_bar_y(self) -> f32 {
        self.bottom() + 36.0 * self.ui_scale
    }
}

pub(crate) fn responsive_ui_scale(viewport_w: f32, viewport_h: f32) -> f32 {
    (viewport_w / 1280.0)
        .min(viewport_h / 720.0)
        .clamp(0.72, 1.5)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HudSlotLayout {
    pub(crate) ui_scale: f32,
    slot_x0: f32,
    slot_w: f32,
    slot_h: f32,
    slot_gap: f32,
    weapon_y: f32,
    passive_y: f32,
    pub(crate) icon_size: f32,
}

impl HudSlotLayout {
    pub(crate) fn new(viewport_w: f32, viewport_h: f32) -> Self {
        let ui_scale = responsive_ui_scale(viewport_w, viewport_h);
        let slot_x0 = 16.0 * ui_scale;
        let slot_gap = HUD_SLOT_GAP * ui_scale;
        let slot_h = HUD_SLOT_H * ui_scale;
        let max_panel_w = (viewport_w - slot_x0 * 2.0).max(300.0);
        let slot_w = (HUD_SLOT_W * ui_scale)
            .min((max_panel_w - (SLOT_COLS - 1) as f32 * slot_gap) / SLOT_COLS as f32);
        let weapon_y = viewport_h - XP_BAR_H * ui_scale - slot_h * 2.0 - slot_gap - 16.0 * ui_scale;
        let passive_y = weapon_y + slot_h + slot_gap;
        Self {
            ui_scale,
            slot_x0,
            slot_w,
            slot_h,
            slot_gap,
            weapon_y,
            passive_y,
            icon_size: (HUD_ICON_SIZE * ui_scale).min(32.0),
        }
    }

    pub(crate) fn panel_rect(self) -> ScreenRect {
        ScreenRect::new(
            self.slot_x0 - 4.0,
            self.weapon_y - 26.0 * self.ui_scale,
            SLOT_COLS as f32 * self.slot_w + (SLOT_COLS - 1) as f32 * self.slot_gap + 8.0,
            self.slot_h * 2.0 + self.slot_gap + 36.0 * self.ui_scale,
        )
    }

    pub(crate) fn slot_rect(self, slot_index: usize, row: usize) -> ScreenRect {
        let x = self.slot_x0 + slot_index as f32 * (self.slot_w + self.slot_gap);
        let y = if row == 0 {
            self.weapon_y
        } else {
            self.passive_y
        };
        ScreenRect::new(x, y, self.slot_w, self.slot_h)
    }

    pub(crate) fn slot_skin_rect(self, slot_index: usize, row: usize) -> ScreenRect {
        self.slot_rect(slot_index, row)
            .inflated(4.0 * self.ui_scale, 5.0 * self.ui_scale)
    }

    pub(crate) fn stripe_rect(self, slot_index: usize, row: usize) -> ScreenRect {
        let slot = self.slot_rect(slot_index, row);
        ScreenRect::new(slot.x, slot.y, 7.0 * self.ui_scale, slot.h)
    }

    pub(crate) fn icon_center(self, slot_index: usize, row: usize) -> Vec2 {
        let slot = self.slot_rect(slot_index, row);
        Vec2::new(slot.x + 22.0 * self.ui_scale, slot.y + slot.h * 0.5)
    }

    pub(crate) fn text_x(self, slot_index: usize) -> f32 {
        self.slot_rect(slot_index, 0).x + HUD_SLOT_TEXT_INSET * self.ui_scale
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LevelUpLayout {
    viewport_w: f32,
    viewport_h: f32,
}

impl LevelUpLayout {
    pub(crate) fn new(viewport_w: f32, viewport_h: f32) -> Self {
        Self {
            viewport_w,
            viewport_h,
        }
    }

    pub(crate) fn panel_rect(self) -> ScreenRect {
        UiLayout::new(self.viewport_w, self.viewport_h).centered_rect(500.0, 300.0)
    }

    pub(crate) fn panel_skin_rect(self) -> ScreenRect {
        self.panel_rect().inflated(24.0, 20.0)
    }

    pub(crate) fn card_rect(self, card_index: usize) -> ScreenRect {
        let center = UiLayout::new(self.viewport_w, self.viewport_h)
            .viewport_rect()
            .center();
        ScreenRect::new(
            center.x - 235.0,
            center.y - 58.0 + card_index as f32 * 56.0,
            470.0,
            46.0,
        )
    }

    pub(crate) fn icon_center(self, card_index: usize) -> Vec2 {
        let center = UiLayout::new(self.viewport_w, self.viewport_h)
            .viewport_rect()
            .center();
        Vec2::new(
            center.x + LEVELUP_ICON_X_OFFSET,
            center.y - 35.0 + card_index as f32 * 56.0,
        )
    }

    pub(crate) fn icon_size(self) -> f32 {
        (LEVEL_ICON_SIZE * responsive_ui_scale(self.viewport_w, self.viewport_h)).clamp(28.0, 36.0)
    }

    pub(crate) fn text_x(self) -> f32 {
        self.viewport_w * 0.5 + LEVELUP_TEXT_X_OFFSET
    }

    pub(crate) fn text_pos(self, card_index: usize) -> Vec2 {
        let card = self.card_rect(card_index);
        Vec2::new(self.text_x(), card.y + (card.h - self.font_size()) * 0.5)
    }

    pub(crate) fn font_size(self) -> f32 {
        (24.0 * responsive_ui_scale(self.viewport_w, self.viewport_h)).clamp(19.0, 24.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShopLayout {
    viewport_w: f32,
    viewport_h: f32,
}

impl ShopLayout {
    pub(crate) fn new(viewport_w: f32, viewport_h: f32) -> Self {
        Self {
            viewport_w,
            viewport_h,
        }
    }

    pub(crate) fn row_step(self) -> f32 {
        if self.viewport_h <= 620.0 {
            26.0
        } else {
            30.0
        }
    }

    pub(crate) fn panel_rect(self) -> ScreenRect {
        ModalLayout::centered(self.viewport_w, self.viewport_h, 1040.0, 680.0, 4.0).panel
    }

    pub(crate) fn content_rect(self) -> ScreenRect {
        self.panel_rect().inset(90.0, 86.0)
    }

    pub(crate) fn visible_rows(self) -> usize {
        let content = self.content_rect();
        let min_rows = if self.viewport_h <= 620.0 { 7 } else { 8 };
        (((content.h - 70.0) / self.row_step()).floor() as usize).clamp(min_rows, 13)
    }

    pub(crate) fn visible_start(self, cursor_index: usize, total_rows: usize) -> usize {
        let visible = self.visible_rows().min(total_rows.max(1));
        if total_rows <= visible {
            0
        } else {
            cursor_index
                .saturating_add(1)
                .saturating_sub(visible)
                .min(total_rows - visible)
        }
    }

    pub(crate) fn title_pos(self) -> Vec2 {
        let panel = self.panel_rect();
        Vec2::new(panel.center().x - 300.0, (panel.y - 38.0).max(24.0))
    }

    pub(crate) fn gold_pos(self) -> Vec2 {
        let content = self.content_rect();
        Vec2::new(
            content.right() - 200.0,
            (self.panel_rect().y - 32.0).max(30.0),
        )
    }

    pub(crate) fn selection_rect(self, row_index: usize) -> ScreenRect {
        let content = self.content_rect();
        ScreenRect::new(
            content.x,
            content.y + 28.0 + row_index as f32 * self.row_step(),
            content.w,
            (self.row_step() - 4.0).max(20.0),
        )
    }

    #[cfg(test)]
    pub(crate) fn selection_skin_rect(self, row_index: usize) -> ScreenRect {
        self.selection_rect(row_index).inflated(12.0, 7.0)
    }

    pub(crate) fn icon_center(self, row_index: usize) -> Vec2 {
        let row = self.selection_rect(row_index);
        Vec2::new(row.x + 26.0, row.y + row.h * 0.5)
    }

    pub(crate) fn icon_size(self) -> f32 {
        (SHOP_ICON_SIZE * responsive_ui_scale(self.viewport_w, self.viewport_h)).clamp(18.0, 24.0)
    }

    pub(crate) fn text_x(self) -> f32 {
        self.content_rect().x + 54.0
    }

    pub(crate) fn text_pos(self, row_index: usize) -> Vec2 {
        let row = self.selection_rect(row_index);
        Vec2::new(self.text_x(), row.y + (row.h - self.font_size()) * 0.5)
    }

    pub(crate) fn text_width(self) -> f32 {
        self.content_rect().right() - self.text_x() - 12.0
    }

    pub(crate) fn font_size(self) -> f32 {
        (if self.viewport_h <= 620.0 {
            15.0_f32
        } else {
            18.0_f32
        })
        .min((self.row_step() - 4.0) * 0.72)
        .max(13.0)
    }

    pub(crate) fn footer_pos(self) -> Vec2 {
        let panel = self.panel_rect();
        let y = if self.viewport_h <= 620.0 {
            (panel.bottom() + 34.0).min(self.viewport_h - 20.0)
        } else {
            panel.bottom() - 28.0
        };
        Vec2::new(self.content_rect().x, y)
    }
}

pub(crate) fn title_logo_rect(viewport_w: f32, viewport_h: f32) -> ScreenRect {
    let layout = UiLayout::new(viewport_w, viewport_h);
    let w = if layout.compact {
        (viewport_w - 52.0).clamp(560.0, 660.0)
    } else {
        (viewport_w * 0.76).clamp(760.0, 1040.0)
    };
    let h = if layout.compact {
        (viewport_h * 0.25).clamp(132.0, 168.0)
    } else {
        (viewport_h * 0.29).clamp(190.0, 236.0)
    };
    ScreenRect::new(
        viewport_w * 0.5 - w * 0.5,
        if layout.compact { 36.0 } else { 28.0 },
        w,
        h,
    )
}

pub(crate) fn title_start_image_rect(
    interaction_rect: ScreenRect,
    viewport_w: f32,
    viewport_h: f32,
) -> ScreenRect {
    if UiLayout::new(viewport_w, viewport_h).compact {
        interaction_rect.inflated(28.0, 18.0)
    } else {
        interaction_rect.inflated(64.0, 36.0)
    }
}

pub(crate) fn title_menu_button_image_rect(
    interaction_rect: ScreenRect,
    viewport_w: f32,
    viewport_h: f32,
) -> ScreenRect {
    if UiLayout::new(viewport_w, viewport_h).compact {
        interaction_rect.inflated(16.0, 14.0)
    } else {
        interaction_rect.inflated(24.0, 20.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_inside_viewport(rect: ScreenRect, viewport: ScreenRect) {
        assert!(
            viewport.contains_rect(rect),
            "{rect:?} should fit in {viewport:?}"
        );
    }

    #[test]
    fn responsive_ui_scale_tracks_supported_presets() {
        assert!((responsive_ui_scale(1280.0, 720.0) - 1.0).abs() < 0.001);
        assert!(responsive_ui_scale(800.0, 600.0) < 1.0);
        assert!(responsive_ui_scale(1920.0, 1080.0) > 1.0);
    }

    #[test]
    fn common_layouts_fit_supported_presets() {
        for (vw, vh) in UI_PRESET_RESOLUTIONS {
            let viewport = UiLayout::new(vw, vh).viewport_rect();
            let hud = HudSlotLayout::new(vw, vh);
            let levelup = LevelUpLayout::new(vw, vh);
            let shop = ShopLayout::new(vw, vh);
            let menu = MenuListLayout::new(vw, vh, 10, 980.0, 680.0, 0.0, 40.0, 17.0);

            assert_inside_viewport(hud.panel_rect(), viewport);
            assert_inside_viewport(levelup.panel_skin_rect(), viewport);
            assert_inside_viewport(shop.panel_rect(), viewport);
            assert_inside_viewport(shop.selection_skin_rect(0), viewport);
            assert_inside_viewport(menu.modal.panel, viewport);
            assert_inside_viewport(menu.selection_rect(0), viewport);
            assert!(menu.footer_baseline.y <= vh - 16.0);
            assert!(menu.row_rect(menu.row_count - 1).bottom() <= menu.modal.panel.bottom() - 40.0);

            for slot in 0..SLOT_COLS {
                let slot_rect = hud.slot_rect(slot, 0);
                let icon_center = hud.icon_center(slot, 0);
                assert!(slot_rect.contains_point(icon_center));
                assert!(icon_center.x + hud.icon_size * 0.5 + 4.0 <= hud.text_x(slot));
                assert!(hud.text_x(slot) < slot_rect.right());
            }

            for card in 0..3 {
                let card_rect = levelup.card_rect(card);
                let icon_center = levelup.icon_center(card);
                assert!(card_rect.contains_point(icon_center));
                assert!(icon_center.x + levelup.icon_size() * 0.5 + 10.0 <= levelup.text_x());
                assert!(card_rect.contains_point(levelup.text_pos(card)));
            }

            for row in 0..shop.visible_rows() {
                let row_rect = shop.selection_rect(row);
                let icon_center = shop.icon_center(row);
                assert_inside_viewport(row_rect, viewport);
                assert!(row_rect.contains_point(icon_center));
                assert!(icon_center.x + shop.icon_size() * 0.5 + 10.0 <= shop.text_x());
                assert!(shop.text_pos(row).y >= row_rect.y);
            }

            for detail in [HudDetail::Minimal, HudDetail::Normal, HudDetail::Detailed] {
                let top = TopHudLayout::new(vw, vh, detail);
                assert_inside_viewport(top.panel, viewport);
                assert!(top.panel.contains_rect(top.hp_bar));
                assert!(top.hp_text_pos.x > top.hp_bar.right());
                assert!(top.boss_bar_y() > top.bottom());
            }
        }
    }

    #[test]
    fn title_images_share_interaction_rect_centers() {
        let start = ScreenRect::new(200.0, 300.0, 400.0, 80.0);
        let image = title_start_image_rect(start, 1280.0, 720.0);
        assert_eq!(image.center(), start.center());

        let button = ScreenRect::new(250.0, 430.0, 180.0, 70.0);
        let image = title_menu_button_image_rect(button, 1280.0, 720.0);
        assert_eq!(image.center(), button.center());
    }
}
