#![feature(iterator_flatten)]
#![feature(box_patterns)]
//! Trains neural network to supervise genetic algorithms, the neural network will get informations
//! about how the learning is going and make a choice about how to modify parameters such as the
//! mutation rate, elitism, childs per survivors etc...
extern crate rand;
extern crate rulinalg;
extern crate ordered_float;
#[macro_use]
extern crate lmsmw;
pub mod algogen;
pub mod problems;
pub mod reilearn;
pub mod params;