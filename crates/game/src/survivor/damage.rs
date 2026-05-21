use engine::{Sprite, Transform, World};
use engine::Entity;

use super::health::Health;
use super::weapon::HitFlash;
use super::xp::spawn_xp_gem;

/// zombie 엔티티에 데미지 적용 + 사망 처리 + XpGem 드롭 + HitFlash 부착.
///
/// 호출처(Whip/Projectile/Garlic/Pool/KingBible/Lightning)에서 동일 패턴을 복사하던
/// 코드를 추출한 단일 진입점. 반환값은 *사망 여부*. 호출자는 반환값으로 killed 카운트 누적.
pub fn apply_damage_to_zombie(world: &mut World, zombie: Entity, damage: f32) -> bool {
    let original = if let Some(s) = world.get_mut::<Sprite>(zombie) {
        let o = s.color;
        s.color = [1.0, 0.3, 0.3, 1.0];
        o
    } else {
        return false;
    };

    let died = if let Some(h) = world.get_mut::<Health>(zombie) {
        h.take_damage(damage)
    } else {
        false
    };

    if died {
        let drop_pos = world.get::<Transform>(zombie).map(|t| t.position);
        world.despawn(zombie);
        if let Some(p) = drop_pos {
            spawn_xp_gem(world, p, 1);
        }
    } else {
        world.add_component(zombie, HitFlash { remaining: 0.1, original });
    }
    died
}
