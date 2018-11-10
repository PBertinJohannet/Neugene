//! Just a problem where a ship must turn around
//! Given an initial position and speed, find a way to come back to the original position
//! Scores are calculated in the following way :
//! -1.0 for every unit of distance at the end
//!
//! Level : Very Easy
use crate::problems::{GenericProblem, GenericSol, SingleStepProblem, Solution};
use rand::{distributions::Standard, Rng, XorShiftRng};
use rulinalg::vector::Vector;
use crate::graphics::SingleStepDrawable;
use crate::graphics::DrawInstruction;
use crate::graphics::Entity;

pub const ARENA_SIZE : f64 = 100.0;
pub const POD_SIZE : f64 = ARENA_SIZE/10.0;

enum RunOption {
    Verbose,
    None,
    Draw,
}

/// The starting position is between 0 and 10, the starting speed is between -5 and 5
#[derive(Clone, Debug)]
pub struct TurnAroundProblem {
    initial_pos: Vector<f64>,
    initial_speed: Vector<f64>,
}

impl TurnAroundProblem {
    fn play(&self, sol: &<Self as SingleStepProblem>::Sol, opt: RunOption,
            frames: &mut Vec<DrawInstruction>,) -> f64 {
        let mut current_pos = self.initial_pos.clone();
        let mut current_speed = self.initial_speed.clone();
        for i in 0..14 {
            match opt {
                RunOption::Draw => frames.push(self.get_frame(&current_pos)),
                RunOption::Verbose => self.print_state(),
                RunOption::None => (),
            }
            current_pos += &current_speed;
            current_speed[0] += sol.1[i] - 0.5;
            current_speed[1] += sol.1[2 * i] - 0.5;
        }
        1000.0
            - (current_pos - &self.initial_pos)
                .iter()
                .map(|p| p.abs())
                .sum::<f64>()
    }

    fn get_frame(&self, pos : &Vector<f64>) -> DrawInstruction{
        DrawInstruction::Frame(vec![[pos[0]+ARENA_SIZE, pos[1]+ARENA_SIZE, POD_SIZE, POD_SIZE, 0.0, 1.0, 0.0],
                                    [self.initial_pos[0]+ARENA_SIZE, self.initial_pos[1]+ARENA_SIZE, POD_SIZE, POD_SIZE, 1.0, 0.0, 0.0]])
    }
}

impl GenericProblem for TurnAroundProblem {
    type ProblemConfig = usize;

    fn random(xsr: &mut XorShiftRng, _conf: &usize) -> Self {
        TurnAroundProblem {
            initial_pos: xsr
                .sample_iter(&Standard)
                .map(|f: f64| f * 10.0)
                .take(2)
                .collect(),
            initial_speed: xsr
                .sample_iter(&Standard)
                .map(|f: f64| f * 5.0 - 5.0)
                .take(2)
                .collect(),
        }
    }


    fn print_state(&self) {
        println!("{:?} ; {:?}", self.initial_speed, self.initial_pos);
    }
}


impl SingleStepDrawable for TurnAroundProblem {
    fn get_frames(&self, sol: &<Self as SingleStepProblem>::Sol) -> Vec<DrawInstruction> {
        let mut ret = Vec::new();
        ret.push(DrawInstruction::WorldSize([ARENA_SIZE as usize*2, ARENA_SIZE as usize*2]));
        self.play(sol, RunOption::Draw, &mut ret);
        ret
    }
}


impl SingleStepProblem for TurnAroundProblem {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        28 * 2
    }

    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        self.play(sol, RunOption::None, &mut vec![])
    }

    fn demonstrate(&self, sol: &<Self as SingleStepProblem>::Sol) {
        self.play(sol, RunOption::Verbose, &mut vec![]);
    }
}
