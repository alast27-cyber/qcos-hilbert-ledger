use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HilbertMode {
    pub mode_index: usize,
    pub amplitude_re: f64,
    pub amplitude_im: f64,
}

pub struct HilbertSpaceLedger {
    pub active_modes: HashMap<usize, HilbertMode>,
}

impl HilbertSpaceLedger {
    /// Shifts all occupied modes to the next odd mode (|2n - 1> -> |2n + 1>)
    pub fn shift_modes(&mut self) -> usize {
        let mut shifted_modes = HashMap::new();
        let count = self.active_modes.len();

        for (current_index, mode) in self.active_modes.drain() {
            // Ensure odd parity is preserved across shifts (+2 shift)
            let new_index = current_index + 2;
            shifted_modes.insert(
                new_index,
                HilbertMode {
                    mode_index: new_index,
                    amplitude_re: mode.amplitude_re,
                    amplitude_im: mode.amplitude_im,
                },
            );
        }

        self.active_modes = shifted_modes;
        count
    }
}