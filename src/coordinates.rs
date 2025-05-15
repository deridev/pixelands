use bevy::math::*;

use crate::constants::CHUNK_SIZE;

pub fn world_to_chunk_position(world_position: Vec2) -> IVec2 {
    IVec2::new(
        world_position.x.div_euclid(CHUNK_SIZE as f32) as i32,
        -(world_position.y.div_euclid(CHUNK_SIZE as f32) as i32),
    )
}

pub fn chunk_to_world_position(chunk_position: IVec2) -> Vec2 {
    Vec2::new(
        chunk_position.x as f32 * CHUNK_SIZE as f32,
        chunk_position.y as f32 * CHUNK_SIZE as f32,
    )
}

pub fn world_to_element_position(world_position: Vec2) -> IVec2 {
    IVec2::new(
        world_position.x.rem_euclid(CHUNK_SIZE as f32) as i32,
        world_position.y.rem_euclid(CHUNK_SIZE as f32) as i32,
    )
}
