use std::{
    env::{self},
    fs,
};

use cool::ast::converter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = &args[1];

    let input = fs::read(file).unwrap();

    let program = converter::convert(&input).unwrap();

    println!("{:#?}", program);
}
