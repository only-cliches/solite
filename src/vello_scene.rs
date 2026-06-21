//! A [`PaintScene`] adapter that drives a [`vello::Scene`] directly on the host
//! application's wgpu device.
//!
//! This is a trimmed vendored copy of `anyrender_vello`'s `VelloScenePainter`
//! (anyrender_vello 0.11.0, MIT/Apache-2.0). The upstream type only exposes a
//! public constructor that sets `renderer`/`texture_handles` to `None`, which
//! disables raster-image rendering. We need both so we can paint directly into
//! a host-owned GPU texture — eliminating the CPU readback + re-upload the
//! `VelloImageRenderer` path performs every frame — while still registering
//! `<img>`/`background-image` textures. The `device_handle` field (used only by
//! custom paint widgets, which solite does not use) and its
//! `renderer_specific_context` override are dropped; the trait default returns
//! `None`.

use std::collections::HashMap;
use std::sync::Arc;

use anyrender::{Filter, NormalizedCoord, Paint, PaintRef, PaintScene, RenderContext, ResourceId};
use kurbo::{Affine, Rect, Shape, Stroke};
use peniko::{BlendMode, BrushRef, Color, Fill, FontData, ImageBrush, ImageData, StyleRef};
use vello::Renderer as VelloRenderer;
use wgpu::Texture;

/// Maps anyrender resource ids to the vello image handles minted for them.
pub(crate) type TextureHandles = HashMap<ResourceId, ImageData>;

pub(crate) struct VelloScenePainter<'a> {
    renderer: &'a mut VelloRenderer,
    texture_handles: &'a mut TextureHandles,
    inner: &'a mut vello::Scene,
}

impl<'a> VelloScenePainter<'a> {
    pub(crate) fn new(
        renderer: &'a mut VelloRenderer,
        texture_handles: &'a mut TextureHandles,
        scene: &'a mut vello::Scene,
    ) -> Self {
        Self {
            renderer,
            texture_handles,
            inner: scene,
        }
    }
}

impl RenderContext for VelloScenePainter<'_> {
    fn try_register_custom_resource(
        &mut self,
        resource: Box<dyn std::any::Any>,
    ) -> Result<ResourceId, anyrender::RegisterResourceError> {
        if let Ok(texture) = resource.downcast::<Texture>() {
            let id = ResourceId::new();
            self.texture_handles
                .insert(id, self.renderer.register_texture(*texture));
            Ok(id)
        } else {
            Err(anyrender::RegisterResourceErrorKind::UnsupportedResourceKind.into())
        }
    }

    fn unregister_resource(&mut self, resource_id: ResourceId) {
        if let Some(handle) = self.texture_handles.remove(&resource_id) {
            self.renderer.unregister_texture(handle);
        }
    }
}

impl PaintScene for VelloScenePainter<'_> {
    fn reset(&mut self) {
        self.inner.reset();
    }

    fn push_layer(
        &mut self,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        clip: &impl Shape,
        _filter: Option<Arc<Filter>>,
        _backdrop_filter: Option<Arc<Filter>>,
    ) {
        self.inner
            .push_layer(Fill::NonZero, blend, alpha, transform, clip);
    }

    fn push_clip_layer(&mut self, transform: Affine, clip: &impl Shape) {
        self.inner.push_clip_layer(Fill::NonZero, transform, clip);
    }

    fn pop_layer(&mut self) {
        self.inner.pop_layer();
    }

    fn stroke<'a>(
        &mut self,
        style: &Stroke,
        transform: Affine,
        paint_ref: impl Into<PaintRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let paint_ref: PaintRef<'_> = paint_ref.into();
        let brush_ref: BrushRef<'_> = paint_ref.into();
        self.inner
            .stroke(style, transform, brush_ref, brush_transform, shape);
    }

    fn fill<'a>(
        &mut self,
        style: Fill,
        transform: Affine,
        paint: impl Into<PaintRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let paint: PaintRef<'_> = paint.into();
        let brush_ref: BrushRef<'_> = match paint {
            Paint::Solid(color) => BrushRef::Solid(color),
            Paint::Gradient(gradient) => BrushRef::Gradient(gradient),
            Paint::Image(image) => BrushRef::Image(image),
            Paint::Resource(brush) => {
                let resource_id = brush.image;
                if let Some(texture_handle) = self.texture_handles.get(&resource_id) {
                    peniko::Brush::Image(ImageBrush {
                        image: texture_handle,
                        sampler: brush.sampler,
                    })
                } else {
                    BrushRef::Solid(Color::TRANSPARENT)
                }
            }
            Paint::Custom(_) => BrushRef::Solid(Color::TRANSPARENT),
        };

        self.inner
            .fill(style, transform, brush_ref, brush_transform, shape);
    }

    fn draw_glyphs<'a, 's: 'a>(
        &'a mut self,
        font: &'a FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: &'a [NormalizedCoord],
        embolden: kurbo::Vec2,
        style: impl Into<StyleRef<'a>>,
        paint: impl Into<PaintRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = anyrender::Glyph>,
    ) {
        self.inner
            .draw_glyphs(font)
            .font_size(font_size)
            .hint(hint)
            .normalized_coords(normalized_coords)
            .font_embolden(vello::FontEmbolden::new(kurbo::Diagonal2::new(
                embolden.x, embolden.y,
            )))
            .brush(paint.into())
            .brush_alpha(brush_alpha)
            .transform(transform)
            .glyph_transform(glyph_transform)
            .draw(
                style,
                glyphs.map(|g: anyrender::Glyph| vello::Glyph {
                    id: g.id,
                    x: g.x,
                    y: g.y,
                }),
            );
    }

    fn draw_box_shadow(
        &mut self,
        transform: Affine,
        rect: Rect,
        brush: Color,
        radius: f64,
        std_dev: f64,
    ) {
        self.inner
            .draw_blurred_rounded_rect(transform, rect, brush, radius, std_dev);
    }
}
