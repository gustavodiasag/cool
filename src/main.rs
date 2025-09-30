use std::{
    env::{self},
    fs,
};

use cool::{
    ast::{bindings::Tree, converter},
    util::interner::Interner,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = &args[1];

    let src = fs::read(file).unwrap();

    let mut interner = Interner::with_capacity(1_024);
    let tree = Tree::new(&src);

    match converter::convert(&src, &tree, &mut interner) {
        Ok(program) => println!("{:#?}", program),
        Err((program, errors)) => {
            println!("{:#?}", program);
            println!("{:#?}", errors);
        }
    }
}
