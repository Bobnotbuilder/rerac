// Experimental RC2 static terrain diagnostic. No RAC1 VU/fog/GS defaults.
#import bevy_pbr::{forward_io::Vertex, mesh_functions, mesh_view_bindings::view}
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var tex_sampler: sampler;
struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(linear) color: vec4<f32>,
}
@vertex fn vertex(v: Vertex) -> Output {
    var out: Output;
    let world = mesh_functions::get_world_from_local(v.instance_index) * vec4<f32>(v.position, 1.0);
    out.position = view.clip_from_world * world;
    out.uv = v.uv;
    out.color = v.color;
    return out;
}
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}
@fragment fn fragment(v: Output) -> @location(0) vec4<f32> {
    // Explicit diagnostic policy: original base RGB/128 modulates raw mip0.
    // Alpha bytes are retained on CPU/GPU but ignored until live TEST/ALPHA
    // is correlated. Lighting/fog/GS mip selection are deliberately absent.
    let t = textureSampleLevel(tex, tex_sampler, v.uv, 0.0);
    let rgb = min(t.rgb * floor(v.color.rgb) / 128.0, vec3<f32>(1.0));
    return vec4<f32>(srgb_to_linear(rgb), 1.0);
}
