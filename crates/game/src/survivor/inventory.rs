/// 무기 종류 + 종류별 파라미터.
///
/// Phase 2-A 는 Whip 만 inventory 에 마이그레이션.
/// Phase 2-B 에서 MagicWand 추가.
/// Phase 2-C 에서 Knife, Axe 추가.
/// Phase 2-D 에서 Cross, FireWand 추가.
#[derive(Debug, Clone, PartialEq)]
pub enum WeaponKind {
    Whip {
        damage:      f32,
        area_width:  f32,
        area_height: f32,
    },
    MagicWand {
        damage:           f32,
        projectile_speed: f32,
        lifetime:         f32,
        pierce:           u8,
    },
    /// 가장 가까운 적 방향으로 직선 투사체를 `amount` 개 동시 발사.
    /// `spread_radians`: `amount > 1` 일 때 부채꼴 각도 범위 (라디안).
    Knife {
        damage:          f32,
        projectile_speed: f32,
        lifetime:        f32,
        pierce:          u8,
        amount:          u8,          // 동시 발사 개수
        spread_radians:  f32,         // amount > 1 일 때 각도 분산 (라디안)
    },
    /// 위로 던져 중력으로 떨어지는 포물선 투사체. `ProjectileBehavior::Arc` 사용.
    Axe {
        damage:        f32,
        initial_speed: f32, // 위 방향 초기 속도 (px/s). velocity.y = -initial_speed.
        gravity:       f32, // 중력 가속도 (px/s^2). 양수 → 매 프레임 velocity.y 증가.
        lifetime:      f32,
        pierce:        u8,
        amount:        u8,
    },
    /// 가장 가까운 적 방향으로 발사 후 `return_at` 초 뒤 방향 반전 (부메랑).
    /// `ProjectileBehavior::Boomerang` 사용.
    Cross {
        damage:           f32,
        projectile_speed: f32,
        lifetime:         f32,
        pierce:           u8,
        amount:           u8,
        return_at:        f32, // 발사 후 이 시간(초)에 방향 반전
    },
    /// 랜덤 적을 향해 고데미지 단발 투사체를 발사. `ProjectileBehavior::Straight` 사용.
    FireWand {
        damage:           f32,
        projectile_speed: f32,
        lifetime:         f32,
        pierce:           u8,
    },
}

/// 무기 인벤토리의 한 슬롯. cooldown 기반 발화 트래킹은 슬롯이 소유한다.
/// (직전까지는 WhipSystem 이 자체 elapsed 를 보유 — 이제 slot 의 elapsed 로 이동.)
#[derive(Debug, Clone)]
pub struct WeaponSlot {
    pub kind:     WeaponKind,
    pub level:    u8,
    pub cooldown: f32,   // 발화 간격 (초)
    pub elapsed:  f32,   // 누적 시간 — cooldown 도달 시 발화 + elapsed -= cooldown
}

impl WeaponSlot {
    /// dt 누적 후 발화 가능 여부. true 면 elapsed -= cooldown 도 함께 적용.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        if self.elapsed >= self.cooldown {
            self.elapsed -= self.cooldown;
            true
        } else {
            false
        }
    }
}

/// Player 컴포넌트. 슬롯은 동적 Vec (Vampire Survivors 원작 최대 6개).
///
/// Phase 2-D: `[Option<WeaponSlot>; 6]` → `Vec<WeaponSlot>` 으로 변경.
/// None/Some 래핑 제거 — 동적 push 방식.
#[derive(Debug, Clone, Default)]
pub struct WeaponInventory {
    pub slots: Vec<WeaponSlot>,
}

impl WeaponInventory {
    /// Phase 2-D 스타터 로드아웃: Whip + MagicWand + Knife + Axe + Cross + FireWand (6무기).
    pub fn with_starter_loadout() -> Self {
        let mut inv = Self::default();
        inv.slots.push(WeaponSlot {
            kind: WeaponKind::Whip {
                damage:      10.0,
                area_width:  120.0,
                area_height: 60.0,
            },
            level:    1,
            cooldown: 1.0,
            elapsed:  0.0,
        });
        inv.slots.push(WeaponSlot {
            kind: WeaponKind::MagicWand {
                damage:           8.0,
                projectile_speed: 300.0,
                lifetime:         1.5,
                pierce:           0,
            },
            level:    1,
            cooldown: 1.2,
            elapsed:  0.0,
        });
        inv.slots.push(WeaponSlot {
            kind: WeaponKind::Knife {
                damage:           6.0,
                projectile_speed: 400.0,
                lifetime:         1.0,
                pierce:           0,
                amount:           1,
                spread_radians:   0.3,
            },
            level:    1,
            cooldown: 0.8,
            elapsed:  0.0,
        });
        inv.slots.push(WeaponSlot {
            kind: WeaponKind::Axe {
                damage:        12.0,
                initial_speed: 250.0,
                gravity:       600.0,
                lifetime:      1.5,
                pierce:        2,
                amount:        1,
            },
            level:    1,
            cooldown: 1.5,
            elapsed:  0.0,
        });
        inv.slots.push(WeaponSlot {
            kind: WeaponKind::Cross {
                damage:           14.0,
                projectile_speed: 280.0,
                lifetime:         3.0,
                pierce:           3,
                amount:           1,
                return_at:        0.7,
            },
            level:    1,
            cooldown: 1.8,
            elapsed:  0.0,
        });
        inv.slots.push(WeaponSlot {
            kind: WeaponKind::FireWand {
                damage:           25.0,
                projectile_speed: 250.0,
                lifetime:         1.5,
                pierce:           0,
            },
            level:    1,
            cooldown: 2.5,
            elapsed:  0.0,
        });
        inv
    }

    /// 하위 호환용 래퍼 — with_starter_loadout 으로 위임. 기존 호출자가 있으면 변경 권장.
    #[deprecated(since = "0.2.0", note = "with_starter_loadout 를 사용하세요")]
    pub fn with_whip_default() -> Self {
        Self::with_starter_loadout()
    }

    /// 첫 매칭되는 Whip 슬롯 (있을 경우) 의 mutable 참조.
    pub fn whip_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots.iter_mut().find(|s| matches!(s.kind, WeaponKind::Whip { .. }))
    }

    pub fn whip_slot(&self) -> Option<&WeaponSlot> {
        self.slots.iter().find(|s| matches!(s.kind, WeaponKind::Whip { .. }))
    }

    /// 첫 매칭되는 MagicWand 슬롯의 mutable 참조.
    pub fn magic_wand_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots.iter_mut().find(|s| matches!(s.kind, WeaponKind::MagicWand { .. }))
    }

    pub fn magic_wand_slot(&self) -> Option<&WeaponSlot> {
        self.slots.iter().find(|s| matches!(s.kind, WeaponKind::MagicWand { .. }))
    }

    /// 첫 매칭되는 Knife 슬롯의 mutable 참조.
    pub fn knife_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots.iter_mut().find(|s| matches!(s.kind, WeaponKind::Knife { .. }))
    }

    pub fn knife_slot(&self) -> Option<&WeaponSlot> {
        self.slots.iter().find(|s| matches!(s.kind, WeaponKind::Knife { .. }))
    }

    /// 첫 매칭되는 Axe 슬롯의 mutable 참조.
    pub fn axe_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots.iter_mut().find(|s| matches!(s.kind, WeaponKind::Axe { .. }))
    }

    pub fn axe_slot(&self) -> Option<&WeaponSlot> {
        self.slots.iter().find(|s| matches!(s.kind, WeaponKind::Axe { .. }))
    }

    /// 첫 매칭되는 Cross 슬롯의 mutable 참조.
    pub fn cross_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots.iter_mut().find(|s| matches!(s.kind, WeaponKind::Cross { .. }))
    }

    pub fn cross_slot(&self) -> Option<&WeaponSlot> {
        self.slots.iter().find(|s| matches!(s.kind, WeaponKind::Cross { .. }))
    }

    /// 첫 매칭되는 FireWand 슬롯의 mutable 참조.
    pub fn fire_wand_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots.iter_mut().find(|s| matches!(s.kind, WeaponKind::FireWand { .. }))
    }

    pub fn fire_wand_slot(&self) -> Option<&WeaponSlot> {
        self.slots.iter().find(|s| matches!(s.kind, WeaponKind::FireWand { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_starts_with_whip_slot_0() {
        let inv = WeaponInventory::with_starter_loadout();
        // Vec 기반 — 인덱스로 직접 접근
        assert_eq!(inv.slots.len(), 6, "6개 슬롯이 있어야 함");
        assert!(matches!(inv.slots[0].kind, WeaponKind::Whip { .. }), "슬롯 0 은 Whip 이어야 함");
        assert!(matches!(inv.slots[1].kind, WeaponKind::MagicWand { .. }), "슬롯 1 은 MagicWand 여야 함");
        assert!(matches!(inv.slots[2].kind, WeaponKind::Knife { .. }), "슬롯 2 는 Knife 여야 함");
        assert!(matches!(inv.slots[3].kind, WeaponKind::Axe { .. }), "슬롯 3 은 Axe 여야 함");
        assert!(matches!(inv.slots[4].kind, WeaponKind::Cross { .. }), "슬롯 4 는 Cross 여야 함");
        assert!(matches!(inv.slots[5].kind, WeaponKind::FireWand { .. }), "슬롯 5 는 FireWand 여야 함");

        let slot = &inv.slots[0];
        assert_eq!(slot.level, 1);
        assert_eq!(slot.cooldown, 1.0);
        assert!(matches!(slot.kind, WeaponKind::Whip { damage, .. } if damage == 10.0));
        // MagicWand 슬롯 확인
        let wand_slot = &inv.slots[1];
        assert_eq!(wand_slot.level, 1);
        assert_eq!(wand_slot.cooldown, 1.2);
        assert!(matches!(
            wand_slot.kind,
            WeaponKind::MagicWand { damage, pierce, .. } if damage == 8.0 && pierce == 0
        ));
        // Knife 슬롯 확인
        let knife_slot = &inv.slots[2];
        assert_eq!(knife_slot.cooldown, 0.8);
        assert!(matches!(
            knife_slot.kind,
            WeaponKind::Knife { damage, amount, .. } if damage == 6.0 && amount == 1
        ));
        // Axe 슬롯 확인
        let axe_slot = &inv.slots[3];
        assert_eq!(axe_slot.cooldown, 1.5);
        assert!(matches!(
            axe_slot.kind,
            WeaponKind::Axe { damage, pierce, .. } if damage == 12.0 && pierce == 2
        ));
    }

    #[test]
    fn weapon_slot_tick_returns_true_on_cooldown() {
        let mut slot = WeaponSlot {
            kind:     WeaponKind::Whip { damage: 10.0, area_width: 120.0, area_height: 60.0 },
            level:    1,
            cooldown: 1.0,
            elapsed:  0.0,
        };
        // cooldown 미달 — false
        assert!(!slot.tick(0.5), "0.5초 누적은 발화하면 안 됨");
        // cooldown 도달 — true, elapsed 가 초기화됨
        assert!(slot.tick(0.5), "1.0초 누적 시 발화해야 함");
        assert_eq!(slot.elapsed, 0.0, "발화 후 elapsed 가 0.0 이어야 함");
    }
}
