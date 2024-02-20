#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_sprite::mesh2d_vertex_output::VertexOutput,

struct OutlineMaterial {
    color: vec4<f32>,
    thickness: f32,
}

@group(2) @binding(0)
var<uniform> input: OutlineMaterial;
@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_color_sampler: sampler;

fn get_sample(
    coords: vec2<f32>
) -> f32 {
    return textureSample(base_color_texture, base_color_sampler, coords).a;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    var outline: f32 = get_sample(uv + vec2<f32>(input.thickness, 0.0));

    outline += get_sample(uv + vec2<f32>(-input.thickness, 0.0));
    outline += get_sample(uv + vec2<f32>(0.0, input.thickness));
    outline += get_sample(uv + vec2<f32>(0.0, -input.thickness));
    outline += get_sample(uv + vec2<f32>(input.thickness, -input.thickness));
    outline += get_sample(uv + vec2<f32>(-input.thickness, input.thickness));
    outline += get_sample(uv + vec2<f32>(input.thickness, input.thickness));
    outline += get_sample(uv + vec2<f32>(-input.thickness, -input.thickness));
    outline = min(outline, 1.0);

    var color = textureSample(base_color_texture, base_color_sampler, uv);

    return mix(color, input.color, outline - color.a);
}