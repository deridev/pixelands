use bevy::prelude::IVec2;

pub struct GridLineIterator {
    pos1: IVec2,
    pos2: IVec2,
    modifier: IVec2,
    upper_bound: i32,
    slope: f32,
    x_diff_is_larger: bool,
    index: i32,
}

impl GridLineIterator {
    pub fn new(pos1: IVec2, pos2: IVec2) -> Self {
        let diff = pos1 - pos2;
        let x_diff_is_larger = diff.x.abs() > diff.y.abs();

        let modifier = IVec2::new(
            if diff.x < 0 { 1 } else { -1 },
            if diff.y < 0 { 1 } else { -1 },
        );
        let upper_bound = diff.x.abs().max(diff.y.abs());
        let min = diff.x.abs().min(diff.y.abs());
        let slope = if min == 0 || upper_bound == 0 {
            0.0
        } else {
            (min + 1) as f32 / (upper_bound + 1) as f32
        };

        Self {
            pos1,
            pos2,
            modifier,
            upper_bound,
            slope,
            x_diff_is_larger,
            index: 0,
        }
    }
}

impl Iterator for GridLineIterator {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos1 == self.pos2 || self.index == self.upper_bound {
            if self.index == 0 {
                self.index += 1;
                return Some(self.pos1);
            }
            return None;
        }

        self.index += 1;

        let smaller_count = (self.index as f32 * self.slope) as i32;
        let (y_increase, x_increase) = if self.x_diff_is_larger {
            (smaller_count, self.index)
        } else {
            (self.index, smaller_count)
        };
        let current_x = self.pos1.x + (x_increase * self.modifier.x);
        let current_y = self.pos1.y + (y_increase * self.modifier.y);

        Some((current_x, current_y).into())
    }
}
