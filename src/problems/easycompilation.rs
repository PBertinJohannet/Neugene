//! Compilation of all easy problems.
//! Takes a problem randomly and acts as it.

use super::{
    easyproblem::EasyProblem, lineareq::LinearEquationProblem, maze::MazeProblem,
    turnaround::TurnAroundProblem, walljump::WallJumpProblem,
};
use crate::problems::{GenericProblem, GenericSol, SingleStepProblem, Solution};
use rand::{Rng, XorShiftRng};

/// The simplest possible problem
/// minimise the difference between a serie of numbers and maximise the product.
#[derive(Clone, Debug)]
pub enum AllProblemsCompilation {
    Maze(Box<MazeProblem>),
    LinEq(Box<LinearEquationProblem>),
    WallJump(Box<WallJumpProblem>),
    Easy(Box<EasyProblem>),
    Turn(Box<TurnAroundProblem>),
}

impl GenericProblem for AllProblemsCompilation {
    type ProblemConfig = usize;

    fn random(xsr: &mut XorShiftRng, conf: &usize) -> Self {
        match xsr.gen_range(0, 4) {
            0 => AllProblemsCompilation::Maze(Box::new(MazeProblem::random(xsr, conf))),
            1 => AllProblemsCompilation::WallJump(Box::new(WallJumpProblem::random(xsr, conf))),
            2 => AllProblemsCompilation::Easy(Box::new(EasyProblem::random(xsr, conf))),
            3 => AllProblemsCompilation::LinEq(Box::new(LinearEquationProblem::random(xsr, conf))),
            _ => AllProblemsCompilation::Turn(Box::new(TurnAroundProblem::random(xsr, conf))),
        }
    }

    fn print_state(&self) {
        match self {
            AllProblemsCompilation::Maze(box p) => p.print_state(),
            AllProblemsCompilation::WallJump(box p) => p.print_state(),
            AllProblemsCompilation::LinEq(box p) => p.print_state(),
            AllProblemsCompilation::Easy(box p) => p.print_state(),
            AllProblemsCompilation::Turn(box p) => p.print_state(),
        }
    }
}

impl SingleStepProblem for AllProblemsCompilation {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        match self {
            AllProblemsCompilation::Maze(box p) => p.get_sol_conf(),
            AllProblemsCompilation::WallJump(box p) => p.get_sol_conf(),
            AllProblemsCompilation::LinEq(box p) => p.get_sol_conf(),
            AllProblemsCompilation::Easy(box p) => p.get_sol_conf(),
            AllProblemsCompilation::Turn(box p) => p.get_sol_conf(),
        }
    }

    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        match self {
            AllProblemsCompilation::Maze(box p) => p.evaluate(sol),
            AllProblemsCompilation::WallJump(box p) => p.evaluate(sol),
            AllProblemsCompilation::LinEq(box p) => p.evaluate(sol),
            AllProblemsCompilation::Easy(box p) => p.evaluate(sol),
            AllProblemsCompilation::Turn(box p) => p.evaluate(sol),
        }
    }

    fn demonstrate(&self, sol: &Self::Sol) {
        match self {
            AllProblemsCompilation::Maze(box p) => p.demonstrate(sol),
            AllProblemsCompilation::LinEq(box p) => p.demonstrate(sol),
            AllProblemsCompilation::WallJump(box p) => p.demonstrate(sol),
            AllProblemsCompilation::Easy(box p) => p.demonstrate(sol),
            AllProblemsCompilation::Turn(box p) => p.demonstrate(sol),
        }
    }
}
