/// 무기 종류 + 종류별 파라미터.
///
/// Phase 2-A 는 Whip 만 inventory 에 마이그레이션. 다음 sub-phase 에서
/// MagicWand 등 추가될 때 variant 가 늘어난다.
#[derive(Debug, Clone, PartialEq)]
pub enum WeaponKind {
    Whip {
        damage:      f32,
        area_width:  f32,
        area_height: f32,
    },
    // 다음 단계 (2-B) 에서 MagicWand { damage, projectile_speed, lifetime, ... } 추가
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
    pub fn with_whip_default() -> Self {
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
        inv
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_starts_with_whip_slot_0() {
        let inv = WeaponInventory::with_whip_default();
        assert!(inv.slots[0].is_some(), "슬롯 0 에 Whip 이 있어야 함");
        for i in 1..6 {
            assert!(inv.slots[i].is_none(), "슬롯 {i} 는 비어 있어야 함");
        }
        let slot = inv.slots[0].as_ref().unwrap();
        assert_eq!(slot.level, 1);
        assert_eq!(slot.cooldown, 1.0);
        assert!(matches!(slot.kind, WeaponKind::Whip { damage, .. } if damage == 10.0));
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
