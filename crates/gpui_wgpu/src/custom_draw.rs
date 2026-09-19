//! Application-supplied WGPU passes.
//!
//! See [`gpui::PaintCustom`] for why this seam exists, what the contract is, and
//! when reaching for it is warranted at all.

use gpui::{Bounds, ContentMask, DevicePixels, ScaledPixels, Size};
use std::{any::Any, sync::Arc};

/// What a custom WGPU pass is handed when the renderer reaches it in the scene.
pub struct WgpuDrawCx<'pass, 'encoder> {
    /// The device GPUI's renderer owns. Create pipelines, buffers and textures
    /// from this; there is no other route to it.
    pub device: &'pass wgpu::Device,
    /// The queue matching [`Self::device`], for uploads outside the pass.
    pub queue: &'pass wgpu::Queue,
    /// The live render pass GPUI is recording into. Encode draws into it
    /// directly.
    pub pass: &'pass mut wgpu::RenderPass<'encoder>,
    /// Colour target format a pipeline must be built against.
    pub format: wgpu::TextureFormat,
    /// Sample count a pipeline must be built against.
    ///
    /// GPUI's main pass is single-sampled. Only the offscreen path intermediate
    /// is multisampled, and paths are resolved out of it and composited back
    /// into this pass as sprites. A pipeline built for another sample count is
    /// rejected when it is bound, so analytic coverage in the fragment shader —
    /// rather than multisampling — is the way to antialias here.
    pub sample_count: u32,
    /// Size of the surface, in device pixels.
    pub viewport_size: Size<DevicePixels>,
    /// The region the pass is expected to draw within.
    pub bounds: Bounds<ScaledPixels>,
    /// The clip region in force, which the pass has to honour itself.
    pub content_mask: ContentMask<ScaledPixels>,
}

/// A GPU pass an application encodes itself, through GPUI's WGPU renderer.
pub trait WgpuCustomDraw: 'static {
    /// Encode the pass. Called once per frame the pass appears in, on the
    /// render thread, from inside GPUI's main render pass.
    ///
    /// Build pipeline state, buffers and textures once and cache them in `self`
    /// behind interior mutability rather than rebuilding them per frame; this is
    /// also the only place `cx.device` is reachable, so first-call
    /// initialisation is the intended pattern.
    fn draw(&self, cx: WgpuDrawCx<'_, '_>);
}

/// A [`WgpuCustomDraw`] packed for [`gpui::Window::paint_custom`].
///
/// The wrapper is needed because an `Arc<dyn Any>` payload can only be downcast
/// to a concrete type, never to another trait object.
pub struct WgpuDrawHandle(pub Arc<dyn WgpuCustomDraw>);

impl WgpuDrawHandle {
    /// Wrap a pass as a payload for [`gpui::Window::paint_custom`].
    pub fn new(draw: Arc<dyn WgpuCustomDraw>) -> Arc<dyn Any> {
        Arc::new(Self(draw))
    }
}
