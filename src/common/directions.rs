use bevy::math::IVec2;

pub const VEC_UP: IVec2 = IVec2 { x: 0, y: 1 };
pub const VEC_UP_LEFT: IVec2 = IVec2 { x: -1, y: 1 };
pub const VEC_UP_RIGHT: IVec2 = IVec2 { x: 1, y: 1 };
pub const VEC_DOWN: IVec2 = IVec2 { x: 0, y: -1 };
pub const VEC_DOWN_LEFT: IVec2 = IVec2 { x: -1, y: -1 };
pub const VEC_DOWN_RIGHT: IVec2 = IVec2 { x: 1, y: -1 };
pub const VEC_RIGHT: IVec2 = IVec2 { x: 1, y: 0 };
pub const VEC_LEFT: IVec2 = IVec2 { x: -1, y: 0 };

pub const DIRECTIONS: [IVec2; 9] = [
    VEC_DOWN_LEFT,
    VEC_DOWN,
    VEC_DOWN_RIGHT,
    VEC_LEFT,
    IVec2::ZERO,
    VEC_RIGHT,
    VEC_UP_LEFT,
    VEC_UP,
    VEC_UP_RIGHT,
];

pub const fn dir_to_index(dir: IVec2) -> usize {
    ((dir.x + 1) + (dir.y + 1) * 3) as usize
}
