use std::time::Instant;

pub struct GpuTelemetry {
    pub last_frame_time: Instant,
    pub frame_count: u64,
    pub fps: f64,
    pub frame_time_ms: f64,
    pub draw_calls: u32,
    pub vertices_drawn: u64,
}

impl Default for GpuTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuTelemetry {
    pub fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
            frame_count: 0,
            fps: 60.0,
            frame_time_ms: 16.6,
            draw_calls: 0,
            vertices_drawn: 0,
        }
    }

    pub fn on_frame(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_frame_time.elapsed().as_secs_f64();
        if elapsed > 0.0001 {
            let current_fps = 1.0 / elapsed;
            // Smooth moving average
            self.fps = self.fps * 0.9 + current_fps * 0.1;
            self.frame_time_ms = elapsed * 1000.0;
        }
        self.last_frame_time = Instant::now();
    }
}
