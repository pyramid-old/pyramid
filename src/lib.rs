#![feature(plugin, box_patterns, convert, vec_push_all, path_ext)]
#![plugin(peg_syntax_ext)]

extern crate xml;
extern crate time;

#[macro_use]
pub mod hashmap_macro;
pub mod document;
mod propnode_test;
pub mod propnode;
pub mod system;
pub mod interface;

pub mod propnode_parser {
    peg_file! propnode_peg("propnode.rustpeg");
    use propnode;
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct ParseError {
        pub line: usize,
        pub column: usize,
        pub offset: usize,
        pub expected: ::std::collections::HashSet<&'static str>,
    }
    pub fn parse(text: &str) -> Result<propnode::PropNode, ParseError> {
        match propnode_peg::body(text) {
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
