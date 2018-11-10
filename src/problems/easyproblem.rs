//!
//! Very Easy

use crate::problems::{GenericProblem, GenericSol, SingleStepProblem, Solution};
use rand::{distributions::Standard, Rng, XorShiftRng};

/// The simplest possible problem
/// minimise the difference between a serie of numbers and maximise the product.
#[derive(Clone, Debug)]
pub struct EasyProblem {
    numbers: Vec<f64>,
}

impl GenericProblem for EasyProblem {
    type ProblemConfig = usize;

    fn random(xsr: &mut XorShiftRng, conf: &usize) -> Self {
        EasyProblem {
            numbers: xsr
                .sample_iter(&Standard)
                .map(|f: f64| f * 10.0)
                .take(*conf)
                .collect(),
        }
    }

    fn print_state(&self) {
        println!("{:?}", self.numbers);
    }
}

impl SingleStepProblem for EasyProblem {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        self.numbers.len()
    }

    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        let diff: f64 = self
            .numbers
            .iter()
            .zip(sol.1.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        let product = sol.1.iter().fold(0.0, |acc, x| acc * x);
        100.0 + product - diff
    }

    fn demonstrate(&self, sol: &Self::Sol) {
        let diff: f64 = self
            .numbers
            .iter()
            .zip(sol.1.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        let product = sol.1.iter().fold(0.0, |acc, x| acc * x);
        println!("diff : {}, product : {}", diff, product);
    }
}
