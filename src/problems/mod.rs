//! The module for the problems.
pub mod easycompilation;
pub mod easyproblem;
pub mod easystep;
pub mod follow;
pub mod lineareq;
pub mod maze;
pub mod turnaround;
pub mod walljump;
use rand::{distributions::Distribution, distributions::Standard, prelude::ThreadRng, Rng};
use rulinalg::vector::Vector;
use std::fmt::Debug;
use std::marker::Sized;

pub trait Solution: Debug + Sized {
    type SolConfig;
    fn random(xsr: &mut ThreadRng, sol_conf: &Self::SolConfig) -> Self;
    fn add_score(&mut self, score: f64);
    fn reset_score(&mut self);
    fn get_score(&self) -> f64;
    fn from_vec(source: Vector<f64>) -> Self;
    fn as_mut_vec(&mut self) -> &mut Vector<f64>;
    fn as_vec(&self) -> &Vector<f64>;

    fn child(&self, other: &Self, xsr: &mut ThreadRng) -> Self {
        Self::from_vec(
            self.as_vec()
                .iter()
                .zip(other.as_vec().iter())
                .map(|(&a, &b)| match xsr.gen() {
                    true => a,
                    _ => b,
                })
                .collect(),
        )
    }

    fn mutate(&mut self, mutrate: f64, xsr: &mut ThreadRng) {
        for s in self.as_mut_vec().iter_mut() {
            *s *= 2.0 * mutrate * (xsr.gen::<f64>() - 0.5);
        }
    }
}

pub trait GenericProblem : Sized{
    /// The problem's configuration.
    type ProblemConfig: Clone + Send;
    /// Creates a random problem.
    fn random(xsr: &mut ThreadRng, prob_conf: &Self::ProblemConfig) -> Self
        where
            Self: Sized;
    /// Prints the state in readable format.
    fn print_state(&self);
}

pub trait SingleStepProblem: Debug + Sized + GenericProblem {
    /// The type of solution that the problem accept.
    type Sol: Solution;
    /// Get the solution's conf
    fn get_sol_conf(&self) -> <Self::Sol as Solution>::SolConfig;
    /// Evaluates a solution's performance on the problem.
    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64;
    /// Evaluates all the solutions on the problem.
    fn add_scores_all(&mut self, sol: &mut Vec<Self::Sol>) {
        for s in sol {
            let score = self.evaluate(s);
            s.add_score(score);
        }
    }
    /// demonstrate the given solution in action.
    fn demonstrate(&self, sol: &Self::Sol);
}

/// A many step problem is the a problem but we make many choices when playing and we may have access
/// to its inner state between choices.
pub trait ManyStepProblem: Debug + Sized + GenericProblem + Clone {
    /// Get the current state of the problem
    fn get_state(&self) -> Vector<f64>;
    /// Make a choice to go to the next level
    fn make_step(&mut self, choice: &Vector<f64>);
    /// Returns the maximum number of steps (None if unknown).
    fn max_step(&self) -> Option<usize>;
    /// Evaluates the current score of the problem.
    fn evaluate(&self) -> f64;
    /// Returns the size of the vector needed at each step
    fn input_space(&self) -> usize;
    /// Returns the size of the vector describing the state of the problem.
    fn output_space(&self) -> usize;
    /// Tells if the problem arrived to an end.
    fn is_solved(&self) -> bool;
}

type GenericSol = (f64, Vector<f64>);

impl Solution for GenericSol {
    /// the length of the vector
    type SolConfig = usize;

    fn random(xsr: &mut ThreadRng, sol_conf: &<Self as Solution>::SolConfig) -> Self {
        (
            0.0,
            Vector::from(
                Standard
                    .sample_iter(xsr)
                    .take(*sol_conf)
                    .collect::<Vec<f64>>(),
            ),
        )
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


/// should have a superviser capable of telling what params to choose given the
/// superviser can be a constant superviser.
pub trait SingleStepProblemSolver: Debug {
    type Problem: SingleStepProblem;
    fn solve_prob(
        &mut self,
        prob: <Self as SingleStepProblemSolver>::Problem,
    ) -> <<Self as SingleStepProblemSolver>::Problem as SingleStepProblem>::Sol;
}

pub trait SupervisableSolver: SingleStepProblemSolver + Debug + Clone + Sized{
    type CreateParam;
    type StepParam;
    fn take_prob(&mut self, prob: Self::Problem);
    fn next_step(&mut self, input: Self::StepParam);
    fn best_sol(&self) -> <<Self as SingleStepProblemSolver>::Problem as SingleStepProblem>::Sol;
    fn get_state(&self) -> Vector<f64>;
    fn evaluate(&self) -> f64;
    fn random(xsr: &mut ThreadRng, prob_conf: &Self::CreateParam) -> Self
    where
    Self: Sized;
    fn input_space(&self) -> usize;
    fn output_space(&self) -> usize;
    fn is_solved(&self) -> bool;
}


impl<P, Create, Step, Solver> GenericProblem for Solver
where
    Create: Clone + Send,
    P: SingleStepProblem,
    Solver: SupervisableSolver<Problem = P, CreateParam = Create, StepParam = Step>,
{
    type ProblemConfig = Create;

    fn random(xsr: &mut ThreadRng, prob_conf: &<Self as GenericProblem>::ProblemConfig) -> Self {
        <Self as SupervisableSolver>::random(xsr, prob_conf)
    }

    fn print_state(&self) {
        println!("{:?}", self.best_sol())
    }
}

impl<P, Step, Create, Solver> ManyStepProblem for Solver
where
    Create: Clone + Send,
    P: SingleStepProblem,
    Step: Into<Vec<f64>> + From<Vec<f64>>,
    Solver: SupervisableSolver<Problem = P, StepParam = Step, CreateParam = Create>
{
    fn get_state(&self) -> Vector<f64> {
        <Self as SupervisableSolver>::get_state(&self).into()
    }

    fn make_step(&mut self, choice: &Vector<f64>) {
        self.next_step(<Self as SupervisableSolver>::StepParam::from(choice.data().clone()))
    }

    fn max_step(&self) -> Option<usize> {
        None
    }

    fn evaluate(&self) -> f64 {
        <Self as SupervisableSolver>::evaluate(self)
    }

    fn input_space(&self) -> usize {
        <Self as SupervisableSolver>::input_space(self)
    }

    fn output_space(&self) -> usize {
        <Self as SupervisableSolver>::output_space(self)
    }

    fn is_solved(&self) -> bool {
        <Self as SupervisableSolver>::is_solved(self)
    }
}

trait TwoPlayerGame: Debug + Sized + GenericProblem + Clone{
    fn get_state(&self) -> Vector<f64>;
    fn play_move(&mut self, choice: &Vector<f64>, player_one : bool);
    fn max_step(&self) -> Option<usize>;
    fn input_space(&self) -> usize;
    fn output_space(&self) -> usize;
    fn score(&self, player_one : bool) -> f64;
    /// Tells if the problem is solved.
    fn is_solved(&self) -> bool;
}

trait TwoPlayerAI: Debug + Sized + Clone{
    type Game: TwoPlayerGame;
    /// Init the game and choose who the ai is playing
    fn init(g: Self::Game, player_one : bool) -> Self;
    /// Given the state of the game, returns the move played.
    fn next_move(&mut self, g: &Self::Game) -> Vector<f64>;
}

/// A two player game with a fixed ai is a one player game.
/// The ai is always the second player
#[derive(Debug, Clone)]
struct PlayAgainst<A : TwoPlayerAI<Game=G>, G: TwoPlayerGame> {
    ai : A,
    game : G,
}

impl<A, G> GenericProblem for PlayAgainst<A, G>
    where A: TwoPlayerAI<Game=G>,
          G: TwoPlayerGame
{
    type ProblemConfig = G::ProblemConfig;

    fn random(xsr: &mut ThreadRng, prob_conf: & <Self as GenericProblem>::ProblemConfig) -> Self where
        Self: Sized {
        let game = G::random(xsr, prob_conf);
        PlayAgainst {
            ai : A::init(game.clone(), false),
            game
        }
    }

    fn print_state(&self) {
        self.game.print_state()
    }
}

/// Play all the moves of the game at once
/// The ai is always the second player
impl<A, G> ManyStepProblem for PlayAgainst<A, G>
    where A: TwoPlayerAI<Game=G>,
          G: TwoPlayerGame
{

    fn get_state(&self) -> Vector<f64> {
        self.game.get_state()
    }

    fn make_step(&mut self, choice: &Vector<f64>) {
        let ai_move = self.ai.next_move(&self.game);
        self.game.play_move(&ai_move, false);
        self.game.play_move(choice, true);
    }

    fn max_step(&self) -> Option<usize> {
        self.game.max_step()
    }

    fn evaluate(&self) -> f64 {
        self.game.score(true)
    }

    fn input_space(&self) -> usize {
        self.game.input_space()
    }

    fn output_space(&self) -> usize {
        self.game.output_space()
    }

    fn is_solved(&self) -> bool {
        self.game.is_solved()
    }
}

impl<P : ManyStepProblem<ProblemConfig=usize>> SingleStepProblem for P {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        self.input_space() * self.max_step().expect("cannot create a single step solution without knowing the number of steps")
    }

    fn evaluate(&mut self, sol: &mut <Self as SingleStepProblem>::Sol) -> f64 {
        let solution = sol.1.data();
        let mut i = 0;
        let step_size = self.input_space();
        while !self.is_solved(){
            let mv = &solution[step_size*i..(step_size+1)*i];
            self.make_step(&Vector::new(mv));
        }
        <Self as ManyStepProblem>::evaluate(self)
    }

    fn demonstrate(&self, sol: &<Self as SingleStepProblem>::Sol) {
        let mut this = self.clone();
        let solution = sol.1.data();
        let mut i = 0;
        let step_size = this.input_space();
        while !this.is_solved(){
            this.print_state();
            let mv = &solution[step_size*i..(step_size+1)*i];
            this.make_step(&Vector::new(mv));
        }
        this.print_state();
    }
}