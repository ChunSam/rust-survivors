/// 무기 종류 + 종류별 파라미터.
///
/// Phase 2-A 는 Whip 만 inventory 에 마이그레이션.
/// Phase 2-B 에서 MagicWand 추가.
/// Phase 2-C 에서 Knife, Axe 추가.
/// Phase 2-D 에서 Cross, FireWand 추가.
/// Phase 2-E 에서 Garlic, HolyWater 추가.
/// Phase 2-F 에서 KingBible, LightningRing 추가.
#[derive(Debug, Clone, PartialEq)]
pub enum WeaponKind {
    Whip {
        damage: f32,
        area_width: f32,
        area_height: f32,
    },
    MagicWand {
        damage: f32,
        projectile_speed: f32,
        lifetime: f32,
        pierce: u8,
    },
    /// 가장 가까운 적 방향으로 직선 투사체를 `amount` 개 동시 발사.
    /// `spread_radians`: `amount > 1` 일 때 부채꼴 각도 범위 (라디안).
    Knife {
        damage: f32,
        projectile_speed: f32,
        lifetime: f32,
        pierce: u8,
        amount: u8,          // 동시 발사 개수
        spread_radians: f32, // amount > 1 일 때 각도 분산 (라디안)
    },
    /// 위로 던져 중력으로 떨어지는 포물선 투사체. `ProjectileBehavior::Arc` 사용.
    Axe {
        damage: f32,
        initial_speed: f32, // 위 방향 초기 속도 (px/s). velocity.y = -initial_speed.
        gravity: f32,       // 중력 가속도 (px/s^2). 양수 → 매 프레임 velocity.y 증가.
        lifetime: f32,
        pierce: u8,
        amount: u8,
    },
    /// 가장 가까운 적 방향으로 발사 후 `return_at` 초 뒤 방향 반전 (부메랑).
    /// `ProjectileBehavior::Boomerang` 사용.
    Cross {
        damage: f32,
        projectile_speed: f32,
        lifetime: f32,
        pierce: u8,
        amount: u8,
        return_at: f32, // 발사 후 이 시간(초)에 방향 반전
    },
    /// 랜덤 적을 향해 고데미지 단발 투사체를 발사. `ProjectileBehavior::Straight` 사용.
    FireWand {
        damage: f32,
        projectile_speed: f32,
        lifetime: f32,
        pierce: u8,
    },
    /// 플레이어 중심 오라. cooldown 마다 반경 안 모든 적에게 tick 데미지.
    Garlic {
        damage: f32, // tick 당 데미지 (cooldown 마다 1회)
        radius: f32, // 플레이어 중심 오라 반경
    },
    /// 플레이어 주변 랜덤 위치에 지속 풀을 드롭. tick_cooldown 마다 area-damage.
    HolyWater {
        damage: f32,        // 풀 tick 당 데미지
        radius: f32,        // 풀의 area-damage 반경
        pool_lifetime: f32, // 풀이 유지되는 시간(초)
        tick_cooldown: f32, // 풀이 데미지 가하는 간격
        drop_count: u8,     // 한 번 발화 시 풀 N 개 드롭 (시작 1)
    },
    /// 플레이어 주위를 회전하는 책. cooldown 마다 book_count 권 스폰, lifetime 만료 시 despawn.
    KingBible {
        damage: f32,
        book_count: u8,
        radius: f32,        // 회전 반경 (px)
        angular_speed: f32, // 라디안/초
        lifetime: f32,      // 책 활성 시간 (초)
        tick_cooldown: f32, // 책이 데미지 가하는 간격
        hit_radius: f32,    // 충돌 판정 반경
    },
    /// cooldown 마다 strike_count 마리의 랜덤 zombie 위치에 즉시 area damage.
    LightningRing {
        damage: f32,
        strike_count: u8,
        hit_radius: f32, // 각 번개 타격 area 반경
    },
}

/// 무기 인벤토리의 한 슬롯. cooldown 기반 발화 트래킹은 슬롯이 소유한다.
/// (직전까지는 WhipSystem 이 자체 elapsed 를 보유 — 이제 slot 의 elapsed 로 이동.)
#[derive(Debug, Clone)]
pub struct WeaponSlot {
    pub kind: WeaponKind,
    pub level: u8,
    pub cooldown: f32, // 발화 간격 (초)
    pub elapsed: f32,  // 누적 시간 — cooldown 도달 시 발화 + elapsed -= cooldown
    pub evolved: bool, // Phase 6: 진화 완료 여부 (기본 false)
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

    /// stats.cooldown 곱을 적용한 effective cooldown 으로 tick.
    ///
    /// `cd_multiplier < 1.0` 이면 발화 간격이 짧아짐 (빨라짐).
    /// slot 의 `cooldown` 필드는 변경하지 않고, *비교 시점에만* 곱한다.
    pub fn tick_with_cooldown_multiplier(&mut self, dt: f32, cd_multiplier: f32) -> bool {
        self.elapsed += dt;
        let effective_cd = (self.cooldown * cd_multiplier).max(0.1); // 0 방지
        if self.elapsed >= effective_cd {
            self.elapsed -= effective_cd;
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
    /// Phase 2-G: assets/data/weapons.ron 을 컴파일 타임에 include_str! 로 포함하여 로드.
    /// 10 무기 하드코딩 제거 — data::load_starter_weapons() 로 위임.
    pub fn with_starter_loadout() -> Self {
        Self {
            slots: super::data::load_starter_weapons(),
        }
    }

    /// 하위 호환용 래퍼 — with_starter_loadout 으로 위임. 기존 호출자가 있으면 변경 권장.
    #[deprecated(since = "0.2.0", note = "with_starter_loadout 를 사용하세요")]
    pub fn with_whip_default() -> Self {
        Self::with_starter_loadout()
    }

    /// 첫 매칭되는 Whip 슬롯 (있을 경우) 의 mutable 참조.
    pub fn whip_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::Whip { .. }))
    }

    pub fn whip_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::Whip { .. }))
    }

    /// 첫 매칭되는 MagicWand 슬롯의 mutable 참조.
    pub fn magic_wand_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::MagicWand { .. }))
    }

    pub fn magic_wand_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::MagicWand { .. }))
    }

    /// 첫 매칭되는 Knife 슬롯의 mutable 참조.
    pub fn knife_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::Knife { .. }))
    }

    pub fn knife_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::Knife { .. }))
    }

    /// 첫 매칭되는 Axe 슬롯의 mutable 참조.
    pub fn axe_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::Axe { .. }))
    }

    pub fn axe_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::Axe { .. }))
    }

    /// 첫 매칭되는 Cross 슬롯의 mutable 참조.
    pub fn cross_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::Cross { .. }))
    }

    pub fn cross_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::Cross { .. }))
    }

    /// 첫 매칭되는 FireWand 슬롯의 mutable 참조.
    pub fn fire_wand_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::FireWand { .. }))
    }

    pub fn fire_wand_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::FireWand { .. }))
    }

    /// 첫 매칭되는 Garlic 슬롯의 mutable 참조.
    pub fn garlic_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::Garlic { .. }))
    }

    pub fn garlic_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::Garlic { .. }))
    }

    /// 첫 매칭되는 HolyWater 슬롯의 mutable 참조.
    pub fn holy_water_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::HolyWater { .. }))
    }

    pub fn holy_water_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::HolyWater { .. }))
    }

    /// 첫 매칭되는 KingBible 슬롯의 mutable 참조.
    pub fn king_bible_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::KingBible { .. }))
    }

    pub fn king_bible_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::KingBible { .. }))
    }

    /// 첫 매칭되는 LightningRing 슬롯의 mutable 참조.
    pub fn lightning_ring_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        self.slots
            .iter_mut()
            .find(|s| matches!(s.kind, WeaponKind::LightningRing { .. }))
    }

    pub fn lightning_ring_slot(&self) -> Option<&WeaponSlot> {
        self.slots
            .iter()
            .find(|s| matches!(s.kind, WeaponKind::LightningRing { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_starts_with_whip_slot_0() {
        let inv = WeaponInventory::with_starter_loadout();
        // Vec 기반 — 인덱스로 직접 접근
        assert_eq!(inv.slots.len(), 10, "10개 슬롯이 있어야 함");
        assert!(
            matches!(inv.slots[0].kind, WeaponKind::Whip { .. }),
            "슬롯 0 은 Whip 이어야 함"
        );
        assert!(
            matches!(inv.slots[1].kind, WeaponKind::MagicWand { .. }),
            "슬롯 1 은 MagicWand 여야 함"
        );
        assert!(
            matches!(inv.slots[2].kind, WeaponKind::Knife { .. }),
            "슬롯 2 는 Knife 여야 함"
        );
        assert!(
            matches!(inv.slots[3].kind, WeaponKind::Axe { .. }),
            "슬롯 3 은 Axe 여야 함"
        );
        assert!(
            matches!(inv.slots[4].kind, WeaponKind::Cross { .. }),
            "슬롯 4 는 Cross 여야 함"
        );
        assert!(
            matches!(inv.slots[5].kind, WeaponKind::FireWand { .. }),
            "슬롯 5 는 FireWand 여야 함"
        );
        assert!(
            matches!(inv.slots[6].kind, WeaponKind::Garlic { .. }),
            "슬롯 6 은 Garlic 이어야 함"
        );
        assert!(
            matches!(inv.slots[7].kind, WeaponKind::HolyWater { .. }),
            "슬롯 7 은 HolyWater 여야 함"
        );
        assert!(
            matches!(inv.slots[8].kind, WeaponKind::KingBible { .. }),
            "슬롯 8 은 KingBible 이어야 함"
        );
        assert!(
            matches!(inv.slots[9].kind, WeaponKind::LightningRing { .. }),
            "슬롯 9 는 LightningRing 이어야 함"
        );

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
            kind: WeaponKind::Whip {
                damage: 10.0,
                area_width: 120.0,
                area_height: 60.0,
            },
            level: 1,
            cooldown: 1.0,
            elapsed: 0.0,
            evolved: false,
        };
        // cooldown 미달 — false
        assert!(!slot.tick(0.5), "0.5초 누적은 발화하면 안 됨");
        // cooldown 도달 — true, elapsed 가 초기화됨
        assert!(slot.tick(0.5), "1.0초 누적 시 발화해야 함");
        assert_eq!(slot.elapsed, 0.0, "발화 후 elapsed 가 0.0 이어야 함");
    }
}
