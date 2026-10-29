use crate::candlestick::{CandlestickPipeline, ChartUniforms};
use crate::treemap::{GpuTreemapNode, TreemapPipeline, TreemapUniforms};
use b3_core::quote::GpuCandle;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct GpuChartCallback {
    pub pipeline: Arc<Mutex<CandlestickPipeline>>,
    pub candles: Vec<GpuCandle>,
    pub uniforms: ChartUniforms,
}

impl CallbackTrait for GpuChartCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut pipe = self.pipeline.lock();
        pipe.update_data(device, queue, &self.candles, self.uniforms);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &CallbackResources,
    ) {
        let rect = info.clip_rect_in_pixels();
        if rect.width_px > 0 && rect.height_px > 0 {
            render_pass.set_scissor_rect(
                rect.left_px.max(0) as u32,
                rect.top_px.max(0) as u32,
                rect.width_px as u32,
                rect.height_px as u32,
            );
            let pipe = self.pipeline.lock();
            let rpass: &mut wgpu::RenderPass<'_> = unsafe { std::mem::transmute(render_pass) };
            pipe.render(rpass);
        }
    }
}

pub struct GpuTreemapCallback {
    pub pipeline: Arc<Mutex<TreemapPipeline>>,
    pub nodes: Vec<GpuTreemapNode>,
    pub uniforms: TreemapUniforms,
}

impl CallbackTrait for GpuTreemapCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut pipe = self.pipeline.lock();
        pipe.update_data(device, queue, &self.nodes, self.uniforms);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &CallbackResources,
    ) {
        let rect = info.clip_rect_in_pixels();
        if rect.width_px > 0 && rect.height_px > 0 {
            render_pass.set_scissor_rect(
                rect.left_px.max(0) as u32,
                rect.top_px.max(0) as u32,
                rect.width_px as u32,
                rect.height_px as u32,
            );
            let pipe = self.pipeline.lock();
            let rpass: &mut wgpu::RenderPass<'_> = unsafe { std::mem::transmute(render_pass) };
            pipe.render(rpass);
        }
    }
}
