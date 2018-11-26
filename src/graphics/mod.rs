pub mod app;
use crate::algogen::AlgoGen;
use crate::params::*;
use crate::problems;
use crate::problems::ManyStepProblem;
use crate::problems::SingleStepProblem;
use crate::reilearn::{LearnParams, ReiLearn};
use lmsmw::network::Network;
use rand::prelude::thread_rng;
use std::sync::{Arc, Mutex};

pub struct ToDraw(Vec<DrawInstruction>);

#[derive(Debug, Clone)]
pub enum DrawInstruction {
    Frame(Vec<Entity>),
    WorldSize([usize; 2]),
}

pub type Frame = Vec<Entity>;
pub type Entity = [f64; 7];

pub trait SingleStepDrawable: SingleStepProblem {
    fn get_frames(&self, sol: &Self::Sol) -> Vec<DrawInstruction>;
}

pub trait ManyStepDrawable: ManyStepProblem {
    fn get_frames(&self) -> Vec<DrawInstruction>;
}

/// Creates a network, making it learn to supervise genetic algorithms and print its score on a
/// set of examples.
pub fn learn_back<T: SingleStepDrawable + Clone>(
    next: Arc<Mutex<ToDraw>>,
    conf: Arc<Mutex<T::ProblemConfig>>,
) where
    <T as problems::SingleStepProblem>::Sol: std::clone::Clone,
{
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
    let mut rl = ReiLearn::<AlgoGen<T>>::new(net, conf.lock().unwrap().clone(), learn_params);
    loop {
        next.lock().unwrap().0 = rl.get_frames();
        println!(
            "score on test data with network : {}",
            rl.run_on_test_example() / (TEST_DATA_SIZE as f64)
        );
        rl.next_gen();
        // oh god yes
        // thank you !!!
    }
}
