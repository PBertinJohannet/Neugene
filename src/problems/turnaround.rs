//! Just a problem where a ship must turn around
//! Given an initial position and speed, find a way to come back to the original solution
//! Scores are calculated in the following way :
//! -1.0 for every unit of distance at the end
//!
//! Level : Very Easy
use rand::{XorShiftRng, Rng, distributions::Standard};
use problems::{Solution, SingleStepProblem, GenericProblem, GenericSol};
use rulinalg::vector::Vector;

/// The starting position is between 0 and 10, the starting speed is between -5 and 5
#[derive(Clone, Debug)]
pub struct TurnAroundProblem{
    initial_pos : Vector<f64>,
    initial_speed : Vector<f64>,
}

impl TurnAroundProblem {


    fn play(&self, sol : &<Self as SingleStepProblem>::Sol, verbose : bool) -> f64{
        let mut current_pos = self.initial_pos.clone();
        let mut current_speed = self.initial_speed.clone();
        for i in 0..14{
            if verbose{
                self.print_state();
            }
            current_pos+= &current_speed;
            current_speed[0]+= sol.1[i]-0.5;
            current_speed[1]+= sol.1[2*i]-0.5;
        }
        1000.0-(current_pos - &self.initial_pos).iter().map(|p|p.abs()).sum::<f64>()
    }
}

impl GenericProblem for TurnAroundProblem{
    type ProblemConfig = usize;

    fn random(xsr: &mut XorShiftRng, _conf : &usize) -> Self {
        TurnAroundProblem {
            initial_pos: xsr.sample_iter(&Standard).map(|f: f64| f * 10.0).take(2).collect(),
            initial_speed: xsr.sample_iter(&Standard).map(|f: f64| f * 5.0 - 5.0).take(2).collect(),
        }
    }

    fn print_state(&self) {
        println!("{:?} ; {:?}", self.initial_speed, self.initial_pos);
    }
}

impl SingleStepProblem for TurnAroundProblem {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        14*2
    }

    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        self.play(sol, false)
    }

    fn demonstrate(&self, sol: &<Self as SingleStepProblem>::Sol) {
        self.play(sol, true);
    }
}