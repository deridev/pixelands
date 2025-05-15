use bevy::math::{IVec2, Vec2};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    common::{
        directions::{self, dir_to_index},
        math,
    },
    constants::CHUNK_SIZE_I32,
    coordinates,
};

use super::*;

#[allow(unused)]
pub struct LocalApi {
    pub center: IVec2,
    pub chunks: Vec<Option<SharedChunk>>,
    pub new_chunks: Vec<usize>,
    pub element: (Element, IVec2),
    pub wframe: u8,
    rng: StdRng,
}

impl LocalApi {
    pub fn new(
        center: IVec2,
        wframe: u8,
        element: (Element, IVec2),
        chunks: Vec<Option<SharedChunk>>,
    ) -> Self {
        Self {
            center,
            chunks,
            element,
            wframe,
            new_chunks: Vec::with_capacity(8),
            rng: StdRng::from_os_rng(),
        }
    }

    fn inner_get_element(&self, chunk_index: usize, element_position: IVec2) -> Element {
        match &self.chunks[chunk_index] {
            None => return Element::default(),
            Some(shared_chunk) => *shared_chunk.read().get_element(element_position),
        }
    }

    fn inner_chunk_index_and_element_position(&self, relative_position: IVec2) -> (usize, IVec2) {
        // Treat the (position) like a world position - it is actually an element position that can reference outside positions.
        // For example: (0, 0) would refer to the element at (0, 0) in the chunk at the center.
        // ^ (-1, 0) would refer to the element at it's left, on the left chunk.
        let chunk_relative_position =
            coordinates::world_to_chunk_position(relative_position.as_vec2());

        let chunk_index =
            ((chunk_relative_position.x + 1) + (chunk_relative_position.y + 1) * 3) as usize;

        let element_position = coordinates::world_to_element_position(relative_position.as_vec2());
        (chunk_index, element_position)
    }

    fn chunk_index_exists(&self, chunk_index: usize) -> bool {
        self.chunks.len() > chunk_index && self.chunks[chunk_index].is_some()
    }

    pub fn random_direction(&mut self) -> i32 {
        if self.rng.random_bool(0.5) {
            1
        } else {
            -1
        }
    }

    pub fn can_move_to(&self, position: IVec2) -> bool {
        let (chunk_index, element_position) = self.inner_chunk_index_and_element_position(position);
        if !self.chunk_index_exists(chunk_index) {
            return false;
        }

        let dest = self.inner_get_element(chunk_index, element_position);
        dest.kind == ElementKind::Air || dest.kind.density() < self.element.0.kind.density()
    }

    pub fn get_element(&self, position: IVec2) -> Element {
        let (chunk_index, element_position) = self.inner_chunk_index_and_element_position(position);
        if !self.chunk_index_exists(chunk_index) {
            return Element {
                wframe: 0,
                color: (245, 226, 24),
                velocity: Vec2::ZERO,
                kind: ElementKind::Sand,
            };
        }

        self.inner_get_element(chunk_index, element_position)
    }

    pub fn set_wframe(&mut self, wframe: u8) {
        let (chunk_index, element_position) =
            self.inner_chunk_index_and_element_position(self.element.1);
        if !self.chunk_index_exists(chunk_index) {
            return;
        }

        self.chunks[chunk_index]
            .as_ref()
            .unwrap()
            .write()
            .set_wframe(element_position, wframe);
    }

    pub fn swap_elements(&mut self, source: IVec2, target: IVec2, source_element: Option<Element>) {
        let (source_chunk_index, ..) = self.inner_chunk_index_and_element_position(source);
        let (target_chunk_index, ..) = self.inner_chunk_index_and_element_position(target);

        if !self.chunk_index_exists(source_chunk_index) {
            return;
        }

        if !self.chunk_index_exists(target_chunk_index) {
            return;
        }

        let source_element = source_element.unwrap_or(self.get_element(source));
        let target_element = self.get_element(target);

        self.set_element(source, target_element);
        self.set_element(target, source_element);
    }

    pub fn set_element(&mut self, position: IVec2, element: Element) {
        let (chunk_index, element_position) = self.inner_chunk_index_and_element_position(position);
        if !self.chunk_index_exists(chunk_index) {
            let chunk_world_position =
                self.center + coordinates::world_to_chunk_position(position.as_vec2());

            let chunk = SharedChunk::new(Chunk::new(chunk_world_position));
            self.new_chunks.push(chunk_index);

            self.chunks[chunk_index] = Some(chunk);
        }

        // Mark neighboring chunks as dirty when setting elements on the edge
        if element_position.x == 0 {
            if let Some(chunk) = self.chunks[dir_to_index(directions::VEC_LEFT)].as_ref() {
                chunk
                    .write()
                    .mark_point_dirty(IVec2::new(CHUNK_SIZE_I32 - 1, element_position.y));
            }
        } else if element_position.x == CHUNK_SIZE_I32 - 1 {
            if let Some(chunk) = self.chunks[dir_to_index(directions::VEC_RIGHT)].as_ref() {
                chunk
                    .write()
                    .mark_point_dirty(IVec2::new(0, element_position.y));
            }
        }

        if element_position.y == 0 {
            if let Some(chunk) = self.chunks[dir_to_index(directions::VEC_UP)].as_ref() {
                chunk
                    .write()
                    .mark_point_dirty(IVec2::new(element_position.x, CHUNK_SIZE_I32 - 1));
            }
        } else if element_position.y == CHUNK_SIZE_I32 - 1 {
            if let Some(chunk) = self.chunks[dir_to_index(directions::VEC_DOWN)].as_ref() {
                chunk
                    .write()
                    .mark_point_dirty(IVec2::new(element_position.x, 0));
            }
        }

        self.chunks[chunk_index]
            .as_ref()
            .unwrap()
            .write()
            .set_element(element_position, element);
    }

    pub fn mark_element_dirty(&mut self) {
        let (chunk_index, element_position) =
            self.inner_chunk_index_and_element_position(self.element.1);
        if !self.chunk_index_exists(chunk_index) {
            return;
        }

        self.chunks[chunk_index]
            .as_ref()
            .unwrap()
            .write()
            .mark_point_dirty(element_position);
    }

    pub fn update_element(&mut self, callback: impl FnOnce(&mut Element)) {
        let (chunk_index, element_position) =
            self.inner_chunk_index_and_element_position(self.element.1);
        if !self.chunk_index_exists(chunk_index) {
            return;
        }

        let mut chunk = self.chunks[chunk_index].as_ref().unwrap().write();
        let element = chunk.get_element_mut(element_position);
        element.wframe = self.wframe;

        callback(element);

        self.element.0 = *element;
    }

    pub fn accelerate(&mut self, x: f32, y: f32) {
        self.update_element(|element| {
            element.velocity.x = (element.velocity.x + x).clamp(-10.0, 10.0);
            element.velocity.y = (element.velocity.y + y).clamp(-10.0, 10.0);
        });

        self.mark_element_dirty();
    }

    pub fn move_element(&mut self) {
        let (chunk_index, element_position) =
            self.inner_chunk_index_and_element_position(self.element.1);
        if !self.chunk_index_exists(chunk_index) {
            return;
        }

        let start_position = element_position;
        let end_position = start_position + self.element.0.velocity.as_ivec2();

        if start_position == end_position {
            return;
        }

        let iter = math::GridLineIterator::new(start_position, end_position);
        let mut destination = start_position;

        for pos in iter {
            if pos == start_position {
                continue;
            }

            if self.can_move_to(pos) {
                destination = pos;
            } else {
                break;
            }
        }

        // Move the element
        if destination != start_position {
            self.swap_elements(start_position, destination, Some(self.element.0));
            self.element.1 = destination;

            // If moved along the X axis and not the Y axis, apply some friction
            if destination.x != start_position.x && destination.y == start_position.y {
                self.update_element(|element| element.velocity.x *= 0.8);
            }
        } else {
            // Decelerate the element a lot with a little bit of bouncing
            self.update_element(|element| {
                element.velocity *= -0.1;

                if element.velocity.length() < f32::EPSILON {
                    element.velocity = Vec2::ZERO;
                }
            });
        }
    }
}
