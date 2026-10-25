pub const ZOMBIE_WIDTH: f32 = 22.0;
pub const BITE_REACH: f32 = 8.0;

pub const WALK_FRAMES: usize = 7;
pub const ATTACK_FRAMES: usize = 7;
pub const DIE_FRAMES: usize = 9;

const WALK_FRAME_DELAY: u8 = 6;
const ATTACK_FRAME_DELAY: u8 = 5;
const DIE_FRAME_DELAY: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZombieKind {
    Basic,
}

pub struct ZombieStats {
    pub hp: f32,
    pub speed: f32,
    pub bite: f32,
}

impl ZombieKind {
    pub const fn stats(self) -> ZombieStats {
        match self {
            Self::Basic => ZombieStats {
                hp: 200.0,
                speed: 0.25,
                bite: 1.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZombieState {
    Walking,
    Eating,
    Dying,
}

impl ZombieState {
    pub const fn frames(self) -> usize {
        match self {
            Self::Walking => WALK_FRAMES,
            Self::Eating => ATTACK_FRAMES,
            Self::Dying => DIE_FRAMES,
        }
    }

    const fn frame_delay(self) -> u8 {
        match self {
            Self::Walking => WALK_FRAME_DELAY,
            Self::Eating => ATTACK_FRAME_DELAY,
            Self::Dying => DIE_FRAME_DELAY,
        }
    }
}

pub struct Zombie {
    pub kind: ZombieKind,
    pub row: usize,
    pub x: f32,
    pub hp: f32,
    pub state: ZombieState,
    anim_frame: u8,
    anim_timer: u8,
    anim_done: bool,
}

impl Zombie {
    pub fn new(kind: ZombieKind, row: usize, x: f32) -> Self {
        Self {
            kind,
            row,
            x,
            hp: kind.stats().hp,
            state: ZombieState::Walking,
            anim_frame: 0,
            anim_timer: WALK_FRAME_DELAY,
            anim_done: false,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    pub fn is_finished_dying(&self) -> bool {
        self.state == ZombieState::Dying && self.anim_done
    }

    pub fn set_state(&mut self, state: ZombieState) {
        if self.state != state {
            self.state = state;
            self.anim_frame = 0;
            self.anim_timer = state.frame_delay();
            self.anim_done = false;
        }
    }

    pub fn start_dying(&mut self) {
        if self.state != ZombieState::Dying {
            self.state = ZombieState::Dying;
            self.anim_frame = 0;
            self.anim_timer = DIE_FRAME_DELAY;
            self.anim_done = false;
        }
    }

    pub fn animation_frame(&self) -> usize {
        (self.anim_frame as usize).min(self.state.frames() - 1)
    }

    pub fn update_animation(&mut self) {
        if self.state == ZombieState::Dying && self.anim_done {
            return;
        }

        if self.anim_timer > 0 {
            self.anim_timer -= 1;
            return;
        }

        let frames = self.state.frames();
        self.anim_frame += 1;
        if self.anim_frame as usize >= frames {
            if self.state == ZombieState::Dying {
                self.anim_frame = (frames - 1) as u8;
                self.anim_done = true;
            } else {
                self.anim_frame = 0;
            }
        }
        self.anim_timer = self.state.frame_delay();
    }
}
