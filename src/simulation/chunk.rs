use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use bevy::math::{IVec2, Vec2};

use crate::{common::Rect, constants::*};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ElementKind {
    Air,
    Sand,
    Stone,
    Water,
}

impl ElementKind {
    pub const fn base_color(&self) -> (u8, u8, u8) {
        match self {
            Self::Air => (0, 0, 0),
            Self::Sand => (232, 171, 79),
            Self::Stone => (114, 121, 133),
            Self::Water => (44, 113, 232),
        }
    }

    pub const fn density(&self) -> u8 {
        match self {
            Self::Air => 0,
            Self::Sand => 100,
            Self::Stone => 255,
            Self::Water => 60,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Element {
    pub color: (u8, u8, u8),
    pub velocity: Vec2,
    pub kind: ElementKind,
    pub wframe: u8,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            color: (0, 0, 0),
            velocity: Vec2::ZERO,
            kind: ElementKind::Air,
            wframe: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub position: IVec2,
    pub current_dirty_rect: Rect,
    pub next_dirty_rect: Rect,
    elements: [Element; CHUNK_SIZE * CHUNK_SIZE],
}

#[allow(unused)]
impl Chunk {
    pub fn new(position: IVec2) -> Self {
        Self {
            position,
            current_dirty_rect: Rect::new(IVec2::ZERO, IVec2::splat(CHUNK_SIZE_I32)),
            next_dirty_rect: Rect::new(IVec2::ZERO, IVec2::splat(CHUNK_SIZE_I32)),
            elements: [Element::default(); CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    pub const fn is_outside(position: IVec2) -> bool {
        position.x < 0
            || position.y < 0
            || position.x >= CHUNK_SIZE_I32
            || position.y >= CHUNK_SIZE_I32
    }

    pub fn dirty_rect(&self) -> Rect {
        self.current_dirty_rect
    }

    pub fn active(&self) -> bool {
        !self.current_dirty_rect.is_empty() || !self.next_dirty_rect.is_empty()
    }

    pub fn is_empty(&self, position: IVec2) -> bool {
        self.get_element(position).kind == ElementKind::Air
    }

    pub fn get_element(&self, position: IVec2) -> &Element {
        &self.elements[Self::to_index(position.x, position.y)]
    }

    pub fn get_element_mut(&mut self, position: IVec2) -> &mut Element {
        &mut self.elements[Self::to_index(position.x, position.y)]
    }

    pub fn set_element(&mut self, position: IVec2, element: Element) {
        self.mark_point_dirty(position);
        self.elements[Self::to_index(position.x, position.y)] = element;
    }

    pub fn set_wframe(&mut self, position: IVec2, wframe: u8) {
        self.elements[Self::to_index(position.x, position.y)].wframe = wframe;
    }

    pub fn mark_point_dirty(&mut self, position: IVec2) {
        self.next_dirty_rect
            .union_point_plus(position, IVec2::splat(2));
    }

    pub fn mark_dirty_everything(&mut self) {
        self.next_dirty_rect = Rect::new(IVec2::ZERO, IVec2::splat(CHUNK_SIZE_I32));
        self.current_dirty_rect = self.next_dirty_rect;
    }

    pub const fn to_index(x: i32, y: i32) -> usize {
        (x * CHUNK_SIZE_I32 + y) as usize
    }
}

#[derive(Debug, Clone)]
pub struct SharedChunk {
    inner: Arc<RwLock<Chunk>>,
}

pub type ReadableChunk<'a> = RwLockReadGuard<'a, Chunk>;
pub type WritableChunk<'a> = RwLockWriteGuard<'a, Chunk>;

impl SharedChunk {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            inner: Arc::new(RwLock::new(chunk)),
        }
    }

    pub fn read(&self) -> ReadableChunk {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> WritableChunk {
        self.inner.write().unwrap()
    }
}
