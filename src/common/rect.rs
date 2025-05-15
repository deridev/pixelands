use bevy::math::{IVec2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    /// The top-left corner of the rectangle.
    pub min: IVec2,
    /// The bottom-right corner of the rectangle.
    pub max: IVec2,
}

#[allow(unused)]
impl Rect {
    pub fn new(min: IVec2, max: IVec2) -> Self {
        Self { min, max }
    }

    pub const fn empty() -> Self {
        Self {
            min: IVec2::MAX,
            max: IVec2::MIN,
        }
    }

    pub fn is_empty(&self) -> bool {
        (self.min == IVec2::MAX && self.max == IVec2::MIN)
            || self.min.x > self.max.x
            || self.min.y > self.max.y
    }

    pub fn size(&self) -> IVec2 {
        self.max - self.min
    }

    pub fn clear(&mut self) {
        *self = Self::empty();
    }

    pub fn contains(&self, point: IVec2) -> bool {
        point.x >= self.min.x
            && point.y >= self.min.y
            && point.x < self.max.x
            && point.y < self.max.y
    }

    pub fn union(&self, other: &Rect) -> Self {
        if self.is_empty() && !other.is_empty() {
            return *other;
        } else if other.is_empty() && !self.is_empty() {
            return *self;
        } else if other.is_empty() && self.is_empty() {
            return Self::empty();
        }

        Self::new(self.min.min(other.min), self.max.max(other.max))
    }

    pub fn union_point(&mut self, point: IVec2) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn union_point_plus(&mut self, point: IVec2, plus: IVec2) {
        self.min = self.min.min(point - plus);
        self.max = self.max.max(point + plus);

        // Bounds check
        if self.min.x < 0 {
            self.min.x = 0;
        }

        if self.min.y < 0 {
            self.min.y = 0;
        }

        if self.min.x > self.max.x {
            self.min.x = self.max.x;
        }

        if self.min.y > self.max.y {
            self.min.y = self.max.y;
        }
    }

    pub fn center(&self) -> IVec2 {
        (self.min + self.max) / 2
    }

    pub fn center_display(&self) -> Vec2 {
        self.center().as_vec2() + Vec2::ONE
    }
}
