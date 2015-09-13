#![feature(plugin, box_patterns, convert, vec_push_all, slice_concat_ext, cell_extras, core_intrinsics)]
#![plugin(peg_syntax_ext)]

extern crate xml;
extern crate cgmath;

#[macro_use]
pub mod hashmap_macro;
pub mod document;
#[cfg(test)]
mod pon_test;
#[macro_use]
pub mod pon;
pub mod pon_translations;
pub mod system;
pub mod interface;
pub mod pon_to_cgmath;
