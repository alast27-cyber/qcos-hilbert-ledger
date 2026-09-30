pub struct QuantizedStateVector {
    pub min_val: f32,
    pub scale: f32,
    pub data: Vec<u8>,
}

impl QuantizedStateVector {
    pub fn compress_gradients(gradients: &[f32]) -> Self {
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;

        for &val in gradients {
            if val < min_val { min_val = val; }
            if val > max_val { max_val = val; }
        }

        let range = (max_val - min_val).max(1e-8);
        let scale = range / 255.0;

        let data = gradients
            .iter()
            .map(|&v| ((v - min_val) / scale).round().clamp(0.0, 255.0) as u8)
            .collect();

        QuantizedStateVector { min_val, scale, data }
    }

    pub fn decompress(&self) -> Vec<f32> {
        self.data
            .iter()
            .map(|&b| self.min_val + (b as f32 * self.scale))
            .collect()
    }
}