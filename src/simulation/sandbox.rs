use std::collections::HashMap;

use bevy::{ecs::resource::Resource, math::*};

use crate::{common::directions::DIRECTIONS, simulation::*};

#[derive(Debug, Clone, Resource)]
pub struct Sandbox {
    pub wframe: u8,
    pub chunks: HashMap<IVec2, SharedChunk>,
    pub fresh_chunks: Vec<IVec2>,
    pub active: bool,
}

impl Sandbox {
    pub fn new() -> Self {
        let mut sandbox = Self {
            wframe: 0,
            chunks: HashMap::new(),
            fresh_chunks: Vec::with_capacity(16),
            active: true,
        };

        for x in -1..=1 {
            for y in 0..=2 {
                sandbox.add_chunk(Chunk::new((x, y).into()));
            }
        }

        sandbox
    }

    pub fn get_shared_chunk(&self, position: IVec2) -> Option<SharedChunk> {
        self.chunks.get(&position).cloned()
    }

    pub fn get_chunk(&self, position: IVec2) -> Option<ReadableChunk> {
        self.chunks
            .get(&position)
            .map(|shared_chunk| shared_chunk.read())
    }

    #[allow(unused)]
    pub fn get_chunk_mut(&self, position: IVec2) -> Option<WritableChunk> {
        self.chunks
            .get(&position)
            .map(|shared_chunk| shared_chunk.write())
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> SharedChunk {
        let chunk_position = chunk.position;

        let shared_chunk = SharedChunk::new(chunk);
        self.chunks.insert(chunk_position, shared_chunk.clone());
        self.fresh_chunks.push(chunk_position);
        self.mark_chunks_surrounding_as_dirty(chunk_position);
        shared_chunk
    }

    pub fn mark_chunks_surrounding_as_dirty(&mut self, position: IVec2) {
        for dir in DIRECTIONS.iter() {
            if dir == &IVec2::ZERO {
                continue;
            }

            let chunk_position = position + *dir;
            if let Some(chunk) = self.chunks.get(&chunk_position) {
                chunk.write().mark_dirty_everything();
            }
        }
    }

    pub fn tick(&mut self) {
        if !self.active {
            return;
        }
        self.wframe = self.wframe.wrapping_add(1);
        let wframe = self.wframe;

        let chunk_positions = self.chunks.keys().copied().collect::<Vec<_>>();

        // Tick every chunk
        let mut new_chunks = Vec::with_capacity(16);
        for pos in chunk_positions.iter() {
            let chunk = self.chunks.get(pos).unwrap().read();
            let dirty = chunk.dirty_rect();
            if !chunk.active() {
                continue;
            }

            // Important because Rust don't automatically drop this until the next line
            drop(chunk);

            let unsafe_chunk_list = DIRECTIONS
                .map(|dir| match self.chunks.get(&(*pos + dir)) {
                    Some(cell) => Some(cell.clone()),
                    None => None,
                })
                .into_iter()
                .collect::<Vec<Option<SharedChunk>>>();
            let mut local_api = LocalApi::new(*pos, wframe, Default::default(), unsafe_chunk_list);

            if wframe % 2 == 0 {
                for x in dirty.min.x..dirty.max.x {
                    for y in dirty.min.y..dirty.max.y {
                        tick_element((x, y).into(), &mut local_api);
                    }
                }
            } else {
                for x in (dirty.min.x..dirty.max.x).rev() {
                    for y in dirty.min.y..dirty.max.y {
                        tick_element((x, y).into(), &mut local_api);
                    }
                }
            }

            let mut chunk = self.chunks.get(pos).unwrap().write();
            chunk.current_dirty_rect = chunk.next_dirty_rect;
            chunk.next_dirty_rect.clear();
            drop(chunk);

            for chunk_index in local_api.new_chunks.drain(..) {
                let chunk = local_api.chunks[chunk_index].clone().unwrap();
                new_chunks.push(chunk);
            }

            drop(local_api);
        }

        // Add new chunks
        for chunk in new_chunks.into_iter() {
            let pos = chunk.read().position;
            self.chunks.insert(pos, chunk);
            self.fresh_chunks.push(pos);
            self.mark_chunks_surrounding_as_dirty(pos);
        }
    }
}

fn tick_element(position: IVec2, api: &mut LocalApi) {
    if Chunk::is_outside(position) {
        return;
    }

    let mut element = api.get_element(position);
    if element.kind == ElementKind::Air || api.wframe == element.wframe {
        return;
    }

    element.wframe = api.wframe;
    api.element.0 = element;
    api.element.1 = position;
    api.set_wframe(api.wframe);

    match api.element.0.kind {
        ElementKind::Sand => tick_sand(position, api),
        ElementKind::Water => tick_water(position, api),
        _ => {}
    }
}

fn tick_sand(position: IVec2, api: &mut LocalApi) {
    let dir = if api.element.0.velocity.x == 0.0 {
        api.random_direction()
    } else {
        api.element.0.velocity.x.signum() as i32
    };

    let can_move_down = api.can_move_to((position.x, position.y + 1).into());

    if can_move_down {
        api.update_element(|element| element.velocity.x *= 0.6);
        api.accelerate(0.0, 0.5);
    } else if api.can_move_to((position.x + dir, position.y + 1).into())
        && api.can_move_to((position.x + dir, position.y).into())
    {
        api.accelerate(0.6 * (dir as f32), 0.5);
    } else if api.can_move_to((position.x - dir, position.y + 1).into())
        && api.can_move_to((position.x - dir, position.y).into())
    {
        api.accelerate(0.6 * (-dir as f32), 0.5);
    }

    api.move_element();
}

fn tick_water(position: IVec2, api: &mut LocalApi) {
    let dir = api.random_direction();

    let can_move_down = api.can_move_to((position.x, position.y + 1).into());

    if can_move_down {
        api.update_element(|element| element.velocity.x *= 0.4);
        api.accelerate(0.0, 0.5);
    } else if api.can_move_to((position.x + dir, position.y + 1).into())
        && api.can_move_to((position.x + dir, position.y).into())
    {
        api.accelerate(0.5 * (dir as f32), 0.5);
    } else if api.can_move_to((position.x - dir, position.y + 1).into())
        && api.can_move_to((position.x - dir, position.y).into())
    {
        api.accelerate(0.5 * (-dir as f32), 0.5);
    } else {
        let dir = if api.element.0.velocity.x == 0.0 {
            api.random_direction()
        } else {
            api.element.0.velocity.x.signum() as i32
        };

        let mut speed = 0.6;
        if api.get_element((position.x, position.y + 1).into()).kind == api.element.0.kind {
            speed += 0.5;
        }

        if api.can_move_to(((position.x + dir), position.y).into()) {
            api.accelerate(speed * (dir as f32), 0.0);
        } else if api.can_move_to(((position.x - dir), position.y).into()) {
            api.accelerate(speed * (-dir as f32), 0.0);
        }
    }

    api.move_element();
}
