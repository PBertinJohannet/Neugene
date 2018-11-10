//! Just a problem where we need to solve a simple system of linear equations.
//!
//! Solve Ax=y
//! Level : Easy
use crate::problems::{GenericProblem, GenericSol, SingleStepProblem, Solution};
use rand::{distributions::Standard, Rng, XorShiftRng};
use rulinalg::{matrix::Matrix, vector::Vector};

const MAT_SIZE: usize = 50;

#[derive(Clone, Debug)]
pub struct LinearEquationProblem {
    matrix_a: Matrix<f64>,
    vector_y: Vector<f64>,
}

impl LinearEquationProblem {
    fn play(&self, sol: &<Self as SingleStepProblem>::Sol, verbose: bool) -> f64 {
        if verbose {
            println!(
                "expected : {}, got : {}",
                self.vector_y,
                &self.matrix_a * &sol.1
            )
        }
        (MAT_SIZE * MAT_SIZE) as f64
            - ((&self.matrix_a * &sol.1) - &self.vector_y)
                .into_iter()
                .map(f64::abs)
                .sum::<f64>()
    }
}

impl GenericProblem for LinearEquationProblem {
    type ProblemConfig = usize;

    fn random(xsr: &mut XorShiftRng, _conf: &usize) -> Self {
        LinearEquationProblem {
            matrix_a: Matrix::new(
                MAT_SIZE,
                MAT_SIZE,
                xsr.sample_iter(&Standard)
                    .take(MAT_SIZE * MAT_SIZE)
                    .collect::<Vec<f64>>(),
            ),
            vector_y: Vector::new(
                xsr.sample_iter(&Standard)
                    .take(MAT_SIZE)
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
        MAT_SIZE
    }

    /// Plays a full game.
    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        self.play(sol, false)
    }

    fn demonstrate(&self, sol: &<Self as SingleStepProblem>::Sol) {
        self.play(sol, true);
    }
}
