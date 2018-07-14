#![feature(iterator_flatten)]
#![feature(box_patterns)]
#![deny(missing_docs)]
//! Trains neural network to supervise genetic algorithms, the neural network will get informations
//! about how the learning is going and make a choice about how to modify parameters such as the
//! mutation rate, elitism, childs per survivors etc...
extern crate rand;
extern crate rulinalg;
extern crate ordered_float;
#[macro_use]
extern crate lmsmw;
mod algogen;
mod problems;
mod reilearn;
use problems::easycompilation::AllProblemsCompilation;
use problems::ManyStepProblem;
use problems::GenericProblem;
use rand::{XorShiftRng, FromEntropy};
use algogen::{AlgoGen, ParamChoice};
use reilearn::{ReiLearn, LearnParams};
use lmsmw::network::Network;

const MAX_ITER : usize= 25;
const TEST_DATA_SIZE : usize = 1000;
const NB_EXAMPLE_PROBLEMS : usize = 100;
const TESTS_PER_PROBLEM : usize = 50;
const MAX_GEN : usize = 2;
const STARTING_COEF : f64 = 1.0;
const COEF_MODIFICATOR : f64 = 0.95;
const PERCENT_ELITE : f64 = 0.05;
const PROB_CONF_SIZE : usize = 100;
const MAX_GENETIC_ALG_GEN : usize = 30;

type Problem = AllProblemsCompilation;

/// Run the algorithm
pub fn main() {
    gen_network();
}

/// Creates a network, making it learn to supervise genetic algorithms and print its score on a
/// set of examples.
pub fn gen_network() {
    let learn_params = LearnParams::new(NB_EXAMPLE_PROBLEMS,
                                            TESTS_PER_PROBLEM,
                                            MAX_GEN,
                                            STARTING_COEF,
                                            COEF_MODIFICATOR,
                                            PERCENT_ELITE);
    let mut random = XorShiftRng::from_entropy();
    let layers = layers![15, 40, 10, 6];
    let net = Network::new(layers, &mut random);
    let mut rl =ReiLearn::<AlgoGen<Problem>>::new(net,
                                                  PROB_CONF_SIZE,
                                                  learn_params);
    normal_test(rl.get_test_problems().clone());
    loop {
        println!("score on test data with network : {}", rl.run_on_test_example()/(TEST_DATA_SIZE as f64));
        rl.next_gen();
    }
}

/// Demonstrate how the network performs on a given problem.
pub fn demo_first(mut a : AlgoGen<Problem>, net : &Network){
    println!("demo for first ");
    for _ in 0..10{
        a.print_state();
        let inputs = a.get_state();
        a.make_step(&net.feed_forward(&inputs));
    }
    println!("for a total score of {}", a.evaluate());
}

/// Run a genetic algorithm with hand crafted parameters against the test problems.
pub fn normal_test(mut algs : Vec<AlgoGen<Problem>>){
    let mut score = 0.0;
    for a in algs.iter_mut() {
        for _ in 0..10{
            a.next_gen(ParamChoice::same());
        }
        score+=a.evaluate();
    }
    println!("total score on test data without network is : {}", score/(TEST_DATA_SIZE as f64));
}