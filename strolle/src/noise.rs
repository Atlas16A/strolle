use std::io::Cursor;

use blue_noise_sampler::spp2 as bn;
use image::io::Reader as ImageReader;

use crate::{gpu, Bindable, MappedStorageBuffer, Texture};

#[derive(Debug)]
pub struct Noise {
    blue_noise: Texture,
    blue_noise_sobol: MappedStorageBuffer<Vec<i32>>,
    blue_noise_scrambling_tile: MappedStorageBuffer<Vec<i32>>,
    blue_noise_ranking_tile: MappedStorageBuffer<Vec<i32>>,
    flushed: bool,
}

impl Noise {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            blue_noise: Texture::builder("blue_noise")
                .with_size(gpu::BlueNoise::SIZE)
                .with_format(wgpu::TextureFormat::Rgba8Unorm)
                .with_usage(wgpu::TextureUsages::COPY_DST)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .build(device),
            blue_noise_sobol: MappedStorageBuffer::new(
                device,
                "blue_noise_sobol",
                bn::SOBOL.to_vec(),
            ),
            blue_noise_scrambling_tile: MappedStorageBuffer::new(
                device,
                "blue_noise_scrambling_tile",
                bn::SCRAMBLING_TILE.to_vec(),
            ),
            blue_noise_ranking_tile: MappedStorageBuffer::new(
                device,
                "blue_noise_ranking_tile",
                bn::RANKING_TILE.to_vec(),
            ),
            flushed: false,
        }
    }

    pub fn bind_blue_noise(&self) -> impl Bindable + '_ {
        self.blue_noise.bind_readable()
    }

    pub fn bind_blue_noise_sobol(&self) -> impl Bindable + '_ {
        self.blue_noise_sobol.bind_readable()
    }

    pub fn bind_blue_noise_scrambling_tile(&self) -> impl Bindable + '_ {
        self.blue_noise_scrambling_tile.bind_readable()
    }

    pub fn bind_blue_noise_ranking_tile(&self) -> impl Bindable + '_ {
        self.blue_noise_ranking_tile.bind_readable()
    }

    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.flushed {
            return;
        }

        let bytes = include_bytes!("../assets/blue-noise.png");

        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        let img = img.as_rgba8().unwrap().as_raw();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: self.blue_noise.tex(),
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
        );

        // ---

        _ = self.blue_noise_sobol.flush(device, queue);
        _ = self.blue_noise_scrambling_tile.flush(device, queue);
        _ = self.blue_noise_ranking_tile.flush(device, queue);

        // ---

        self.flushed = true;
    }
}
