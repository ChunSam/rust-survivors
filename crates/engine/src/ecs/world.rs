use std::any::{Any, TypeId};
use std::collections::HashMap;

/// 게임 오브젝트를 식별하는 고유 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

/// ECS의 중심 저장소
///
/// - 엔티티(Entity): 단순한 u32 ID
/// - 컴포넌트: TypeId 기준으로 구분되는 Vec<Option<Box<dyn Any>>>
/// - 리소스: 전역 싱글턴 데이터 (입력 상태, 물리 세계 등)
pub struct World {
    next_id:    u32,
    entities:   Vec<Entity>,
    // TypeId -> Vec indexed by entity id (없으면 None)
    components: HashMap<TypeId, Vec<Option<Box<dyn Any>>>>,
    resources:  HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id:    0,
            entities:   Vec::new(),
            components: HashMap::new(),
            resources:  HashMap::new(),
        }
    }

    /// 빈 엔티티를 생성하고 반환한다.
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        self.next_id += 1;
        self.entities.push(entity);
        // 기존 컴포넌트 저장소를 새 엔티티 크기만큼 확장
        for vec in self.components.values_mut() {
            while vec.len() <= entity.0 as usize {
                vec.push(None);
            }
        }
        entity
    }

    /// 엔티티에 컴포넌트를 붙인다.
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let vec = self
            .components
            .entry(TypeId::of::<T>())
            .or_insert_with(Vec::new);
        while vec.len() <= entity.0 as usize {
            vec.push(None);
        }
        vec[entity.0 as usize] = Some(Box::new(component));
    }

    /// 엔티티의 컴포넌트를 불변 참조로 가져온다.
    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())?
            .get(entity.0 as usize)?
            .as_ref()?
            .downcast_ref::<T>()
    }

    /// 엔티티의 컴포넌트를 가변 참조로 가져온다.
    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())?
            .get_mut(entity.0 as usize)?
            .as_mut()?
            .downcast_mut::<T>()
    }

    /// 특정 컴포넌트 T를 가진 모든 (Entity, &T) 쌍을 순회한다.
    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id   = TypeId::of::<T>();
        let entities  = &self.entities;
        let components = self.components.get(&type_id);
        entities.iter().filter_map(move |&entity| {
            let comp = components?
                .get(entity.0 as usize)?
                .as_ref()?
                .downcast_ref::<T>()?;
            Some((entity, comp))
        })
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // ── 리소스 (전역 싱글턴) ────────────────────────────────────────────────

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn resource<T: 'static>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    pub fn resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())?
            .downcast_mut::<T>()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    struct Position { x: f32, y: f32 }
    #[allow(dead_code)]
    struct Health(u32);

    #[test]
    fn spawn_and_query() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position { x: 1.0, y: 2.0 });
        world.add_component(e, Health(100));

        let pos = world.get::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);

        let count = world.query::<Position>().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn resource() {
        let mut world = World::new();
        world.insert_resource(42u32);
        assert_eq!(*world.resource::<u32>().unwrap(), 42);
    }
}
