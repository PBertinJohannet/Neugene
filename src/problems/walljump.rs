//! Just a problem where a little creature must use a wall to jump
//! The problem works as following :
//! Press the key Right or Up
//! right moves the player at the right of 20 unit
//! Up creates an upper movement of 25, the upper movement reduces by 5 every turn until the y pos
//! is 0.
//! Up does not work if the x position is not 0;
//! The aim is to get the maximum to the right
//! There will be a wall at some position with some height.
//! To move to the right of the wall one must be upper than the wall.
//!
//! The aim is to find a serie of 25 movements that goes the farthest to the right.
//!
//!
//! Level : Very Easy
use rand::{XorShiftRng, Rng};
use crate::problems::{Solution, SingleStepProblem, GenericProblem, GenericSol};


/// The starting position is between 0 and 10, the starting speed is between -5 and 5
#[derive(Clone, Debug)]
pub struct WallJumpProblem{
    wall_pos : f64,
    wall_height : f64,
}

impl WallJumpProblem {
    fn validate(x : f64)-> f64{
        if x > 2.0 {
            2.0
        } else if x < 0.0 {
            0.0
        } else {
            x
        }
    }

    fn play(&self, sol : &<Self as SingleStepProblem>::Sol, verbose : bool) -> f64 {
        let mut pos = vec![0.0, 0.0];
        let mut speed_up = 0.0;
        //println!("wall is {}m height at pos : {}", self.wall_height, self.wall_pos);
        let mut mv_r = 0.0;
        for i in 0..25{
            if verbose {
                self.print_pos(&pos);
            }
            mv_r+=Self::validate(sol.1[i])/2.0;
            mv_r*=0.9;
            let mv_up = sol.1[i*2];
            // check if we hit the wall
            if pos[0]<self.wall_pos && pos[0]+mv_r>self.wall_pos{
                if pos[1]>self.wall_height{
                    // We pass up
                    pos[0]+=mv_r;
                } else {
                    // We dont
                    pos[0] = self.wall_height-0.1;
                }
            } else {
                pos[0]+=mv_r;
            }
            // Plays up if we are at the ground
            if mv_up > 0.5 && pos[1]== 0.0{
                speed_up = 5.0;
            }
            // Move by the up speed
            pos[1]+=speed_up;
            // When we hit the ground, no vertical speed remains.
            if pos[1] <0.0{
                pos[1] = 0.0;
                speed_up = 0.0;
            }
            // If we are in the air, go down
            if pos[1] > 0.0{
                speed_up-=1.5;
            }
        }
        pos[0]
    }

    fn print_pos(&self, pos : &Vec<f64>) {
        for p_x in 0..11{
            let x = 10-p_x;
            for y in 0..50{
                if pos[0] as usize == y && pos[1] as usize == x{
                    print!("#");
                } else if self.wall_pos as usize == y && x<= self.wall_height as usize{
                    print!("|");
                } else {
                    print!(" ");
                }
            }
            println!("");
        }
        println!("wall at {}, height : {}, pos : {:?}", self.wall_pos, self.wall_height, pos);
    }

}

impl GenericProblem for WallJumpProblem{
    type ProblemConfig = usize;

    fn random(xsr: &mut XorShiftRng, _conf : &usize) -> Self {
        WallJumpProblem {
            wall_pos: 2.0+xsr.gen::<f64>()*10.0,
            wall_height: 5.0+xsr.gen::<f64>()*10.0,
        }
    }

    fn print_state(&self) {
        println!("wall at {}, height : {}", self.wall_pos, self.wall_height);
    }

}

impl SingleStepProblem for WallJumpProblem {
    type Sol = GenericSol;

    fn get_sol_conf(&self) -> <<Self as SingleStepProblem>::Sol as Solution>::SolConfig {
        50
    }

    /// Plays a full game.
    fn evaluate(&mut self, sol: &mut Self::Sol) -> f64 {
        self.play(sol, false)
    }

    fn demonstrate(&self, sol: &<Self as SingleStepProblem>::Sol) {
        self.play(sol, true);
    }
}