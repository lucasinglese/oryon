use crate::traits::{Output, StreamingTransform};
use smallvec::smallvec;

/// Stub feature that adds 1.0 to its input. No warm-up.
pub struct AddOneStub {
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl AddOneStub {
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> Self {
        AddOneStub { inputs, outputs }
    }
}

impl StreamingTransform for AddOneStub {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }
    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }
    fn update(&mut self, state: &[Option<f64>]) -> Output {
        smallvec![state[0].map(|x| x + 1.0)]
    }
    fn reset(&mut self) {}
    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(AddOneStub::new(self.inputs.clone(), self.outputs.clone()))
    }
}

/// Stub feature with warm-up of 1: returns None on first bar, passthrough after.
pub struct WarmUpOneStub {
    inputs: Vec<String>,
    outputs: Vec<String>,
    seen_one: bool,
}

impl WarmUpOneStub {
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> Self {
        WarmUpOneStub {
            inputs,
            outputs,
            seen_one: false,
        }
    }
}

impl StreamingTransform for WarmUpOneStub {
    fn input_names(&self) -> Vec<String> {
        self.inputs.clone()
    }
    fn output_names(&self) -> Vec<String> {
        self.outputs.clone()
    }
    fn warm_up_period(&self) -> usize {
        1
    }
    fn update(&mut self, state: &[Option<f64>]) -> Output {
        if !self.seen_one {
            self.seen_one = true;
            smallvec![None]
        } else {
            smallvec![state[0]]
        }
    }
    fn reset(&mut self) {
        self.seen_one = false;
    }
    fn fresh(&self) -> Box<dyn StreamingTransform> {
        Box::new(WarmUpOneStub::new(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }
}
