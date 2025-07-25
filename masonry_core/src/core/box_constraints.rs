// Copyright 2019 the Xilem Authors and the Druid Authors
// SPDX-License-Identifier: Apache-2.0

use vello::kurbo::Size;

/// Constraints for layout.
///
/// The layout strategy for Masonry is strongly inspired by Flutter,
/// and this struct is similar to the [Flutter BoxConstraints] class.
///
/// At the moment, it represents simply a minimum and maximum size.
/// A widget's [`layout`] method should choose an appropriate size that
/// meets these constraints.
///
/// Further, a container widget should compute appropriate constraints
/// for each of its child widgets, and pass those down when recursing.
///
/// The constraints are always [rounded away from zero] to integers
/// to enable pixel perfect layout.
///
/// [`layout`]: crate::core::Widget::layout
/// [Flutter BoxConstraints]: https://api.flutter.dev/flutter/rendering/BoxConstraints-class.html
/// [rounded away from zero]: Size::expand
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    min: Size,
    max: Size,
}

impl BoxConstraints {
    /// An unbounded box constraints object.
    ///
    /// Can be satisfied by any nonnegative size.
    pub const UNBOUNDED: Self = Self {
        min: Size::ZERO,
        max: Size::new(f64::INFINITY, f64::INFINITY),
    };

    /// Create a new box constraints object.
    ///
    /// Create constraints based on minimum and maximum size.
    ///
    /// The given sizes are also [rounded away from zero],
    /// so that the layout is aligned to integers.
    ///
    /// [rounded away from zero]: Size::expand
    pub fn new(min: Size, max: Size) -> Self {
        Self {
            min: min.expand(),
            max: max.expand(),
        }
    }

    /// Create a "tight" box constraints object.
    ///
    /// A "tight" constraint can only be satisfied by a single size.
    ///
    /// The given size is also [rounded away from zero],
    /// so that the layout is aligned to integers.
    ///
    /// [rounded away from zero]: Size::expand
    pub fn tight(size: Size) -> Self {
        let size = size.expand();
        Self {
            min: size,
            max: size,
        }
    }

    /// Create a "loose" version of the constraints.
    ///
    /// Make a version with zero minimum size, but the same maximum size.
    pub fn loosen(&self) -> Self {
        Self {
            min: Size::ZERO,
            max: self.max,
        }
    }

    /// Clamp a given size so that it fits within the constraints.
    ///
    /// The given size is also [rounded away from zero],
    /// so that the layout is aligned to integers.
    ///
    /// [rounded away from zero]: Size::expand
    pub fn constrain(&self, size: impl Into<Size>) -> Size {
        size.into().expand().clamp(self.min, self.max)
    }

    /// Returns the max size of these constraints.
    pub fn max(&self) -> Size {
        self.max
    }

    /// Returns the min size of these constraints.
    pub fn min(&self) -> Size {
        self.min
    }

    /// Whether there is an upper bound on the width.
    pub fn is_width_bounded(&self) -> bool {
        self.max.width.is_finite()
    }

    /// Whether there is an upper bound on the height.
    pub fn is_height_bounded(&self) -> bool {
        self.max.height.is_finite()
    }

    /// Return the max in each axis if bounded, else the value given in `size`.
    pub fn bounded_or(&self, size: Size) -> Size {
        Size {
            width: if self.is_width_bounded() {
                self.max.width
            } else {
                size.width
            },
            height: if self.is_height_bounded() {
                self.max.height
            } else {
                size.height
            },
        }
    }

    /// Check to see if these constraints are legit.
    ///
    /// In Debug mode, logs a warning if `BoxConstraints` are invalid.
    pub fn debug_check(&self, name: &str) {
        if cfg!(not(debug_assertions)) {
            return;
        }

        if self.min.width.is_nan() {
            debug_panic!("Minimum width constraint passed to {name} is NaN");
        }
        if self.min.height.is_nan() {
            debug_panic!("Minimum height constraint passed to {name} is NaN");
        }
        if self.max.width.is_nan() {
            debug_panic!("Maximum width constraint passed to {name} is NaN");
        }
        if self.max.height.is_nan() {
            debug_panic!("Maximum height constraint passed to {name} is NaN");
        }

        if self.min.width.is_infinite() {
            debug_panic!("Infinite minimum width constraint passed to {name}");
        }
        if self.min.height.is_infinite() {
            debug_panic!("Infinite minimum height constraint passed to {name}");
        }

        if !(0.0 <= self.min.width
            && self.min.width <= self.max.width
            && 0.0 <= self.min.height
            && self.min.height <= self.max.height
            && self.min.expand() == self.min
            && self.max.expand() == self.max)
        {
            debug_panic!("Bad BoxConstraints passed to {name}: {self:?}",);
        }
    }

    /// Shrink min and max constraints by size
    ///
    /// The given size is also [rounded away from zero],
    /// so that the layout is aligned to integers.
    ///
    /// [rounded away from zero]: Size::expand
    pub fn shrink(&self, diff: impl Into<Size>) -> Self {
        let diff = diff.into().expand();
        let min = Size::new(
            (self.min().width - diff.width).max(0.),
            (self.min().height - diff.height).max(0.),
        );
        let max = Size::new(
            (self.max().width - diff.width).max(0.),
            (self.max().height - diff.height).max(0.),
        );

        Self::new(min, max)
    }

    /// Test whether these constraints contain the given `Size`.
    pub fn contains(&self, size: impl Into<Size>) -> bool {
        let size = size.into();
        (self.min.width <= size.width && size.width <= self.max.width)
            && (self.min.height <= size.height && size.height <= self.max.height)
    }

    /// Find the `Size` within these `BoxConstraint`s that minimises the difference between the
    /// returned `Size`'s aspect ratio and `aspect_ratio`, where *aspect ratio* is defined as
    /// `height / width`.
    ///
    /// If multiple `Size`s give the optimal `aspect_ratio`, then the one with the `width` nearest
    /// the supplied width will be used. Specifically, if `width == 0.0` then the smallest possible
    /// `Size` will be chosen, and likewise if `width == f64::INFINITY`, then the largest `Size`
    /// will be chosen.
    ///
    /// Use this function when maintaining an aspect ratio is more important than minimizing the
    /// distance between input and output size width and height.
    ///
    /// ## Panics
    ///
    /// Panics if `aspect_ratio` or `width` are NaN, infinite or negative.
    #[track_caller]
    pub fn constrain_aspect_ratio(&self, aspect_ratio: f64, width: f64) -> Size {
        assert!(
            aspect_ratio.is_finite(),
            "aspect_ratio must be a finite value"
        );
        assert!(width.is_finite(), "width must be finite value");
        assert!(aspect_ratio >= 0.0, "aspect_ratio must be 0.0 or greater");
        assert!(width >= 0.0, "width must be 0.0 or greater");

        // Minimizing/maximizing based on aspect ratio seems complicated, but in reality everything
        // is linear, so the amount of work to do is low.
        let ideal_size = Size {
            width,
            height: width * aspect_ratio,
        };

        // It may be possible to remove these in the future if the invariant is checked elsewhere.
        let aspect_ratio = aspect_ratio.abs();
        let width = width.abs();

        // Firstly check if we can simply return the exact requested
        if self.contains(ideal_size) {
            return ideal_size;
        }

        let min_w_min_h = self.min.height / self.min.width;
        let max_w_min_h = self.min.height / self.max.width;
        let min_w_max_h = self.max.height / self.min.width;
        let max_w_max_h = self.max.height / self.max.width;

        // When the aspect ratio line crosses the constraints, the closest point must be one of the
        // two points where the aspect ratio enters/exits.

        // When the aspect ratio line doesn't intersect the box of possible sizes, the closest
        // point must be either (max width, min height) or (max height, min width). So all we have
        // to do is check which one of these has the closest aspect ratio.

        // Check each possible intersection (or not) of the aspect ratio line with the constraints
        if aspect_ratio > min_w_max_h {
            // outside max height min width
            Size {
                width: self.min.width,
                height: self.max.height,
            }
        } else if aspect_ratio < max_w_min_h {
            // outside min height max width
            Size {
                width: self.max.width,
                height: self.min.height,
            }
        } else if aspect_ratio > min_w_min_h {
            // hits the constraints on the min width line
            if width < self.min.width {
                // we take the point on the min width
                Size {
                    width: self.min.width,
                    height: self.min.width * aspect_ratio,
                }
            } else if aspect_ratio < max_w_max_h {
                // exits through max.width
                Size {
                    width: self.max.width,
                    height: self.max.width * aspect_ratio,
                }
            } else {
                // exits through max.height
                Size {
                    width: self.max.height * aspect_ratio.recip(),
                    height: self.max.height,
                }
            }
        } else {
            // final case is where we hit constraints on the min height line
            if width < self.min.width {
                // take the point on the min height
                Size {
                    width: self.min.height * aspect_ratio.recip(),
                    height: self.min.height,
                }
            } else if aspect_ratio > max_w_max_h {
                // exit thru max height
                Size {
                    width: self.max.height * aspect_ratio.recip(),
                    height: self.max.height,
                }
            } else {
                // exit thru max width
                Size {
                    width: self.max.width,
                    height: self.max.width * aspect_ratio,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bc(min_width: f64, min_height: f64, max_width: f64, max_height: f64) -> BoxConstraints {
        BoxConstraints::new(
            Size::new(min_width, min_height),
            Size::new(max_width, max_height),
        )
    }

    #[test]
    fn constrain_aspect_ratio() {
        for (bc, aspect_ratio, width, output) in [
            // The ideal size lies within the constraints
            (bc(0.0, 0.0, 100.0, 100.0), 1.0, 50.0, Size::new(50.0, 50.0)),
            (bc(0.0, 10.0, 90.0, 100.0), 1.0, 50.0, Size::new(50.0, 50.0)),
            // The correct aspect ratio is available (but not width)
            // min height
            (
                bc(10.0, 10.0, 100.0, 100.0),
                1.0,
                5.0,
                Size::new(10.0, 10.0),
            ),
            (
                bc(40.0, 90.0, 60.0, 100.0),
                2.0,
                30.0,
                Size::new(45.0, 90.0),
            ),
            (
                bc(10.0, 10.0, 100.0, 100.0),
                0.5,
                5.0,
                Size::new(20.0, 10.0),
            ),
            // min width
            (
                bc(10.0, 10.0, 100.0, 100.0),
                2.0,
                5.0,
                Size::new(10.0, 20.0),
            ),
            (
                bc(90.0, 40.0, 100.0, 60.0),
                0.5,
                60.0,
                Size::new(90.0, 45.0),
            ),
            (
                bc(50.0, 0.0, 50.0, 100.0),
                1.0,
                100.0,
                Size::new(50.0, 50.0),
            ),
            // max height
            (
                bc(10.0, 10.0, 100.0, 100.0),
                2.0,
                105.0,
                Size::new(50.0, 100.0),
            ),
            (
                bc(10.0, 10.0, 100.0, 100.0),
                0.5,
                105.0,
                Size::new(100.0, 50.0),
            ),
            // The correct aspect ratio is not available
            (
                bc(20.0, 20.0, 40.0, 40.0),
                10.0,
                30.0,
                Size::new(20.0, 40.0),
            ),
            (bc(20.0, 20.0, 40.0, 40.0), 0.1, 30.0, Size::new(40.0, 20.0)),
            // non-finite
            (
                bc(50.0, 0.0, 50.0, f64::INFINITY),
                1.0,
                100.0,
                Size::new(50.0, 50.0),
            ),
        ]
        .iter()
        {
            assert_eq!(
                bc.constrain_aspect_ratio(*aspect_ratio, *width),
                *output,
                "bc:{bc:?}, aspect_ratio:{aspect_ratio}, width:{width}",
            );
        }
    }

    #[test]
    fn unbounded() {
        assert!(!BoxConstraints::UNBOUNDED.is_width_bounded());
        assert!(!BoxConstraints::UNBOUNDED.is_height_bounded());

        assert_eq!(BoxConstraints::UNBOUNDED.min(), Size::ZERO);
    }
}
