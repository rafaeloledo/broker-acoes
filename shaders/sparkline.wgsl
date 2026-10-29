struct SparklineUniforms {
    view_width: f32,
    view_height: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: SparklineUniforms;

struct GpuSparklinePoint {
    x_clip: f32,
    y_clip: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    _pad: vec3<f32>,
};

@group(0) @binding(1)
var<storage, read> points: array<GpuSparklinePoint>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let pt = points[vertex_index];
    out.clip_position = vec4<f32>(pt.x_clip, pt.y_clip, 0.0, 1.0);
    out.color = vec4<f32>(pt.color_r, pt.color_g, pt.color_b, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
