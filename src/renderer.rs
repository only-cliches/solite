use anyrender::PaintScene;
use blitz_dom::BaseDocument;
use kurbo::{Affine, Rect};
use peniko::Fill;
use std::sync::Arc;

use crate::scrollbar::{ScrollbarColors, ScrollbarRegion};
use crate::spinner::NumberSpinner;
use crate::vello_scene::{TextureHandles, VelloScenePainter};

#[derive(Debug, Clone, Copy)]
pub(crate) struct InputCaret {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: peniko::Color,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InputSelection {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Renderer: blitz-dom layout + paint via Vello.
///
/// Vello paints **directly into the host-owned wgpu texture** on the host
/// device — no CPU readback and no re-upload.
pub(crate) struct Painter {
    width: u32,
    height: u32,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    vello: vello::Renderer,
    scene: vello::Scene,
    /// Vello image handles for any `<img>`/`background-image` textures
    /// registered while painting the scene.
    texture_handles: TextureHandles,
}

impl Painter {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        width: u32,
        height: u32,
    ) -> Self {
        let vello = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                use_cpu: false,
                num_init_threads: vello_init_threads(),
                antialiasing_support: vello::AaSupport::area_only(),
                pipeline_cache: None,
            },
        )
        .expect("create vello renderer on host device");
        Self {
            width,
            height,
            device,
            queue,
            vello,
            scene: vello::Scene::new(),
            texture_handles: TextureHandles::default(),
        }
    }

    /// The host queue, so the instance can read the rendered texture back.
    pub(crate) fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    /// Resolve document layout and paint directly into `target`.
    pub fn paint(
        &mut self,
        document: &mut BaseDocument,
        scrollbars: &[ScrollbarRegion],
        input_selections: &[InputSelection],
        input_carets: &[InputCaret],
        number_spinners: &[NumberSpinner],
        theme_override: Option<ScrollbarColors>,
        scale: f64,
        target: &wgpu::Texture,
    ) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        // Build the Vello scene from the resolved document + overlays.
        self.scene.reset();
        {
            let mut scene =
                VelloScenePainter::new(&mut self.vello, &mut self.texture_handles, &mut self.scene);
            blitz_paint::paint_scene(&mut scene, document, scale, self.width, self.height, 0, 0);
            paint_input_selections(&mut scene, input_selections, scale);
            paint_input_carets(&mut scene, input_carets, scale);
            crate::scrollbar::paint_scrollbars(
                &mut scene,
                document,
                scrollbars,
                scale,
                theme_override,
            );
            crate::spinner::paint_number_spinners(&mut scene, number_spinners, scale);
        }

        // Rasterize straight into the host texture on the host device. No CPU
        // readback, no re-upload. `target` must be `Rgba8Unorm` with
        // `STORAGE_BINDING` (see `Instance` texture allocation).
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        self.vello
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.scene,
                &view,
                &vello::RenderParams {
                    base_color: peniko::Color::TRANSPARENT,
                    width: self.width,
                    height: self.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .expect("vello render_to_texture");

        // Free the scene's retained geometry until the next frame.
        self.scene.reset();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // Vello is resolution-independent: the target texture is reallocated by
        // the caller and the size is passed via `RenderParams` each frame.
        self.width = width;
        self.height = height;
    }
}

/// Vello's recommended shader-init threading: single-threaded on macOS (a known
/// driver workaround), all cores elsewhere. Mirrors `anyrender_vello`.
fn vello_init_threads() -> Option<std::num::NonZeroUsize> {
    #[cfg(target_os = "macos")]
    {
        std::num::NonZeroUsize::new(1)
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn paint_input_selections<S: PaintScene>(scene: &mut S, selections: &[InputSelection], scale: f64) {
    let color = peniko::Color::from_rgba8(180, 213, 255, 150);
    for selection in selections {
        let rect = Rect::new(
            selection.x as f64,
            selection.y as f64,
            (selection.x + selection.width) as f64,
            (selection.y + selection.height) as f64,
        );
        scene.fill(Fill::NonZero, Affine::scale(scale), color, None, &rect);
    }
}

fn paint_input_carets<S: PaintScene>(scene: &mut S, carets: &[InputCaret], scale: f64) {
    for caret in carets {
        let rect = Rect::new(
            caret.x as f64,
            caret.y as f64,
            (caret.x + caret.width) as f64,
            (caret.y + caret.height) as f64,
        );
        scene.fill(
            Fill::NonZero,
            Affine::scale(scale),
            caret.color,
            None,
            &rect,
        );
    }
}
