//! Just a problem where we need to solve a simple system of linear equations.
//!
//! Solve Ax=y
//! Level : Easy
use crate::problems::{GenericProblem, GenericSol, SingleStepProblem, Solution};
use rand::{distributions::Standard, prelude::ThreadRng, Rng};
use rulinalg::{
    matrix::{BaseMatrix, Matrix},
    vector::Vector,
};

#[derive(Clone, Debug)]
pub struct LinearEquationProblem {
    matrix_a: Matrix<f64>,
    vector_y: Vector<f64>,
}

impl LinearEquationProblem {
    fn play(&self, sol: &<Self as SingleStepProblem>::Sol, _: bool) -> f64 {
        (self.matrix_a.cols() * self.matrix_a.cols()) as f64
            - ((&self.matrix_a * &sol.1) - &self.vector_y)
                .into_iter()
                .map(f64::abs)
                .sum::<f64>()
    }
}

impl GenericProblem for LinearEquationProblem {
    type ProblemConfig = usize;

    fn random(xsr: &mut ThreadRng, _conf: &usize) -> Self {
        LinearEquationProblem {
            matrix_a: Matrix::new(
                *_conf,
                *_conf,
                xsr.sample_iter(&Standard)
                    .take(*_conf * *_conf)
                    .collect::<Vec<f64>>(),
            ),
            vector_y: Vector::new(
                xsr.sample_iter(&Standard)
                    .take(*_conf)
                    .collect::<Vec<f64>>(),
            ),
        }
    }

    fn print_state(&self) {
        println!("mat : {}, expected : {}", self.matrix_a, self.vector_y);
    }
}

impl SingleStepProblem for LinearEquationProblem {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        self.vector_y.size()
    }

    /// Plays a full game.
    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        self.play(sol, false)
    }

    fn demonstrate(&self, sol: &<Self as SingleStepProblem>::Sol) {
        self.play(sol, true);
    }
}
