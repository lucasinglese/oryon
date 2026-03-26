use crate::ops::average;
use crate::traits::{Feature, Output};
use smallvec::smallvec;
use std::collections::VecDeque;

pub struct Sma {
    window_size: usize,
    name: Vec<String>,
    col: Vec<String>,
    buffer: VecDeque<Option<f64>>,
}

impl Sma {
    pub fn new(window_size: usize, name: Vec<String>, col: Vec<String>) -> Self {
        Sma {
            window_size,
            name,
            col,
            buffer: VecDeque::with_capacity(window_size),
        }
    }
}

impl Feature for Sma {
    fn update(&mut self, state: &[Option<f64>]) -> Output {
        self.buffer.push_back(state[0]);

        if self.buffer.len() > self.window_size {
            self.buffer.pop_front();
        }

        if self.buffer.len() == self.window_size {
            let slices = self.buffer.make_contiguous();
            smallvec![average(slices)]
        } else {
            smallvec![None]
        }
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn warm_up_period(&self) -> usize {
        self.window_size - 1
    }

    fn names(&self) -> Vec<String> {
        self.name.clone()
    }

    fn required_columns(&self) -> Vec<String> {
        self.col.clone()
    }

    fn fresh(&self) -> Box<dyn Feature> {
        Box::new(Sma::new(
            self.window_size,
            self.name.clone(),
            self.col.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    fn out(v: Option<f64>) -> Output {
        smallvec![v]
    }

    #[test]
    fn test_update() {
        let mut sma = Sma::new(3, vec!["close_sma_3".into()], vec!["close".into()]);
        assert_eq!(sma.update(&[Some(1.0)]), out(None));
        assert_eq!(sma.update(&[Some(2.0)]), out(None));
        assert_eq!(sma.update(&[Some(3.0)]), out(Some(2.0)));
        assert_eq!(sma.update(&[Some(4.0)]), out(Some(3.0)));
        assert_eq!(sma.update(&[Some(5.0)]), out(Some(4.0)));
    }

    #[test]
    fn test_state_is_none() {
        let mut sma = Sma::new(3, vec!["close_sma_3".into()], vec!["close".into()]);
        assert_eq!(sma.update(&[Some(1.0)]), out(None));
        assert_eq!(sma.update(&[Some(2.0)]), out(None));
        assert_eq!(sma.update(&[Some(3.0)]), out(Some(2.0)));
        assert_eq!(sma.update(&[None]), out(None));
        assert_eq!(sma.update(&[Some(4.0)]), out(None));
        assert_eq!(sma.update(&[Some(5.0)]), out(None));
        assert_eq!(sma.update(&[Some(6.0)]), out(Some(5.0)));
    }

    #[test]
    fn test_window_size_is_one() {
        let mut sma = Sma::new(1, vec!["close_sma_1".into()], vec!["close".into()]);
        assert_eq!(sma.update(&[Some(1.0)]), out(Some(1.0)));
        assert_eq!(sma.update(&[Some(2.0)]), out(Some(2.0)));
    }

    #[test]
    fn test_reset() {
        let mut sma = Sma::new(3, vec!["close_sma_3".into()], vec!["close".into()]);
        sma.update(&[Some(1.0)]);
        assert_eq!(sma.buffer[0], Some(1.0));

        sma.reset();
        assert_eq!(sma.buffer.len(), 0);
        assert_eq!(sma.buffer.capacity(), sma.window_size);
    }

    #[test]
    fn test_warm_up_period() {
        let sma = Sma::new(3, vec!["close_sma_3".into()], vec!["close".into()]);
        assert_eq!(sma.warm_up_period(), 2);
    }

    #[test]
    fn test_names() {
        let sma = Sma::new(3, vec!["close_sma_3".into()], vec!["close".into()]);
        assert_eq!(sma.names(), vec!["close_sma_3".to_string()]);
    }

    #[test]
    fn test_fresh() {
        let mut sma = Sma::new(3, vec!["close_sma_3".into()], vec!["close".into()]);
        sma.update(&[Some(1.0)]);

        let mut fresh_sma = sma.fresh();
        assert_eq!(fresh_sma.names(), vec!["close_sma_3".to_string()]);
        assert_eq!(fresh_sma.warm_up_period(), 2);

        assert_eq!(fresh_sma.update(&[Some(1.0)]), out(None));
        assert_eq!(fresh_sma.update(&[Some(2.0)]), out(None));
        assert_eq!(fresh_sma.update(&[Some(3.0)]), out(Some(2.0)));

        assert_eq!(sma.update(&[Some(2.0)]), out(None));
        assert_eq!(sma.update(&[Some(3.0)]), out(Some(2.0)));
        assert_eq!(sma.update(&[Some(4.0)]), out(Some(3.0)));
        assert_eq!(sma.update(&[Some(5.0)]), out(Some(4.0)));
    }
}
