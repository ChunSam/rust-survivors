#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// 데미지를 차감. 반환값: 차감 후 사망(<= 0) 여부.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        self.current -= amount;
        self.current <= 0.0
    }
}
