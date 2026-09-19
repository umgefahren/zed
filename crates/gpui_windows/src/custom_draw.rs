//! Application-supplied Direct3D 11 passes.
//!
//! See [`gpui::PaintCustom`] for why this seam exists, what the contract is, and
//! when reaching for it is warranted at all.

use gpui::{Bounds, ContentMask, DevicePixels, ScaledPixels, Size};
use std::{any::Any, sync::Arc};
use windows::Win32::Graphics::{
    Direct3D11::{ID3D11Device, ID3D11DeviceContext},
    Dxgi::Common::DXGI_FORMAT,
};

/// What a custom Direct3D pass is handed when the renderer reaches it in the
/// scene.
pub struct DirectXDrawCx<'a> {
    /// The device GPUI's renderer owns. Create shaders, buffers and textures
    /// from this; there is no other route to it.
    pub device: &'a ID3D11Device,
    /// The immediate context GPUI is recording the frame into, with the main
    /// render target and viewport already bound.
    ///
    /// Unlike Metal and WGPU, Direct3D 11 has no command encoder object: this
    /// context *is* the state machine, and anything set on it persists. See
    /// [`DirectXCustomDraw::draw`] for which state is safe to leave behind.
    pub device_context: &'a ID3D11DeviceContext,
    /// Format of the render target a pipeline must be built against.
    pub format: DXGI_FORMAT,
    /// Sample count of the render target a pipeline must be built against.
    ///
    /// GPUI's main target is single-sampled. Only the offscreen path
    /// intermediate is 4x MSAA, and paths are resolved out of it and composited
    /// back as sprites. So analytic coverage in the pixel shader — rather than
    /// multisampling — is the way to antialias here.
    pub sample_count: u32,
    /// Size of the swap chain, in device pixels.
    pub viewport_size: Size<DevicePixels>,
    /// The region the pass is expected to draw within.
    pub bounds: Bounds<ScaledPixels>,
    /// The clip region in force, which the pass has to honour itself.
    pub content_mask: ContentMask<ScaledPixels>,
}

/// A GPU pass an application encodes itself, through GPUI's DirectX renderer.
pub trait DirectXCustomDraw: 'static {
    /// Encode the pass. Called once per frame the pass appears in, on the render
    /// thread, with GPUI's main render target bound.
    ///
    /// Build shaders, buffers and textures once and cache them in `self` behind
    /// interior mutability rather than rebuilding them per frame; this is also
    /// the only place `cx.device` is reachable, so first-call initialisation is
    /// the intended pattern.
    ///
    /// # What may be left behind
    ///
    /// GPUI rebinds shader resource views, primitive topology, both shaders and
    /// the blend state before every one of its own batches, so changing those is
    /// free. The renderer restores the render target, the viewport and the
    /// rasterizer state after this returns, because GPUI sets those once per
    /// frame or once per device rather than per batch.
    ///
    /// Everything else on the context is *not* restored — notably depth-stencil
    /// state, constant buffers and sampler bindings. GPUI does not currently set
    /// them, so a pass that changes them and does not put them back may break
    /// later batches if GPUI starts relying on them.
    fn draw(&self, cx: DirectXDrawCx<'_>);
}

/// A [`DirectXCustomDraw`] packed for [`gpui::Window::paint_custom`].
///
/// The wrapper is needed because an `Arc<dyn Any>` payload can only be downcast
/// to a concrete type, never to another trait object.
pub struct DirectXDrawHandle(pub Arc<dyn DirectXCustomDraw>);

impl DirectXDrawHandle {
    /// Wrap a pass as a payload for [`gpui::Window::paint_custom`].
    pub fn new(draw: Arc<dyn DirectXCustomDraw>) -> Arc<dyn Any> {
        Arc::new(Self(draw))
    }
}
