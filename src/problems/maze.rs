//! Just a problem where we need to find our way in a maze.
//! The aim is to find a serie of 25 movements that goes the farthest to the right.
//!
//! Level : Easy
use rand::{XorShiftRng, Rng};
use crate::problems::{Solution, SingleStepProblem, GenericProblem, GenericSol};
use rulinalg::vector::Vector;

const SOL_SIZE : f64 = 2.0;

/// The starting position is between 0 and 10, the starting speed is between -5 and 5
#[derive(Clone, Debug)]
pub struct MazeProblem{
    maze_size : usize,
    maze : Vec<Vec<bool>>,
    end : Vec<f64>,
}

impl MazeProblem {
    fn validate_pos(&self, pos : usize) -> usize{
        if pos > self.maze_size-1{
            return self.maze_size-1;
        } else {
            return pos
        }
    }

    fn clamp(mv : f64) -> f64 {
        if mv > 1.0{
            return 1.0;
        } else if mv < -1.0 {
            return -1.0;
        } else {
            return mv;
        }
    }

    /// The maze is created with the following algorithm :
    /// Start at the begining and check for possible 1.
    /// Explore using random DFS
    /// Explore only not already explored areas.
    /// When exploring an area, checks that it has only a maximum of 1 neibourgh explored
    /// A possible move must not
    /// Returns the end of the maze.
    fn create_maze(maze : &mut Vec<Vec<bool>>, xsr: &mut XorShiftRng) -> Vec<usize>{
        let current_pos = vec![0usize, 0usize];
        let mut queue = vec![current_pos.clone()];
        let mut farthest = (0, vec![0,0]);
        let mut end = current_pos;
        while let Some(next) = queue.pop(){
            end = next.clone();
            if Self::is_explorable(maze, &next){
                maze[next[0]][next[1]] = true;
                if next[0]+next[1]>farthest.0{
                    farthest = (next[0]+next[1], next.clone());
                }
                let mut possibles = Self::get_1_explore(maze, next);
                xsr.shuffle(&mut possibles);
                for other in possibles{
                    queue.push(other);
                }
            }
        }
        maze[end[0]][end[1]] = true;
        end
    }

    /// Returns the explorable positions when creating the labyrinth.
    ///
    fn get_1_explore(mut maze : &mut Vec<Vec<bool>>, pos : Vec<usize>) -> Vec<Vec<usize>>{
        let possibles = vec![vec![pos[0], pos[1]+1], vec![pos[0], pos[1]-1], vec![pos[0]-1, pos[1]], vec![pos[0]+1, pos[1]]];
        let res: Vec<Vec<usize>>  = possibles.into_iter()
            .filter(|target|Self::is_explorable(&mut maze, target)).collect();
        res

    }
    /// Returns the explorable positions when creating the labyrinth.
    ///
    fn get_nb_adjacing_explored(maze : &mut Vec<Vec<bool>>, pos : Vec<usize>) -> usize{
        let possibles = vec![vec![pos[0], pos[1]+1], vec![pos[0], pos[1]-1], vec![pos[0]-1, pos[1]], vec![pos[0]+1, pos[1]]];
        let mut total = 0;
        for p in possibles{
            if p[0]<maze.len()-1 && p[1]<maze.len() && maze[p[0]][p[1]]{
                total +=1;
            }
        }
        total
    }

    /// Checks if the position was explored or not.
    fn is_explorable(maze : &mut Vec<Vec<bool>>, pos : &Vec<usize>) -> bool{
        pos[0]<maze.len() && pos[1]<maze.len() && !maze[pos[0]][pos[1]] && {
            let res = Self::get_nb_adjacing_explored(maze, pos.clone()) < 2;
            res
        }
    }

    fn play(&self, sol : &<Self as SingleStepProblem>::Sol, verbose : bool) -> f64 {
        let mut pos = vec![0.0, 0.0];
        let mut mv = vec![0.0, 0.0];
        for i in 0..(self.maze_size as f64*SOL_SIZE) as usize {
            if verbose {
                self.print_pos(&pos);
            }
            if verbose {
                println!("mv {:?}", mv);
            }
            mv[0]+=Self::clamp(sol.1[i]);
            mv[1]+=Self::clamp(sol.1[i*2]);
            mv[0]*=0.8;
            mv[1]*=0.8;
            // get future block
            let pos_future = vec![pos[0]+mv[0], pos[1]+mv[1]];
            let (block_x, block_y) = (self.validate_pos(pos_future[0] as usize),
                                      self.validate_pos(pos_future[1]as usize));
            if self.maze[block_x][block_y]{
                if verbose {
                    println!("from {:?} to {:?}", pos, pos_future);
                }
                pos = pos_future;
            }
        }
        self.pathfind(pos)
    }

    fn pathfind(&self, pos : Vec<f64>) -> f64{
        self.maze_size as f64*1.4 -
            (Vector::new(pos)-
                Vector::new(self.end.clone())).iter().map(|a|a.abs()).sum::<f64>()
    }

    fn print_pos(&self, pos : &Vec<f64>) {
        println!("\n----");
        for x in 0..self.maze_size{
            for y in 0..self.maze_size{
                if pos[0] as usize == x && pos[1] as usize == y {
                    print!("xx");
                } else if self.end[0] as usize == x && self.end[1] as usize == y{
                    print!("TT");
                } if self.maze[x][y]{
                    print!("  ");
                }else {
                    print!("##");
                }
            }
            println!("");
        }
        println!("----\n");
    }

}

impl GenericProblem for MazeProblem{
    type ProblemConfig = usize;

    fn random(xsr: &mut XorShiftRng, conf : &usize) -> Self {
        let mut maze : Vec<Vec<bool>> = (0..*conf).map(|_|(0..*conf).map(|_|false).collect()).collect();
        let end = Self::create_maze(&mut maze, xsr);
        maze[0][0] = true;
        MazeProblem {
            maze : maze,
            maze_size : *conf,
            end : vec![end[0] as f64, end[1] as f64],
        }
    }

    fn print_state(&self) {
        println!("oker");
    }

}

impl SingleStepProblem for MazeProblem {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        (self.maze_size as f64*SOL_SIZE*2.0)as usize
    }

    /// Plays a full game.
    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        self.play(sol, false)
    }

    fn demonstrate(&self, sol: &<Self as SingleStepProblem>::Sol) {
        self.play(sol, true);
    }
}