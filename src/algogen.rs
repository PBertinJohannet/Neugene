//! Module algogen Defines a trait for this.
use crate::graphics::{DrawInstruction, Entity, Frame, ManyStepDrawable, SingleStepDrawable};
use crate::problems;
use crate::problems::{GenericProblem, ManyStepProblem, SingleStepProblem, Solution};
use rand::XorShiftRng;
use rulinalg::vector::Vector;
/// Represents the starting size of the population in individuals.
const POP_START: f64 = 25.0;
/// Represent the mutation rate (0.1 = 10% modification in average).
const MUT_START: f64 = 0.5;
/// Represents how much of the best people do we keep alive without mutation.
const ELITE_KEEP_START: f64 = 1.0;
/// Represents how much of the worst people do we kill
const DEATH_START: f64 = 20.0;
/// All the non-killed people will have childs, this represents how much per couple.
/// If there are 10 couples and 1.1 childpercouple the best couple will have 2 child.
const CHILD_PER_COUPLE_START: f64 = 4.0;

/// If you change the struct, change its size please.
const PARAM_CHOICE_SIZE: usize = 6;
#[derive(Debug, Clone)]
pub struct ParamChoice {
    global: f64,
    mutrate: f64,
    elite: f64,
    kills: f64,
    birth_rate: f64,
}
impl ParamChoice {
    pub fn new() -> Self {
        ParamChoice {
            global: 1.0,
            mutrate: MUT_START,
            elite: ELITE_KEEP_START,
            kills: DEATH_START,
            birth_rate: CHILD_PER_COUPLE_START,
        }
    }
    /// Everything stays the same.
    pub fn same() -> Self {
        ParamChoice {
            global: 0.0,
            mutrate: 0.5,
            elite: 0.5,
            kills: 0.5,
            birth_rate: 0.5,
        }
    }
    /// Updates the generation's result with the new values chosen by the neural net.
    /// The updates are interpreted as following : with the global at 1 every 0.1 is 10% modification.
    /// 0.5 is no changes.
    /// 1.0 is +100%
    /// with the global at 2 every 0.1 would be 20%
    /// 1.0 would be +200%
    pub fn update(&mut self, other: Self) {
        self.global = other.global;
        self.mutrate += self.mutrate * (other.mutrate - 0.5) * other.global;
        self.elite += self.elite * (other.elite - 0.5) * other.global;
        self.kills += self.kills * (other.kills - 0.5) * other.global;
        self.birth_rate += self.birth_rate * (other.birth_rate - 0.5) * other.global;
    }
    /// Returns the genresult as a slice.
    pub fn from_vector(vec: Vector<f64>) -> Self {
        ParamChoice {
            global: vec[0],
            mutrate: vec[1],
            elite: vec[2],
            kills: vec[3],
            birth_rate: vec[4],
        }
    }
}

const GEN_RESULT_SIZE: usize = 15;
#[derive(Debug, Clone)]
pub struct GenResult {
    pub max: f64,
    pub min: f64,
    pub q1: f64,
    pub med: f64,
    pub q3: f64,
    pub max5: [f64; 5],
    pub med5: [f64; 5],
}

impl GenResult {
    pub fn new() -> Self {
        GenResult {
            max: 0.0,
            min: 0.0,
            q1: 0.0,
            med: 0.0,
            q3: 0.0,
            max5: [0.0; 5],
            med5: [0.0; 5],
        }
    }
    /// Updates the generation's result with the new values.
    pub fn update(&mut self, max: f64, min: f64, q1: f64, med: f64, q3: f64) {
        for i in 0..4 {
            self.max5[i] = self.max5[i + 1];
            self.med5[i] = self.med5[i + 1];
        }
        self.max5[4] = self.max;
        self.med5[4] = self.med;
        self.max = max;
        self.min = min;
        self.med = med;
        self.q1 = q1;
        self.q3 = q3;
    }
    /// Returns the genresult as a slice.
    /// a - min / (max - min)
    pub fn into_vector(self) -> Vector<f64> {
        let f = |a| match self.min == self.max {
            false => (a - self.min) / (self.max - self.min),
            true => 0.0,
        };
        let mut ret = vec![
            f(self.max),
            f(self.med),
            f(self.q1),
            f(self.q3),
            f(self.min),
        ];
        ret.append(&mut self.max5.to_vec().into_iter().map(f).collect());
        ret.append(&mut self.max5.to_vec().into_iter().map(f).collect());
        Vector::new(ret)
    }
}

/// Proceed as told, see the next_gen function.
#[derive(Debug, Clone)]
pub struct AlgoGen<P: SingleStepProblem> {
    random: XorShiftRng,
    pop: Vec<P::Sol>,
    params: ParamChoice,
    problem: P,
    last_res: GenResult,
    individuals_played: usize,
}

impl<P: SingleStepProblem> AlgoGen<P> {
    pub fn initiate(prob_conf: P::ProblemConfig, my_rand: &mut XorShiftRng) -> Self {
        let mut random = my_rand;
        let prob = P::random(&mut random, &prob_conf);
        let pop = (0..POP_START as usize)
            .map(|_| P::Sol::random(&mut random, &prob.get_sol_conf()))
            .collect();
        AlgoGen {
            problem: prob,
            random: random.clone(),
            pop: pop,
            last_res: GenResult::new(),
            params: ParamChoice::new(),
            individuals_played: 0,
        }
    }

    ///
    /// The Algo must proceed like following :
    ///  At the start of next_gen all solutions are sorted by score.
    ///
    ///  - kill the required amount of the worst people.
    ///  - mutate the rest by keeping the elite unchanged.
    ///  - form couples between the remaining solutions (1&2, 3&4 etc...)
    ///  - Calculate how much childs must be created.
    ///  - Start with the best couple and make a child, continue and come back if necessary until all th
    ///  childs are created.
    ///
    ///  - evaluate all solutions.
    ///  - sort them by score.
    ///  Returns the statistics.
    ///
    pub fn next_gen(&mut self, choice_next: ParamChoice) -> &GenResult {
        self.apply_params(choice_next);
        self.kill_last();
        self.mutate_average();
        self.make_childs();
        self.sort_pop();
        self.update_res();
        &self.last_res
    }

    /// Apply the params and does a little bit of sanity checks :
    /// No pop > 200
    /// No pop < 2
    /// Params elite < pop+2
    /// It does so by modifying the birth rate and checking the kills/pop.
    pub fn apply_params(&mut self, choice: ParamChoice) {
        self.params.update(choice);
        if self.params.kills >= self.pop.len() as f64 - 2.0 {
            self.params.kills = self.pop.len() as f64 - 2.0;
        }
        if (self.pop.len() as f64 - self.params.kills) * self.params.birth_rate > 200.0 {
            self.params.birth_rate = 200.0 / (self.pop.len() as f64 - self.params.kills);
        }
        if (self.pop.len() as f64 - self.params.kills) * self.params.birth_rate < 4.0 {
            self.params.birth_rate = 4.0 / (self.pop.len() as f64 - self.params.kills);
        }
    }

    /// Updates the statistics.
    fn update_res(&mut self) {
        self.last_res.update(
            self.pop.first().unwrap().get_score(),
            self.pop.last().unwrap().get_score(),
            self.pop[self.pop.len() / 4].get_score(),
            self.pop[self.pop.len() / 2].get_score(),
            self.pop[3 * self.pop.len() / 4].get_score(),
        )
    }

    /// Evaluates the population and sorts it to get the worst individuals at the end.
    fn sort_pop(&mut self) {
        use ordered_float::OrderedFloat;
        self.pop.iter_mut().for_each(|s| s.reset_score());
        self.problem.add_scores_all(&mut self.pop);
        self.individuals_played += self.pop.len();
        self.pop.sort_by_key(|sol| OrderedFloat(-sol.get_score()))
    }

    /// Make childs for every couple.
    fn make_childs(&mut self) {
        let (mut cur_index, mut childs_done) = (0, 0);
        let mut childs = vec![];
        /*println!("params : {:?}", self.params);
        println!("childs to make : {}", (self.params.birth_rate * self.pop.len() as f64) as usize);
        println!("pop : {}", self.pop.len());*/
        while childs_done < (self.params.birth_rate * self.pop.len() as f64) as usize {
            if cur_index + 1 == self.pop.len() {
                cur_index = 0;
            }
            childs.push(self.pop[cur_index].child(&self.pop[cur_index + 1], &mut self.random));
            cur_index += 1;
            childs_done += 1;
        }
        self.pop.append(&mut childs);
    }

    /// Mutate the average performing individuals
    fn mutate_average(&mut self) {
        let nb_average = self.pop.len() - self.params.elite as usize;
        for i in nb_average..self.pop.len() {
            self.pop[i].mutate(self.params.mutrate, &mut self.random);
        }
    }

    /// Kill the required amount of bad performing individuals.
    fn kill_last(&mut self) {
        let to_keep = self.pop.len() - self.params.kills as usize;
        self.pop.truncate(to_keep);
    }

    /// Returns the best performing individual in the population.
    pub fn best(&self) -> &P::Sol {
        use ordered_float::OrderedFloat;
        self.pop
            .iter()
            .max_by_key(|sol| OrderedFloat(sol.get_score()))
            .unwrap()
    }

    pub fn demonstrate(&self) {
        self.problem.demonstrate(self.pop.first().unwrap());
    }
}

impl<T: SingleStepProblem + Clone> GenericProblem for AlgoGen<T> {
    type ProblemConfig = T::ProblemConfig;

    fn random(xsr: &mut XorShiftRng, prob_conf: &<Self as GenericProblem>::ProblemConfig) -> Self {
        AlgoGen::<T>::initiate(prob_conf.clone(), xsr)
    }

    fn print_state(&self) {
        println!(
            "best : {}\tmin : {}\t pop : {}\n",
            self.last_res.max,
            self.last_res.min,
            self.pop.len()
        );
    }
}

/// This is the twist !!
impl<T: SingleStepProblem + Clone> ManyStepProblem for AlgoGen<T>
where
    <T as SingleStepProblem>::Sol: Clone,
{
    fn get_state(&self) -> Vector<f64> {
        self.last_res.clone().into_vector()
    }

    fn make_step(&mut self, choice: &Vector<f64>) {
        self.next_gen(ParamChoice::from_vector(choice.clone()));
    }

    fn max_step(&self) -> Option<usize> {
        Some(20)
    }

    fn evaluate(&self) -> f64 {
        //println!("{} in {} gives : {}", self.last_res.max, self.individuals_played, self.last_res.max.powf(2.0)/self.individuals_played as f64);
        self.last_res.max.powf(2.0) / self.individuals_played as f64
    }

    fn input_space(&self) -> usize {
        PARAM_CHOICE_SIZE
    }

    fn output_space(&self) -> usize {
        GEN_RESULT_SIZE
    }

    fn is_solved(&self) -> bool {
        self.individuals_played > 150
    }
}

impl<T: SingleStepDrawable + Clone> ManyStepDrawable for AlgoGen<T>
where
    <T as problems::SingleStepProblem>::Sol: std::clone::Clone,
{
    fn get_frames(&self) -> Vec<DrawInstruction> {
        let mut a = vec![];
        a.append(&mut self.problem.get_frames(self.pop.first().unwrap()));
        a.append(&mut self.problem.get_frames(self.pop.last().unwrap()));
        a
    }
}
