/// 무기 종류 + 종류별 파라미터.
///
/// Phase 2-A 는 Whip 만 inventory 에 마이그레이션.
/// Phase 2-B 에서 MagicWand 추가.
/// Phase 2-C 에서 Knife, Axe 추가.
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

/// Player 컴포넌트. 슬롯 6개 고정 (Vampire Survivors 원작 무기 슬롯 수).
///
/// Phase 2-A 는 첫 슬롯에 Whip 만 채워둔다.
#[derive(Debug, Clone, Default)]
pub struct WeaponInventory {
    pub slots: [Option<WeaponSlot>; 6],
}

impl WeaponInventory {
    /// Phase 2-C 스타터 로드아웃: Whip(slot[0]) + MagicWand(slot[1]) + Knife(slot[2]) + Axe(slot[3]).
    pub fn with_starter_loadout() -> Self {
        let mut inv = Self::default();
        inv.slots[0] = Some(WeaponSlot {
            kind: WeaponKind::Whip {
                damage:      10.0,
                area_width:  120.0,
                area_height: 60.0,
            },
            level:    1,
            cooldown: 1.0,
            elapsed:  0.0,
        });
        inv.slots[1] = Some(WeaponSlot {
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
        inv.slots[2] = Some(WeaponSlot {
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
        inv.slots[3] = Some(WeaponSlot {
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
        inv
    }

    /// 하위 호환용 래퍼 — with_starter_loadout 으로 위임. 기존 호출자가 있으면 변경 권장.
    #[deprecated(since = "0.2.0", note = "with_starter_loadout 를 사용하세요")]
    pub fn with_whip_default() -> Self {
        Self::with_starter_loadout()
    }

    /// 첫 매칭되는 Whip 슬롯 (있을 경우) 의 mutable 참조.
    pub fn whip_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        for slot in &mut self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::Whip { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn whip_slot(&self) -> Option<&WeaponSlot> {
        for slot in &self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::Whip { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }

    /// 첫 매칭되는 MagicWand 슬롯의 mutable 참조.
    pub fn magic_wand_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        for slot in &mut self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::MagicWand { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn magic_wand_slot(&self) -> Option<&WeaponSlot> {
        for slot in &self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::MagicWand { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }

    /// 첫 매칭되는 Knife 슬롯의 mutable 참조.
    pub fn knife_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        for slot in &mut self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::Knife { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn knife_slot(&self) -> Option<&WeaponSlot> {
        for slot in &self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::Knife { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }

    /// 첫 매칭되는 Axe 슬롯의 mutable 참조.
    pub fn axe_slot_mut(&mut self) -> Option<&mut WeaponSlot> {
        for slot in &mut self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::Axe { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn axe_slot(&self) -> Option<&WeaponSlot> {
        for slot in &self.slots {
            if let Some(s) = slot {
                if matches!(s.kind, WeaponKind::Axe { .. }) {
                    return Some(s);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_starts_with_whip_slot_0() {
        let inv = WeaponInventory::with_starter_loadout();
        assert!(inv.slots[0].is_some(), "슬롯 0 에 Whip 이 있어야 함");
        // slot[1] 은 MagicWand
        assert!(inv.slots[1].is_some(), "슬롯 1 에 MagicWand 가 있어야 함");
        // slot[2] 는 Knife, slot[3] 은 Axe
        assert!(inv.slots[2].is_some(), "슬롯 2 에 Knife 가 있어야 함");
        assert!(inv.slots[3].is_some(), "슬롯 3 에 Axe 가 있어야 함");
        for i in 4..6 {
            assert!(inv.slots[i].is_none(), "슬롯 {i} 는 비어 있어야 함");
        }
        let slot = inv.slots[0].as_ref().unwrap();
        assert_eq!(slot.level, 1);
        assert_eq!(slot.cooldown, 1.0);
        assert!(matches!(slot.kind, WeaponKind::Whip { damage, .. } if damage == 10.0));
        // MagicWand 슬롯 확인
        let wand_slot = inv.slots[1].as_ref().unwrap();
        assert_eq!(wand_slot.level, 1);
        assert_eq!(wand_slot.cooldown, 1.2);
        assert!(matches!(
            wand_slot.kind,
            WeaponKind::MagicWand { damage, pierce, .. } if damage == 8.0 && pierce == 0
        ));
        // Knife 슬롯 확인
        let knife_slot = inv.slots[2].as_ref().unwrap();
        assert_eq!(knife_slot.cooldown, 0.8);
        assert!(matches!(
            knife_slot.kind,
            WeaponKind::Knife { damage, amount, .. } if damage == 6.0 && amount == 1
        ));
        // Axe 슬롯 확인
        let axe_slot = inv.slots[3].as_ref().unwrap();
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
