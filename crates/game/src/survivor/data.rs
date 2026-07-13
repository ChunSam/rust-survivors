use super::inventory::{WeaponKind, WeaponSlot};
/// weapons.ron 데이터 로딩 모듈.
///
/// 컴파일 타임에 RON 파일을 바이너리에 포함(`include_str!`)하여
/// 런타임 파일 의존 없이 무기 스탯을 로드한다.
use serde::Deserialize;

/// RON 파일의 개별 무기 정의. 모든 무기별 필드를 Option으로 평탄화.
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponDef {
    pub kind: String,
    pub cooldown: f32,
    // 공통 / 투사체
    pub damage: Option<f32>,
    pub area_width: Option<f32>,
    pub area_height: Option<f32>,
    pub projectile_speed: Option<f32>,
    pub lifetime: Option<f32>,
    pub pierce: Option<u8>,
    pub amount: Option<u8>,
    pub spread_radians: Option<f32>,
    // Axe 전용
    pub initial_speed: Option<f32>,
    pub gravity: Option<f32>,
    // Cross 전용
    pub return_at: Option<f32>,
    // 오라·풀 공통
    pub radius: Option<f32>,
    // HolyWater 전용
    pub pool_lifetime: Option<f32>,
    pub tick_cooldown: Option<f32>,
    pub drop_count: Option<u8>,
    // KingBible 전용
    pub book_count: Option<u8>,
    pub angular_speed: Option<f32>,
    pub hit_radius: Option<f32>,
    // LightningRing 전용
    pub strike_count: Option<u8>,
}

/// RON 파일 최상위 구조체.
#[derive(Debug, Clone, Deserialize)]
pub struct WeaponsDataFile {
    pub weapons: Vec<WeaponDef>,
}

/// 컴파일 타임에 내장된 RON 텍스트.
const WEAPONS_RON: &str = include_str!("../../../../assets/data/weapons.ron");

/// `assets/data/weapons.ron`을 파싱하여 WeaponSlot 벡터를 반환한다.
///
/// 파싱 실패 시 panic (개발 시 즉시 오류 감지).
pub fn load_starter_weapons() -> Vec<WeaponSlot> {
    let parsed: WeaponsDataFile =
        ron::from_str(WEAPONS_RON).expect("assets/data/weapons.ron 파싱 실패");
    parsed.weapons.iter().map(weapon_def_to_slot).collect()
}

fn weapon_def_to_slot(def: &WeaponDef) -> WeaponSlot {
    let kind = match def.kind.as_str() {
        "Whip" => WeaponKind::Whip {
            damage: def.damage.expect("Whip: damage 필드 필요"),
            area_width: def.area_width.expect("Whip: area_width 필드 필요"),
            area_height: def.area_height.expect("Whip: area_height 필드 필요"),
        },
        "MagicWand" => WeaponKind::MagicWand {
            damage: def.damage.expect("MagicWand: damage 필드 필요"),
            projectile_speed: def
                .projectile_speed
                .expect("MagicWand: projectile_speed 필드 필요"),
            lifetime: def.lifetime.expect("MagicWand: lifetime 필드 필요"),
            pierce: def.pierce.expect("MagicWand: pierce 필드 필요"),
        },
        "Knife" => WeaponKind::Knife {
            damage: def.damage.expect("Knife: damage 필드 필요"),
            projectile_speed: def
                .projectile_speed
                .expect("Knife: projectile_speed 필드 필요"),
            lifetime: def.lifetime.expect("Knife: lifetime 필드 필요"),
            pierce: def.pierce.expect("Knife: pierce 필드 필요"),
            amount: def.amount.expect("Knife: amount 필드 필요"),
            spread_radians: def.spread_radians.expect("Knife: spread_radians 필드 필요"),
        },
        "Axe" => WeaponKind::Axe {
            damage: def.damage.expect("Axe: damage 필드 필요"),
            initial_speed: def.initial_speed.expect("Axe: initial_speed 필드 필요"),
            gravity: def.gravity.expect("Axe: gravity 필드 필요"),
            lifetime: def.lifetime.expect("Axe: lifetime 필드 필요"),
            pierce: def.pierce.expect("Axe: pierce 필드 필요"),
            amount: def.amount.expect("Axe: amount 필드 필요"),
        },
        "Cross" => WeaponKind::Cross {
            damage: def.damage.expect("Cross: damage 필드 필요"),
            projectile_speed: def
                .projectile_speed
                .expect("Cross: projectile_speed 필드 필요"),
            lifetime: def.lifetime.expect("Cross: lifetime 필드 필요"),
            pierce: def.pierce.expect("Cross: pierce 필드 필요"),
            amount: def.amount.expect("Cross: amount 필드 필요"),
            return_at: def.return_at.expect("Cross: return_at 필드 필요"),
        },
        "FireWand" => WeaponKind::FireWand {
            damage: def.damage.expect("FireWand: damage 필드 필요"),
            projectile_speed: def
                .projectile_speed
                .expect("FireWand: projectile_speed 필드 필요"),
            lifetime: def.lifetime.expect("FireWand: lifetime 필드 필요"),
            pierce: def.pierce.expect("FireWand: pierce 필드 필요"),
        },
        "Garlic" => WeaponKind::Garlic {
            damage: def.damage.expect("Garlic: damage 필드 필요"),
            radius: def.radius.expect("Garlic: radius 필드 필요"),
        },
        "HolyWater" => WeaponKind::HolyWater {
            damage: def.damage.expect("HolyWater: damage 필드 필요"),
            radius: def.radius.expect("HolyWater: radius 필드 필요"),
            pool_lifetime: def
                .pool_lifetime
                .expect("HolyWater: pool_lifetime 필드 필요"),
            tick_cooldown: def
                .tick_cooldown
                .expect("HolyWater: tick_cooldown 필드 필요"),
            drop_count: def.drop_count.expect("HolyWater: drop_count 필드 필요"),
        },
        "KingBible" => WeaponKind::KingBible {
            damage: def.damage.expect("KingBible: damage 필드 필요"),
            book_count: def.book_count.expect("KingBible: book_count 필드 필요"),
            radius: def.radius.expect("KingBible: radius 필드 필요"),
            angular_speed: def
                .angular_speed
                .expect("KingBible: angular_speed 필드 필요"),
            lifetime: def.lifetime.expect("KingBible: lifetime 필드 필요"),
            tick_cooldown: def
                .tick_cooldown
                .expect("KingBible: tick_cooldown 필드 필요"),
            hit_radius: def.hit_radius.expect("KingBible: hit_radius 필드 필요"),
        },
        "LightningRing" => WeaponKind::LightningRing {
            damage: def.damage.expect("LightningRing: damage 필드 필요"),
            strike_count: def
                .strike_count
                .expect("LightningRing: strike_count 필드 필요"),
            hit_radius: def.hit_radius.expect("LightningRing: hit_radius 필드 필요"),
        },
        other => panic!("알 수 없는 무기 종류: {other}"),
    };

    WeaponSlot {
        kind,
        level: 1,
        cooldown: def.cooldown,
        elapsed: 0.0,
        evolved: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivor::inventory::WeaponKind;

    #[test]
    fn weapons_ron_loads_ten_weapons() {
        let slots = load_starter_weapons();
        assert_eq!(slots.len(), 10, "weapons.ron 은 10개 무기를 정의해야 함");

        // 순서대로 각 variant 검증
        assert!(
            matches!(slots[0].kind, WeaponKind::Whip { .. }),
            "슬롯 0: Whip"
        );
        assert!(
            matches!(slots[1].kind, WeaponKind::MagicWand { .. }),
            "슬롯 1: MagicWand"
        );
        assert!(
            matches!(slots[2].kind, WeaponKind::Knife { .. }),
            "슬롯 2: Knife"
        );
        assert!(
            matches!(slots[3].kind, WeaponKind::Axe { .. }),
            "슬롯 3: Axe"
        );
        assert!(
            matches!(slots[4].kind, WeaponKind::Cross { .. }),
            "슬롯 4: Cross"
        );
        assert!(
            matches!(slots[5].kind, WeaponKind::FireWand { .. }),
            "슬롯 5: FireWand"
        );
        assert!(
            matches!(slots[6].kind, WeaponKind::Garlic { .. }),
            "슬롯 6: Garlic"
        );
        assert!(
            matches!(slots[7].kind, WeaponKind::HolyWater { .. }),
            "슬롯 7: HolyWater"
        );
        assert!(
            matches!(slots[8].kind, WeaponKind::KingBible { .. }),
            "슬롯 8: KingBible"
        );
        assert!(
            matches!(slots[9].kind, WeaponKind::LightningRing { .. }),
            "슬롯 9: LightningRing"
        );

        // 대표 스탯 검증
        assert!(
            matches!(slots[0].kind, WeaponKind::Whip { damage, .. } if (damage - 10.0).abs() < 0.001)
        );
        assert_eq!(slots[0].cooldown, 1.0);
        assert!(
            matches!(slots[1].kind, WeaponKind::MagicWand { damage, pierce, .. } if (damage - 8.0).abs() < 0.001 && pierce == 0)
        );
        assert!(
            matches!(slots[9].kind, WeaponKind::LightningRing { strike_count, .. } if strike_count == 1)
        );
    }

    #[test]
    fn starter_weapon_stats_are_playable_positive_values() {
        for slot in load_starter_weapons() {
            assert!(slot.cooldown > 0.0, "{:?} has invalid cooldown", slot.kind);
            assert_eq!(slot.level, 1);
            assert_eq!(slot.elapsed, 0.0);
            assert!(!slot.evolved);

            match slot.kind {
                WeaponKind::Whip {
                    damage,
                    area_width,
                    area_height,
                } => {
                    assert!(damage > 0.0);
                    assert!(area_width >= 80.0);
                    assert!(area_height >= 40.0);
                }
                WeaponKind::MagicWand {
                    damage,
                    projectile_speed,
                    lifetime,
                    ..
                }
                | WeaponKind::FireWand {
                    damage,
                    projectile_speed,
                    lifetime,
                    ..
                } => {
                    assert!(damage > 0.0);
                    assert!(projectile_speed > 0.0);
                    assert!(lifetime > 0.0);
                }
                WeaponKind::Knife {
                    damage,
                    projectile_speed,
                    lifetime,
                    amount,
                    spread_radians,
                    ..
                } => {
                    assert!(damage > 0.0);
                    assert!(projectile_speed > 0.0);
                    assert!(lifetime > 0.0);
                    assert!(amount >= 1);
                    assert!(spread_radians >= 0.0);
                }
                WeaponKind::Axe {
                    damage,
                    initial_speed,
                    gravity,
                    lifetime,
                    amount,
                    ..
                } => {
                    assert!(damage > 0.0);
                    assert!(initial_speed > 0.0);
                    assert!(gravity > 0.0);
                    assert!(lifetime > 0.0);
                    assert!(amount >= 1);
                }
                WeaponKind::Cross {
                    damage,
                    projectile_speed,
                    lifetime,
                    return_at,
                    amount,
                    ..
                } => {
                    assert!(damage > 0.0);
                    assert!(projectile_speed > 0.0);
                    assert!(lifetime > return_at);
                    assert!(return_at > 0.0);
                    assert!(amount >= 1);
                }
                WeaponKind::Garlic { damage, radius } => {
                    assert!(damage > 0.0);
                    assert!(radius > 0.0);
                }
                WeaponKind::HolyWater {
                    damage,
                    radius,
                    pool_lifetime,
                    tick_cooldown,
                    drop_count,
                } => {
                    assert!(damage > 0.0);
                    assert!(radius > 0.0);
                    assert!(pool_lifetime > tick_cooldown);
                    assert!(tick_cooldown > 0.0);
                    assert!(drop_count >= 1);
                }
                WeaponKind::KingBible {
                    damage,
                    book_count,
                    radius,
                    angular_speed,
                    lifetime,
                    tick_cooldown,
                    hit_radius,
                } => {
                    assert!(damage > 0.0);
                    assert!(book_count >= 1);
                    assert!(radius > hit_radius);
                    assert!(angular_speed > 0.0);
                    assert!(lifetime > tick_cooldown);
                    assert!(tick_cooldown > 0.0);
                    assert!(hit_radius > 0.0);
                }
                WeaponKind::LightningRing {
                    damage,
                    strike_count,
                    hit_radius,
                } => {
                    assert!(damage > 0.0);
                    assert!(strike_count >= 1);
                    assert!(hit_radius > 0.0);
                }
            }
        }
    }
}
