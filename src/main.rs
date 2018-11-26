#![feature(box_patterns)]
//! Trains neural network to supervise genetic algorithms, the neural network will get informations
//! about how the learning is going and make a choice about how to modify parameters such as the
//! mutation rate, elitism, childs per survivors etc...
extern crate cairo;
extern crate gtk;
extern crate ordered_float;
extern crate rand;
extern crate rulinalg;
#[macro_use]
extern crate lmsmw;
mod algogen;
pub mod graphics;
mod params;
mod problems;
mod reilearn;

use self::graphics::app;
use crate::algogen::{AlgoGen, ParamChoice};
use crate::params::*;
use crate::problems::turnaround::TurnAroundProblem;
use crate::problems::GenericProblem;
use crate::problems::ManyStepProblem;
use crate::reilearn::{LearnParams, ReiLearn};
use lmsmw::network::Network;
use rand::prelude::thread_rng;
type Problem = TurnAroundProblem;

/// Run the algorithm
pub fn main() {
    match &mut app::App::<Problem>::new() {
        Ok(a) => a.start(PROB_CONF_SIZE),
        Err(_) => panic!("gtk::init failed"),
    }
}

/// Creates a network, making it learn to supervise genetic algorithms and print its score on a
/// set of examples.
pub fn gen_network() {
    let learn_params = LearnParams::new(
        NB_EXAMPLE_PROBLEMS,
        TESTS_PER_PROBLEM,
        MAX_GEN,
        STARTING_COEF,
        COEF_MODIFICATOR,
        PERCENT_ELITE,
    );
    let mut random = thread_rng();
    let layers = layers![15, 40, 10, 6];
    let net = Network::new(layers, &mut random);
    let mut rl = ReiLearn::<AlgoGen<Problem>>::new(net, PROB_CONF_SIZE, learn_params);
    normal_test(rl.get_test_problems().clone());
    let mut random = thread_rng();
    rl.demonstrate_on(AlgoGen::initiate(600, &mut random));
    loop {
        println!(
            "score on test data with network : {}",
            rl.run_on_test_example() / (1000_000.0 * TEST_DATA_SIZE as f64)
        );
        rl.next_gen();
        rl.demonstrate_on(AlgoGen::initiate(1000, &mut random));
    }
}

/// Demonstrate how the network performs on a given problem.
pub fn demo_first(mut a: AlgoGen<Problem>, net: &Network) {
    println!("demo for first ");
    for _ in 0..10 {
        a.print_state();
        let inputs = a.get_state();
        a.make_step(&net.feed_forward(&inputs));
    }
    println!("for a total score of {}", a.evaluate());
}

/// Run a genetic algorithm with hand crafted parameters against the test problems.
pub fn normal_test(mut algs: Vec<AlgoGen<Problem>>) {
    let mut score = 0.0;
    for a in algs.iter_mut() {
        for _ in 0..10 {
            a.next_gen(ParamChoice::same());
        }
        score += a.evaluate();
    }
    println!(
        "total score on test data without network is : {}",
        score / (1000_000.0 * TEST_DATA_SIZE as f64)
    );
}
