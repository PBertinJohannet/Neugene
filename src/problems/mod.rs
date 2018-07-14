//! The module for the problems.
pub mod easyproblem;
pub mod easystep;
pub mod turnaround;
pub mod walljump;
pub mod maze;
pub mod easycompilation;
pub mod lineareq;
use std::fmt::Debug;
use std::marker::Sized;
use rulinalg::vector::Vector;
use rand::{XorShiftRng, Rng, distributions::Standard, distributions::Distribution};

pub trait Solution : Debug + Sized{
    type SolConfig;
    fn random(xsr : &mut XorShiftRng, sol_conf : &Self::SolConfig) -> Self;
    fn add_score(&mut self, score : f64);
    fn reset_score(&mut self);
    fn get_score(&self) -> f64;
    fn from_vec(source : Vector<f64>) -> Self;
    fn as_mut_vec(&mut self) -> &mut Vector<f64>;
    fn as_vec(&self) -> &Vector<f64>;

    fn child(&self, other: &Self, xsr: &mut XorShiftRng) -> Self {
        Self::from_vec(self.as_vec().iter().zip(other.as_vec().iter()).map(|(&a, &b)| match xsr.gen() {
            true => a,
            _ => b,
        }).collect())
    }

    fn mutate(&mut self, mutrate: f64, xsr: &mut XorShiftRng) {
        for s in self.as_mut_vec().iter_mut(){
            *s *= 2.0*mutrate*(xsr.gen::<f64>()-0.5);
        }
    }
}


pub trait GenericProblem {
    /// The problem's configuration.
    type ProblemConfig : Clone;
    /// Creates a random problem.
    fn random(xsr : &mut XorShiftRng, prob_conf : &Self::ProblemConfig) -> Self;
    /// Prints the state in readable format.
    fn print_state(&self);
}

pub trait SingleStepProblem: Debug + Sized + GenericProblem{
    /// The type of solution that the problem accept.
    type Sol : Solution;
    /// Get the solution's conf
    fn get_sol_conf(&self) -> <Self::Sol as Solution>::SolConfig;
    /// Evaluates a solution's performance on the problem.
    fn evaluate(&mut self, sol : &mut Self::Sol) -> f64;
    /// Evaluates all the solutions on the problem.
    fn add_scores_all(&mut self, sol : &mut Vec<Self::Sol>){
        for s in sol {
            let score = self.evaluate(s);
            s.add_score(score);
        }
    }
    /// demonstrate the given solution in action.
    fn demonstrate(&self, sol : &Self::Sol);
}

/// A many step problem is the a problem but we make many choices when playing and we may have access
/// to its inner state between choices.
/// Same as a multi single step problem but it
pub trait ManyStepProblem : Debug + Sized + GenericProblem + Clone{
    fn get_state(&self) -> Vector<f64>;
    fn make_step(&mut self, choice : &Vector<f64>);
    fn max_step(&self) -> Option<usize>;
    fn evaluate(&self) -> f64;
    fn input_space(&self) -> usize;
    fn output_space(&self) -> usize;
    /// Tells if the problem is solved.
    fn is_solved(&self) -> bool;
}



type GenericSol = (f64, Vector<f64>);

impl Solution for GenericSol{
    type SolConfig = usize;

    fn random(xsr: &mut XorShiftRng, sol_conf: &<Self as Solution>::SolConfig) -> Self {
        (0.0, Vector::from(Standard.sample_iter(xsr).take(*sol_conf).collect::<Vec<f64>>()))
    }

    fn add_score(&mut self, score: f64) {
        self.0 += score;
    }

    fn reset_score(&mut self) {
        self.0 = 0.0;
    }

    fn get_score(&self) -> f64 {
        self.0
    }

    fn from_vec(source: Vector<f64>) -> Self {
        (0.0, source)
    }

    fn as_mut_vec(&mut self) -> &mut Vector<f64> {
        &mut self.1
    }

    fn as_vec(&self) -> &Vector<f64> {
        &self.1
    }
}