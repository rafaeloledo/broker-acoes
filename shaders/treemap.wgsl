struct TreemapUniforms {
    view_width: f32,
    view_height: f32,
    time_sec: f32,
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: TreemapUniforms;

struct GpuTreemapNode {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    change_pct: f32,
    flash_intensity: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(1)
var<storage, read> nodes: array<GpuTreemapNode>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) change_pct: f32,
    @location(2) flash_intensity: f32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let node = nodes[instance_index];

    // Quad vertices (0..5)
    var u = 0.0;
    var v = 0.0;
    if (vertex_index == 0u) { u = 0.0; v = 0.0; }
    else if (vertex_index == 1u) { u = 1.0; v = 0.0; }
    else if (vertex_index == 2u) { u = 1.0; v = 1.0; }
    else if (vertex_index == 3u) { u = 0.0; v = 0.0; }
    else if (vertex_index == 4u) { u = 1.0; v = 1.0; }
    else { u = 0.0; v = 1.0; }

    let px = node.x + u * node.w;
    let py = node.y + v * node.h;

    // Convert to clip space [-1, 1] (y inverted for graphics coords)
    let clip_x = px * 2.0 - 1.0;
    let clip_y = 1.0 - py * 2.0;

    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.uv = vec2<f32>(u, v);
    out.change_pct = node.change_pct;
    out.flash_intensity = node.flash_intensity;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Border check
    let border_w = 0.02;
    if (in.uv.x < border_w || in.uv.x > (1.0 - border_w) || in.uv.y < border_w || in.uv.y > (1.0 - border_w)) {
        return vec4<f32>(0.12, 0.13, 0.16, 1.0); // Dark border
    }

    // Color based on % change:
    // Strong Green: #007A3D (>= +3%)
    // Mild Green: #144929 (0% to +3%)
    // Neutral: #1E222D (0%)
    // Mild Red: #5A1E24 (-3% to 0%)
    // Strong Red: #9E1924 (<= -3%)
    var base_color: vec3<f32>;
    let pct = in.change_pct;

    if (pct > 0.0) {
        let t = clamp(pct / 4.0, 0.0, 1.0);
        base_color = mix(vec3<f32>(0.10, 0.22, 0.16), vec3<f32>(0.03, 0.60, 0.30), t);
    } else if (pct < 0.0) {
        let t = clamp(abs(pct) / 4.0, 0.0, 1.0);
        base_color = mix(vec3<f32>(0.28, 0.12, 0.14), vec3<f32>(0.75, 0.15, 0.20), t);
    } else {
        base_color = vec3<f32>(0.18, 0.20, 0.24);
    }

    // Add tick flash pulse (white / cyan highlight decaying)
    let flash = in.flash_intensity;
    let final_color = mix(base_color, vec3<f32>(1.0, 1.0, 1.0), flash * 0.45);

    return vec4<f32>(final_color, 1.0);
}
