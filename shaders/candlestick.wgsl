struct ChartUniforms {
    view_width: f32,
    view_height: f32,
    min_price: f32,
    max_price: f32,
    start_idx: f32,
    end_idx: f32,
    candle_width: f32,
    wick_width: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: ChartUniforms;

struct GpuCandle {
    timestamp_sec: f32,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    volume: f32,
    is_up: f32,
    _pad: f32,
};

@group(0) @binding(1)
var<storage, read> candles: array<GpuCandle>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let candle_idx = u32(uniforms.start_idx) + instance_index;
    let candle = candles[candle_idx];

    let total_visible = max(uniforms.end_idx - uniforms.start_idx, 1.0);
    let x_norm = f32(instance_index) / total_visible; // 0.0 to 1.0
    let x_clip = (x_norm * 2.0 - 1.0); // -1.0 to 1.0

    let price_range = max(uniforms.max_price - uniforms.min_price, 0.001);
    let open_y = ((candle.open - uniforms.min_price) / price_range) * 2.0 - 1.0;
    let close_y = ((candle.close - uniforms.min_price) / price_range) * 2.0 - 1.0;
    let high_y = ((candle.high - uniforms.min_price) / price_range) * 2.0 - 1.0;
    let low_y = ((candle.low - uniforms.min_price) / price_range) * 2.0 - 1.0;

    let body_top = max(open_y, close_y);
    let body_bottom = min(open_y, close_y);

    let half_w = (uniforms.candle_width / uniforms.view_width);
    let wick_w = (uniforms.wick_width / uniforms.view_width);

    // Color: Green (#089981) or Red (#F23645)
    var col = vec4<f32>(0.949, 0.212, 0.271, 1.0); // Red
    if (candle.is_up > 0.5) {
        col = vec4<f32>(0.031, 0.600, 0.506, 1.0); // Green
    }

    // 12 vertices per instance:
    // 0..5: Wick quad (from low to high)
    // 6..11: Body quad (from body_bottom to body_top)
    var pos = vec2<f32>(0.0, 0.0);

    if (vertex_index < 6u) {
        // Wick quad
        let quad_v = vertex_index % 6u;
        if (quad_v == 0u) { pos = vec2<f32>(x_clip - wick_w, low_y); }
        else if (quad_v == 1u) { pos = vec2<f32>(x_clip + wick_w, low_y); }
        else if (quad_v == 2u) { pos = vec2<f32>(x_clip + wick_w, high_y); }
        else if (quad_v == 3u) { pos = vec2<f32>(x_clip - wick_w, low_y); }
        else if (quad_v == 4u) { pos = vec2<f32>(x_clip + wick_w, high_y); }
        else { pos = vec2<f32>(x_clip - wick_w, high_y); }
    } else {
        // Body quad
        let quad_v = (vertex_index - 6u) % 6u;
        let b_top = max(body_top, body_bottom + 0.003); // Ensure minimum 1px height
        if (quad_v == 0u) { pos = vec2<f32>(x_clip - half_w, body_bottom); }
        else if (quad_v == 1u) { pos = vec2<f32>(x_clip + half_w, body_bottom); }
        else if (quad_v == 2u) { pos = vec2<f32>(x_clip + half_w, b_top); }
        else if (quad_v == 3u) { pos = vec2<f32>(x_clip - half_w, body_bottom); }
        else if (quad_v == 4u) { pos = vec2<f32>(x_clip + half_w, b_top); }
        else { pos = vec2<f32>(x_clip - half_w, b_top); }
    }

    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.color = col;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
