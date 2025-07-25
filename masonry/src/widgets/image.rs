// Copyright 2020 the Xilem Authors and the Druid Authors
// SPDX-License-Identifier: Apache-2.0

//! An Image widget.
//! Please consider using SVG and the SVG widget as it scales much better.

use accesskit::{Node, Role};
use tracing::{Span, trace_span};
use vello::Scene;
use vello::kurbo::{Affine, Size};
use vello::peniko::{BlendMode, Image as ImageBuf};

use crate::core::{
    AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, ObjectFit, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetId, WidgetMut,
};

// TODO - Resolve name collision between masonry::Image and peniko::Image

/// A widget that renders a bitmap Image.
///
/// The underlying image uses `Arc` for buffer data, making it cheap to clone.
///
/// This currently uses bilinear interpolation, which falls down when the image is
/// larger than its layout size (e.g. it is in a [sized box](super::SizedBox) smaller
/// than the image size).
pub struct Image {
    image_data: ImageBuf,
    object_fit: ObjectFit,
}

// --- MARK: BUILDERS
impl Image {
    /// Create an image drawing widget from an image buffer.
    ///
    /// By default, the Image will scale to fit its box constraints ([`ObjectFit::Fill`]).
    #[inline]
    pub fn new(image_data: ImageBuf) -> Self {
        Self {
            image_data,
            object_fit: ObjectFit::default(),
        }
    }

    /// Builder-style method for specifying the object fit.
    #[inline]
    pub fn fit_mode(mut self, mode: ObjectFit) -> Self {
        self.object_fit = mode;
        self
    }
}

// --- MARK: WIDGETMUT
impl Image {
    /// Modify the widget's object fit.
    #[inline]
    pub fn set_fit_mode(this: &mut WidgetMut<'_, Self>, new_object_fit: ObjectFit) {
        this.widget.object_fit = new_object_fit;
        this.ctx.request_layout();
    }

    /// Set new `ImageBuf`.
    #[inline]
    pub fn set_image_data(this: &mut WidgetMut<'_, Self>, image_data: ImageBuf) {
        this.widget.image_data = image_data;
        this.ctx.request_layout();
    }
}

// --- MARK: IMPL WIDGET
impl Widget for Image {
    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        // If either the width or height is constrained calculate a value so that the image fits
        // in the size exactly. If it is unconstrained by both width and height take the size of
        // the image.
        let image_size = Size::new(self.image_data.width as f64, self.image_data.height as f64);
        if image_size.is_zero_area() {
            let size = bc.min();
            return size;
        }
        let image_aspect_ratio = image_size.height / image_size.width;
        match self.object_fit {
            ObjectFit::Contain => bc.constrain_aspect_ratio(image_aspect_ratio, image_size.width),
            ObjectFit::Cover => Size::new(bc.max().width, bc.max().width * image_aspect_ratio),
            ObjectFit::Fill => bc.max(),
            ObjectFit::FitHeight => {
                Size::new(bc.max().height / image_aspect_ratio, bc.max().height)
            }
            ObjectFit::FitWidth => Size::new(bc.max().width, bc.max().width * image_aspect_ratio),
            ObjectFit::None => image_size,
            ObjectFit::ScaleDown => {
                let mut size = image_size;

                if !bc.contains(size) {
                    size = bc.constrain_aspect_ratio(image_aspect_ratio, size.width);
                }

                size
            }
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        let image_size = Size::new(self.image_data.width as f64, self.image_data.height as f64);
        let transform = self.object_fit.affine_to_fill(ctx.size(), image_size);

        let clip_rect = ctx.size().to_rect();
        scene.push_layer(BlendMode::default(), 1., Affine::IDENTITY, &clip_rect);
        scene.draw_image(&self.image_data, transform);
        scene.pop_layer();
    }

    fn accessibility_role(&self) -> Role {
        Role::Image
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
        // TODO - Handle alt text and such.
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("Image", id = id.trace())
    }
}

// FIXME - remove cfg?
#[cfg(not(target_arch = "wasm32"))]
// --- MARK: TESTS
#[cfg(test)]
mod tests {
    use vello::peniko::ImageFormat;

    use super::*;
    use crate::testing::{TestHarness, assert_render_snapshot};
    use crate::theme::default_property_set;

    /// Painting an empty image shouldn't crash.
    #[test]
    fn empty_paint() {
        // TODO - Blob::empty() function?
        let image_data = ImageBuf::new(Vec::new().into(), ImageFormat::Rgba8, 0, 0);

        let image_widget = Image::new(image_data);
        let mut harness = TestHarness::create(default_property_set(), image_widget);
        let _ = harness.render();
    }

    #[test]
    fn tall_paint() {
        let image_data = ImageBuf::new(
            // This could have a more concise chain, but previously used versions either
            // had unreadable formatting or used `rustfmt::skip`, which broke formatting
            // across large parts of the file.
            [
                [255, 255, 255, 255],
                [000, 000, 000, 255],
                [000, 000, 000, 255],
                [255, 255, 255, 255],
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .into(),
            ImageFormat::Rgba8,
            2,
            2,
        );
        let image_widget = Image::new(image_data);

        let mut harness = TestHarness::create_with_size(
            default_property_set(),
            image_widget,
            Size::new(40., 60.),
        );
        assert_render_snapshot!(harness, "image_tall_paint");
    }

    #[test]
    fn edit_image() {
        let image_data = ImageBuf::new(vec![255; 4 * 8 * 8].into(), ImageFormat::Rgba8, 8, 8);

        let render_1 = {
            let image_widget = Image::new(image_data.clone());

            let mut harness = TestHarness::create_with_size(
                default_property_set(),
                image_widget,
                Size::new(40.0, 60.0),
            );

            harness.render()
        };

        let render_2 = {
            let other_image_data =
                ImageBuf::new(vec![10; 4 * 8 * 8].into(), ImageFormat::Rgba8, 8, 8);
            let image_widget = Image::new(other_image_data);

            let mut harness = TestHarness::create_with_size(
                default_property_set(),
                image_widget,
                Size::new(40.0, 60.0),
            );

            harness.edit_root_widget(|mut image| {
                Image::set_image_data(&mut image, image_data);
            });

            harness.render()
        };

        // TODO - Use Kompari instead
        // We don't use assert_eq because we don't want rich assert
        assert!(render_1 == render_2);
    }

    #[test]
    fn layout() {
        let image_data = ImageBuf::new(vec![255; 4 * 8 * 8].into(), ImageFormat::Rgba8, 8, 8);
        let harness_size = Size::new(100.0, 50.0);

        // Contain.
        let image_widget = Image::new(image_data.clone()).fit_mode(ObjectFit::Contain);
        let mut harness =
            TestHarness::create_with_size(default_property_set(), image_widget, harness_size);
        assert_render_snapshot!(harness, "image_layout_contain");

        // Cover.
        let image_widget = Image::new(image_data.clone()).fit_mode(ObjectFit::Cover);
        let mut harness =
            TestHarness::create_with_size(default_property_set(), image_widget, harness_size);
        assert_render_snapshot!(harness, "image_layout_cover");

        // Fill.
        let image_widget = Image::new(image_data.clone()).fit_mode(ObjectFit::Fill);
        let mut harness =
            TestHarness::create_with_size(default_property_set(), image_widget, harness_size);
        assert_render_snapshot!(harness, "image_layout_fill");

        // FitHeight.
        let image_widget = Image::new(image_data.clone()).fit_mode(ObjectFit::FitHeight);
        let mut harness =
            TestHarness::create_with_size(default_property_set(), image_widget, harness_size);
        assert_render_snapshot!(harness, "image_layout_fitheight");

        // FitWidth.
        let image_widget = Image::new(image_data.clone()).fit_mode(ObjectFit::FitWidth);
        let mut harness =
            TestHarness::create_with_size(default_property_set(), image_widget, harness_size);
        assert_render_snapshot!(harness, "image_layout_fitwidth");

        // None.
        let image_widget = Image::new(image_data.clone()).fit_mode(ObjectFit::None);
        let mut harness =
            TestHarness::create_with_size(default_property_set(), image_widget, harness_size);
        assert_render_snapshot!(harness, "image_layout_none");

        // ScaleDown.
        let image_widget = Image::new(image_data.clone()).fit_mode(ObjectFit::ScaleDown);
        let mut harness =
            TestHarness::create_with_size(default_property_set(), image_widget, harness_size);
        assert_render_snapshot!(harness, "image_layout_scaledown");
    }
}
