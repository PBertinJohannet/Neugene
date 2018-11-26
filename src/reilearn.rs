//! The reilearn module
use crate::graphics::{DrawInstruction, ManyStepDrawable};
use crate::params::{MAX_GENETIC_ALG_GEN, MAX_ITER, TEST_DATA_SIZE};
use crate::problems::ManyStepProblem;
use lmsmw::Test;
/// Contains methods to apply reinforcment learning to lmsmw.
/// does not work like usual reinforcment learning due to the fact that it is not choosing
/// in a discrete set of options.
///
use lmsmw::{network::Network, ExamplesConfig, Learner};
use rand::{
    prelude::{thread_rng, ThreadRng},
    Rng,
};
use rulinalg::vector::Vector;
/// Represents a choice made by the neural network in a given situation.
pub struct Choice {
    inputs: Vector<f64>,
    #[allow(dead_code)]
    outputs: Vector<f64>,
    choice: Vector<f64>,
}

impl Choice {
    pub fn new(inputs: Vector<f64>, outputs: Vector<f64>, choice: Vector<f64>) -> Self {
        Choice {
            inputs: inputs,
            outputs: outputs,
            choice: choice,
        }
    }
    /// Reinforce the good moves.
    /// If the move was good, tell the network to keep making these choices
    pub fn into_good_test(self) -> Test {
        Test {
            inputs: self.inputs,
            outputs: self.choice,
        }
    }
    /// Don't reinforce the good moves.
    /// If the move was bad, tell the network to do the oppposite next time
    pub fn into_bad_test(self) -> Test {
        Test {
            inputs: self.inputs,
            outputs: self.choice.iter().map(|c| 1.0 - c).collect(),
        }
    }
}

/// The different parameters for learning
pub struct LearnParams {
    nb_problems: usize,
    test_per_prob: usize,
    max_gen: usize,
    starting_coef: f64,
    coef_mod: f64,
    percent_elite: f64,
}

impl LearnParams {
    pub fn new(
        nb_problems: usize,
        test_per_prob: usize,
        max_gen: usize,
        starting_coef: f64,
        coef_mod: f64,
        percent_elite: f64,
    ) -> Self {
        LearnParams {
            nb_problems: nb_problems,
            test_per_prob: test_per_prob,
            max_gen: max_gen,
            starting_coef: starting_coef,
            coef_mod: coef_mod,
            percent_elite: percent_elite,
        }
    }
}

/// Contains the tests problems
pub struct ReiLearn<P: ManyStepProblem> {
    test_problems: Vec<P>,
    net: Network,
    random: ThreadRng,
    params: LearnParams,
    problem_confs: P::ProblemConfig,
    pub coef: f64,
}

impl<P: ManyStepProblem> ReiLearn<P> {
    pub fn new(net: Network, prob_conf: P::ProblemConfig, learn_param: LearnParams) -> Self {
        let mut my_rand = thread_rng();
        ReiLearn {
            coef: learn_param.starting_coef,
            test_problems: (0..TEST_DATA_SIZE)
                .map(|_| P::random(&mut my_rand, &prob_conf))
                .collect(),
            net: (net),
            random: (my_rand),
            params: learn_param,
            problem_confs: prob_conf,
        }
    }
    #[allow(dead_code)]
    pub fn get_net(&self) -> &Network {
        &self.net
    }

    pub fn get_test_problems(&self) -> &Vec<P> {
        &self.test_problems
    }

    #[allow(dead_code)]
    pub fn demonstrate(&self) {
        for p in self.test_problems.iter() {
            let mut prob = p.clone();
            for _ in 0..p.max_step().unwrap_or(MAX_GENETIC_ALG_GEN) {
                prob.print_state();
                let inputs = prob.get_state();
                prob.make_step(&self.net.feed_forward(&inputs));
                if prob.is_solved() {
                    break;
                }
            }
            prob.print_state();
            println!("demo \n")
        }
    }

    pub fn demonstrate_on(&self, p: P) {
        let mut prob = p.clone();
        for _ in 0..p.max_step().unwrap_or(MAX_GENETIC_ALG_GEN) {
            prob.print_state();
            let inputs = prob.get_state();
            prob.make_step(&self.net.feed_forward(&inputs));
            if prob.is_solved() {
                break;
            }
        }
        prob.print_state();
        println!("demo \n")
    }

    pub fn run_on_test_example(&self) -> f64 {
        let mut score = 0.0;
        for p in self.test_problems.iter() {
            let mut prob = p.clone();
            for _ in 0..p.max_step().unwrap_or(MAX_GENETIC_ALG_GEN) {
                let inputs = prob.get_state();
                prob.make_step(&self.net.feed_forward(&inputs));
                if prob.is_solved() {
                    break;
                }
            }
            score += prob.evaluate();
        }
        score
    }

    /// Reinforce the inner network.
    pub fn reinforce(&mut self, tests: &Vec<Test>) {
        let layers = self.net.layers();
        let mut my_tests = tests.clone();
        self.random.shuffle(&mut my_tests);
        let net = Learner::new(ExamplesConfig::Ready(my_tests.clone()), layers)
            .set_net(self.net.clone())
            .gradient_descent_iters(20)
            .gradient_descent_step(0.01)
            .gradient_descent_nb_batches(tests.len() / 200)
            .lvbm_nb_batches(tests.len() / 50)
            .aim_score(0.05) // try to go until 0.05 score
            .max_iter(self.params.max_gen)
            .lvbm_max_iters(MAX_ITER)
            .start();
        self.net = net;
    }

    /// Performs a next iteration of testing -> reinforcing the network.
    pub fn next_gen(&mut self) {
        let mut tests = vec![];
        for _ in 0..self.params.nb_problems {
            let prob = P::random(&mut self.random, &self.problem_confs);
            tests.append(&mut self.gen_tests_for_prob(prob));
        }
        println!("training on {} examples", &tests.len());
        self.reinforce(&tests);
        self.coef *= self.params.coef_mod;
    }

    /// Tests the network several times on the problem and returns the reinforcment directives for
    /// this problem.
    pub fn gen_tests_for_prob(&mut self, prob: P) -> (Vec<Test>) {
        let mut results_prob = vec![];
        for _ in 0..self.params.test_per_prob {
            let cloned = prob.clone();
            results_prob.push(self.play_problem(cloned));
        }
        self.gen_tests_from_choices(results_prob)
    }

    /// Takes a list of playouts done on a problem.
    /// Changes the choices of the best playouts to reinforcment tests.
    pub fn gen_tests_from_choices(&mut self, mut games: Vec<(f64, Vec<Choice>)>) -> Vec<Test> {
        use ordered_float::OrderedFloat;
        games.sort_by_key(|g| OrderedFloat(g.0));
        let len = games.len();
        let lower = (len as f64 * self.params.percent_elite) as usize;
        let upper = (len as f64 * (1.0 - self.params.percent_elite)) as usize;
        games
            .into_iter()
            .enumerate()
            .map(|(index, (_, choices))| {
                if index < lower {
                    choices
                        .into_iter()
                        .map(|c| c.into_bad_test())
                        .collect::<Vec<Test>>()
                } else if index > upper {
                    choices
                        .into_iter()
                        .map(|c| c.into_good_test())
                        .collect::<Vec<Test>>()
                } else {
                    vec![]
                }
            })
            .flatten()
            .collect()
    }

    /// Asks the network to play the game.
    /// Returns the choices made and the score obtained.
    pub fn play_problem(&mut self, problem: P) -> (f64, Vec<Choice>) {
        let mut prob = problem;
        let mut choices = vec![];
        for _ in 0..prob.max_step().unwrap_or(MAX_GENETIC_ALG_GEN) {
            choices.push(self.make_choice(&mut prob));
            if prob.is_solved() {
                break;
            }
        }
        (prob.evaluate(), choices)
    }

    /// Make a choice given a problem.
    pub fn make_choice(&mut self, prob: &mut P) -> Choice {
        let inputs = prob.get_state();
        let outputs = self.net.feed_forward(&inputs);
        let choice = self.modify_outputs(&outputs);
        prob.make_step(&choice);
        Choice::new(inputs, outputs, choice)
    }

    pub fn modify_outputs(&mut self, res: &Vector<f64>) -> Vector<f64> {
        res.iter()
            .map(|val| {
                let ret = val + (self.random.gen::<f64>() - 0.5) * self.coef;
                if ret > 1.0 {
                    1.0
                } else if ret < 0.0 {
                    0.0
                } else {
                    ret
                }
            })
            .collect()
    }
}

impl<P: ManyStepDrawable> ReiLearn<P> {
    pub fn get_frames(&self) -> Vec<DrawInstruction> {
        let mut frames = Vec::new();
        let p = self.test_problems.first().unwrap();
        let mut prob = p.clone();
        for _ in 0..p.max_step().unwrap_or(MAX_GENETIC_ALG_GEN) {
            let inputs = prob.get_state();
            prob.make_step(&self.net.feed_forward(&inputs));
            if prob.is_solved() {
                break;
            }
            prob.print_state();
            println!("append frames : ");
            frames.append(&mut prob.get_frames());
        }
        println!("done\n");
        frames
    }
}
