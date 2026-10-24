const PEA_SPEED: f32 = 3.0;
const PEA_DAMAGE: i32 = 20;

pub struct Projectile {
    pub row: usize,
    pub x: f32,
    pub damage: i32,
    pub speed: f32,
}

impl Projectile {
    pub fn pea(row: usize, x: f32) -> Self {
        Self {
            row,
            x,
            damage: PEA_DAMAGE,
            speed: PEA_SPEED,
        }
    }
}
