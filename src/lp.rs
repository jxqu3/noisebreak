use std::f32::consts::PI;

// Define the low-pass filter structure
pub struct LowPassFilter {
    // Coefficients for the filter
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    // Past input samples
    past_inputs: [f32; 2],
    // Past output samples
    past_outputs: [f32; 2],

    sample_rate: f32,
}

impl LowPassFilter {
    // Create a new low-pass filter
    pub fn new(cutoff_frequency: f32, sampling_frequency: f32, quality_factor: f32) -> Self {
        let mut filter = LowPassFilter {
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            past_inputs: [0.0, 0.0],
            past_outputs: [0.0, 0.0],
            sample_rate: sampling_frequency,
        };
        filter.set_cutoff_frequency(cutoff_frequency, quality_factor);
        filter
    }

    pub fn set_sample_rate(&mut self, sampling_frequency: f32) {
        self.sample_rate = sampling_frequency;
    }


    // Update the cutoff frequency
    pub fn set_cutoff_frequency(&mut self, cutoff_frequency: f32, quality_factor: f32) {
        // Calculate necessary values
        let omega = 2.0 * PI * cutoff_frequency / self.sample_rate;
        let alpha = omega.sin() / (2.0 * quality_factor);
        let cos_omega = omega.cos();
        let a0 = 1.0 + alpha;

        // Calculate and store filter coefficients
        self.b0 = (1.0 - cos_omega) / (2.0 * a0);
        self.b1 = (1.0 - cos_omega) / a0;
        self.b2 = self.b0;
        self.a1 = -2.0 * cos_omega / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    // Process a new sample and return the filtered output
    pub fn process(&mut self, input_sample: f32) -> f32 {
        // Calculate the new output sample using the filter coefficients and past samples
        let new_output = self.b0 * input_sample
            + self.b1 * self.past_inputs[0]
            + self.b2 * self.past_inputs[1]
            - self.a1 * self.past_outputs[0]
            - self.a2 * self.past_outputs[1];

        // Update past samples for next time
        self.past_inputs[1] = self.past_inputs[0];
        self.past_inputs[0] = input_sample;
        self.past_outputs[1] = self.past_outputs[0];
        self.past_outputs[0] = new_output;

        // Return the new output sample
        new_output
    }
}
