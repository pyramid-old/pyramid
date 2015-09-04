#![feature(plugin, box_patterns, convert, vec_push_all, path_ext)]
#![plugin(peg_syntax_ext)]

extern crate xml;
extern crate time;

#[macro_use]
pub mod hashmap_macro;
pub mod document;
mod pon_test;
#[macro_use]
pub mod pon;
pub mod system;
pub mod interface;

pub mod pon_parser {
    peg_file! pon_peg("pon.rustpeg");
    use pon;
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct ParseError {
        pub line: usize,
        pub column: usize,
        pub offset: usize,
        pub expected: ::std::collections::HashSet<&'static str>,
    }
    pub fn parse(text: &str) -> Result<pon::Pon, ParseError> {
        match pon_peg::body(text) {
            Ok(node) => Ok(node),
            Err(err) => Err(ParseError {
                line: err.line,
                column: err.column,
                offset: err.offset,
                expected: err.expected
            })
        }
    }
}
