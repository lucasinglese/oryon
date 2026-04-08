/// Generates a stateless binary operator struct that implements [`StreamingTransform`].
///
/// The generated struct holds named `inputs` (exactly 2) and `outputs` (non-empty).
/// `update()` applies `$op(a, b)` when both inputs are `Some`, otherwise returns `None`.
///
/// # Usage
/// ```ignore
/// binary_operator!(
///     Add,
///     "Element-wise addition: `A + B`.\n\nReturns `None` if either input is `None`.",
///     |a: f64, b: f64| Some(a + b),
/// );
/// ```
macro_rules! binary_operator {
    ($name:ident, $doc:expr, $op:expr $(,)?) => {
        #[doc = $doc]
        #[derive(Debug)]
        pub struct $name {
            inputs: Vec<String>,
            outputs: Vec<String>,
        }

        impl $name {
            /// Create a new instance.
            ///
            /// - `inputs` - names of the two input columns `[A, B]`. Must contain exactly 2 entries.
            /// - `outputs` - name of the output column. Must not be empty.
            pub fn new(
                inputs: Vec<String>,
                outputs: Vec<String>,
            ) -> Result<Self, crate::error::OryonError> {
                if inputs.len() != 2 {
                    return Err(crate::error::OryonError::InvalidInput {
                        msg: "inputs must contain exactly 2 columns".into(),
                    });
                }
                if outputs.is_empty() {
                    return Err(crate::error::OryonError::InvalidInput {
                        msg: "outputs must not be empty".into(),
                    });
                }
                Ok($name { inputs, outputs })
            }
        }

        impl crate::traits::StreamingTransform for $name {
            fn input_names(&self) -> Vec<String> {
                self.inputs.clone()
            }

            fn output_names(&self) -> Vec<String> {
                self.outputs.clone()
            }

            fn fresh(&self) -> Box<dyn crate::traits::StreamingTransform> {
                Box::new(
                    $name::new(self.inputs.clone(), self.outputs.clone())
                        .expect("fresh: config was already validated at construction"),
                )
            }

            fn reset(&mut self) {}

            fn update(&mut self, state: &[Option<f64>]) -> crate::traits::Output {
                let result = match (state[0], state[1]) {
                    (Some(a), Some(b)) => ($op)(a, b),
                    _ => None,
                };
                smallvec::smallvec![result]
            }
        }
    };
}

/// Generates a stateless unary operator struct that implements [`StreamingTransform`].
///
/// The generated struct holds named `inputs` (exactly 1) and `outputs` (non-empty).
/// `update()` applies `$op(x)` when the input is `Some`, otherwise returns `None`.
///
/// # Usage
/// ```ignore
/// unary_operator!(
///     Log,
///     "Natural logarithm: `ln(x)`.\n\nReturns `None` if the input is `None` or `<= 0`.",
///     |x: f64| if x > 0.0 { Some(x.ln()) } else { None },
/// );
/// ```
macro_rules! unary_operator {
    ($name:ident, $doc:expr, $op:expr $(,)?) => {
        #[doc = $doc]
        #[derive(Debug)]
        pub struct $name {
            inputs: Vec<String>,
            outputs: Vec<String>,
        }

        impl $name {
            /// Create a new instance.
            ///
            /// - `inputs` - name of the input column. Must contain exactly 1 entry.
            /// - `outputs` - name of the output column. Must not be empty.
            pub fn new(
                inputs: Vec<String>,
                outputs: Vec<String>,
            ) -> Result<Self, crate::error::OryonError> {
                if inputs.len() != 1 {
                    return Err(crate::error::OryonError::InvalidInput {
                        msg: "inputs must contain exactly 1 column".into(),
                    });
                }
                if outputs.is_empty() {
                    return Err(crate::error::OryonError::InvalidInput {
                        msg: "outputs must not be empty".into(),
                    });
                }
                Ok($name { inputs, outputs })
            }
        }

        impl crate::traits::StreamingTransform for $name {
            fn input_names(&self) -> Vec<String> {
                self.inputs.clone()
            }

            fn output_names(&self) -> Vec<String> {
                self.outputs.clone()
            }

            fn fresh(&self) -> Box<dyn crate::traits::StreamingTransform> {
                Box::new(
                    $name::new(self.inputs.clone(), self.outputs.clone())
                        .expect("fresh: config was already validated at construction"),
                )
            }

            fn reset(&mut self) {}

            fn update(&mut self, state: &[Option<f64>]) -> crate::traits::Output {
                let result = match state[0] {
                    Some(x) => ($op)(x),
                    None => None,
                };
                smallvec::smallvec![result]
            }
        }
    };
}
