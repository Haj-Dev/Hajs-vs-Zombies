const PEA_SPEED: f32 = 3.0;
const PEA_DAMAGE: f32 = 20.0;

pub const BULLET_SPIN_SPEED: f32 = 18.0;
pub const EXPLOSION_FRAMES: usize = 3;
const EXPLOSION_FRAME_DELAY: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileState {
    Flying,
    Exploding { frame: u8, timer: u8 },
}

pub struct Projectile {
    pub row: usize,
    pub x: f32,
    pub damage: f32,
    pub speed: f32,
    pub state: ProjectileState,
    pub spin: f32,
}

impl Projectile {
    pub fn pea(row: usize, x: f32) -> Self {
        Self {
            row,
            x,
            damage: PEA_DAMAGE,
            speed: PEA_SPEED,
            state: ProjectileState::Flying,
            spin: 0.0,
        }
    }

    pub fn is_flying(&self) -> bool {
        matches!(self.state, ProjectileState::Flying)
    }

    pub fn explode(&mut self) {
        self.state = ProjectileState::Exploding {
            frame: 0,
            timer: EXPLOSION_FRAME_DELAY,
        };
    }

    pub fn tick_explosion(&mut self) -> bool {
        let ProjectileState::Exploding { frame, timer } = &mut self.state else {
            return true;
        };

        *timer = timer.saturating_sub(1);
        if *timer == 0 {
            *frame += 1;
            if *frame as usize >= EXPLOSION_FRAMES {
                return false;
            }
            *timer = EXPLOSION_FRAME_DELAY;
        }
        true
    }
}
