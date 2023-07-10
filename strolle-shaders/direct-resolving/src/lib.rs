#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    direct_primary_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)]
    direct_primary_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)]
    direct_secondary_rays: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4)]
    direct_secondary_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 5)]
    direct_secondary_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    direct_initial_samples: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 7)]
    raw_direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 8, storage_buffer)]
    direct_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        LightsView::new(lights),
        MaterialsView::new(materials),
        camera,
        direct_primary_hits_d0,
        direct_primary_hits_d1,
        direct_secondary_rays,
        direct_secondary_hits_d0,
        direct_secondary_hits_d1,
        direct_initial_samples,
        raw_direct_colors,
        direct_spatial_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    lights: LightsView,
    materials: MaterialsView,
    camera: &Camera,
    direct_primary_hits_d0: TexRgba32f,
    direct_primary_hits_d1: TexRgba32f,
    direct_secondary_rays: TexRgba32f,
    direct_secondary_hits_d0: TexRgba32f,
    direct_secondary_hits_d1: TexRgba32f,
    direct_initial_samples: &[Vec4],
    raw_direct_colors: TexRgba16f,
    direct_spatial_reservoirs: &[Vec4],
) {
    let screen_idx = camera.screen_to_idx(screen_pos);

    let (ray, hit) = Hit::find_direct(
        camera,
        direct_primary_hits_d0,
        direct_primary_hits_d1,
        direct_secondary_rays,
        direct_secondary_hits_d0,
        direct_secondary_hits_d1,
        screen_pos,
    );

    let out = if hit.is_some() {
        let reservoir = DirectReservoir::read(
            direct_spatial_reservoirs,
            camera.screen_to_idx(screen_pos),
        );

        if reservoir.w > 0.0 {
            let contribution = if reservoir.sample.is_sky() {
                reservoir.sample.light_contribution
            } else {
                let material = materials.get(hit.material_id);

                lights
                    .get(reservoir.sample.light_id)
                    .contribution(material, hit, ray)
                    .sum()
            };

            contribution * reservoir.w
        } else {
            Vec3::ZERO
        }
    } else {
        unsafe { direct_initial_samples.get_unchecked(screen_idx).xyz() }
    };

    unsafe {
        raw_direct_colors.write(screen_pos, out.extend(1.0));
    }
}
