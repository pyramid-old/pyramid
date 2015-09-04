#![feature(plugin, box_patterns, convert, vec_push_all, path_ext)]
#![plugin(peg_syntax_ext)]

extern crate xml;
extern crate time;
extern crate cgmath;

#[macro_use]
pub mod hashmap_macro;
pub mod document;
#[cfg(test)]
mod pon_test;
#[macro_use]
pub mod pon;
pub mod system;
pub mod interface;
pub mod pon_to_cgmath;
