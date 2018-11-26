use crate::problems::{GenericProblem, ManyStepProblem};
use rand::{prelude::ThreadRng, Rng};
use rulinalg::vector::Vector;

/// One simple problem step by step.
/// Given a List of positions, visit every position in the minimum possible time.
/// The positions are given in the following form :
/// [ 01 01 01 11 01 01 01] first digit = me, second digit = visited.
#[derive(Clone, Debug)]
pub struct EasyStep {
    my_pos: usize,
    visited: Vec<f64>,
}

impl GenericProblem for EasyStep {
    type ProblemConfig = usize;

    fn random(xsr: &mut ThreadRng, conf: &usize) -> Self {
        EasyStep {
            visited: (0..*conf).map(|_| 1.0).collect(),
            my_pos: xsr.gen::<usize>() % *conf,
        }
    }

    fn print_state(&self) {
        for i in 0..self.visited.len() {
            if i == self.my_pos {
                print!(".#.")
            } else if self.visited[i] == 1.0 {
                print!(". .");
            } else {
                print!(".|.");
            }
        }
        print!("\n");
    }
}

impl ManyStepProblem for EasyStep {
    fn get_state(&self) -> Vector<f64> {
        Vector::new(
            self.visited
                .iter()
                .enumerate()
                .map(|(index, &val)| vec![(index == self.my_pos) as i32 as f64, val])
                .flatten()
                .collect::<Vec<f64>>(),
        )
    }
    fn make_step(&mut self, choice: &Vector<f64>) {
        if choice[0] > choice[1] {
            if self.my_pos < self.visited.len() - 1 {
                self.my_pos += 1;
            }
        } else if self.my_pos > 0 {
            self.my_pos -= 1;
        }
        self.visited[self.my_pos] = 0.0;
    }
    fn max_step(&self) -> Option<usize> {
        Some((self.visited.len() as f64 * 1.5) as usize)
    }
    fn evaluate(&self) -> f64 {
        return -self.visited.iter().sum::<f64>();
    }
    fn input_space(&self) -> usize {
        2
    }
    fn output_space(&self) -> usize {
        self.visited.len()
    }

    fn is_solved(&self) -> bool {
        self.evaluate() < 1.0
    }
}
