use rand::Rng;

use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct ReferenceTracingPass {
    pass: CameraComputePass<gpu::ReferencePassParams>,
}

impl ReferenceTracingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("reference_tracing")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas_sampled(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.reference_rays.bind_readable(),
                &buffers.reference_hits.bind_writable(),
            ])
            .build(device, &engine.shaders.reference_tracing);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        depth: u8,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        let params = gpu::ReferencePassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
            depth: depth as u32,
        };

        self.pass.run(camera, encoder, size, &params);
    }
}
