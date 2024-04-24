use wgpu::TextureView;
use winit::window::Window;

pub struct WgpuFrame<'frame> {
    pub device: &'frame wgpu::Device,
    pub queue: &'frame wgpu::Queue,
    pub encoder: &'frame mut wgpu::CommandEncoder,
    pub window: &'frame Window,
    pub texture_view: &'frame TextureView,
    pub size: winit::dpi::PhysicalSize<u32>,
}
