pub mod font;
use alloc::vec;
use alloc::vec::Vec;
use core::iter::Iterator;

/// A structure representing a graphical display with a fixed-size buffer.
///
/// # Fields
///
/// * `buffer` - An array of 64 `u128` values used to store the pixel data of the display.
///   This serves as the internal representation of the display's graphical content.
///
/// * `height` - The height of the display, specified in pixels. This field is publicly accessible
///   within the current crate.
///
/// * `width` - The width of the display, specified in pixels. This field is also publicly accessible
///   within the current crate.
///
/// * `capacity` - The total capacity of the display's buffer, which defines the maximum number of
///   pixels that the display can accommodate. This field is publicly accessible within the current crate.
///
/// # Notes
///
/// This structure is primarily used to handle and store graphical data and dimensions of the display.
/// Access to the `height`, `width`, and `capacity` fields is restricted to the current crate, while
/// the `buffer` remains private to encapsulate the display's graphical state.
pub struct Display {
    planes: [[u128; 64]; 2],
    targeted_plane: TargetPlane,
    pub(crate) width: usize,
    pub(crate) height: usize,
    is_extended: bool,
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// We support up to 4 planes, but normally only planes 1 and 2 are used.
#[derive(Default, Copy, Clone)]
#[repr(u8)]
pub enum TargetPlane {
    None = 0,
    #[default]
    Plane1 = 0b01,
    Plane2 = 0b10,
    Both = 0b11,
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    /// Create a new [`Display`]
    pub fn new() -> Display {
        Display {
            planes: [[0u128; 64]; 2],
            targeted_plane: TargetPlane::Plane1,
            width: 64,
            height: 32,
            is_extended: false,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.planes.iter_mut().for_each(|plane| plane.fill(0))
    }
    pub fn get_screen(&self) -> (&[u128], &[u128]) {
        (&self.planes[0][..], &self.planes[1][..])
    }
    pub fn get_screen_mut(&mut self) -> (&mut [u128], &mut [u128]) {
        let (left, right) = self.planes.split_at_mut(1);
        (left[0].as_mut(), right[0].as_mut())
    }

    pub fn enter_hi_res(&mut self) {
        self.width = 128;
        self.height = 64;
        self.is_extended = true;
    }

    pub fn enter_lo_res(&mut self) {
        self.width = 64;
        self.height = 32;
        self.is_extended = false;
    }

    pub fn is_extended(&self) -> bool {
        self.is_extended
    }

    pub fn dimensions(&self) -> (usize, usize, usize) {
        (self.width, self.height, self.width * self.height)
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn get_plane_idx(&mut self) -> Vec<usize> {
        use TargetPlane::*;
        match self.targeted_plane {
            None => vec![],
            Plane1 => vec![0],
            Plane2 => vec![1],
            Both => vec![0, 1],
        }
    }

    pub(crate) fn for_each_selected_plane<F: FnMut(&mut [u128])>(&mut self, mut func: F) {
        self.get_plane_idx()
            .into_iter()
            .for_each(|plane| func(&mut self.planes[plane]))
    }

    pub(crate) fn set_targeted_plane(&mut self, plane: TargetPlane) {
        self.targeted_plane = plane;
    }

    pub fn draw_line(
        &mut self,
        row: usize,
        col: usize,
        mask: u16,
        plane: usize,
        wrap: bool,
    ) -> bool {
        let mask = if wrap {
            ((mask as u128) << col) | (mask as u128 >> (self.width - col))
        } else {
            (mask as u128) << col
        };
        let mut collisions = 0;
        let buf_line = &mut self.planes[plane][row];
        collisions += ((*buf_line & mask) != 0) as u8;
        *buf_line ^= mask;
        collisions > 0
    }

    pub fn draw_sprite(
        &mut self,
        row: usize,
        col: usize,
        masks: &[u16],
        plane: usize,
        wrap: bool,
    ) -> u8 {
        masks
            .iter()
            .enumerate()
            .fold(0u8, |collisions, (i, &mask)| {
                let curr_row = row + i;
                if curr_row >= self.height && !wrap {
                    collisions
                } else if self.draw_line(curr_row % self.height, col, mask, plane, wrap) {
                    collisions + 1
                } else {
                    collisions
                }
            })
    }

    pub fn scroll_selected_planes_by(&mut self, amount: usize, direction: Direction) {
        if amount == 0 {
            return;
        }
        match direction {
            Direction::Left => {
                self.scroll_left(amount);
            }
            Direction::Right => {
                self.scroll_right(amount);
            }
            Direction::Up => {
                self.scroll_up(amount);
            }
            Direction::Down => {
                self.scroll_down(amount);
            }
        }
    }

    fn scroll_up(&mut self, amount: usize) {
        let height = self.height;
        self.for_each_selected_plane(|plane| {
            for curr_row in 0..height.saturating_sub(amount) {
                plane[curr_row] = plane[curr_row + amount];
            }
            plane.iter_mut().rev().take(amount).for_each(|row| *row = 0);
        });
    }

    fn scroll_down(&mut self, amount: usize) {
        let height = self.height;
        self.for_each_selected_plane(|plane| {
            for curr_row in (0..height.saturating_sub(amount)).rev() {
                plane[curr_row + amount] = plane[curr_row];
            }
            plane.iter_mut().take(amount).for_each(|row| *row = 0);
        });
    }

    // Scrolling left and right is normally by 4 pixels, but some implementations expect 2 pixels
    // on lores, so we add the amount arg to account for that.
    fn scroll_left(&mut self, amount: usize) {
        self.for_each_selected_plane(|plane| plane.iter_mut().for_each(|row| *row >>= amount));
    }

    fn scroll_right(&mut self, amount: usize) {
        self.for_each_selected_plane(|plane| plane.iter_mut().for_each(|row| *row <<= amount));
    }
}
