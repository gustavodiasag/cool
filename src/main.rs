use std::{
    env::{self},
    fs,
};

use cool::ast::bindings::Tree;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = &args[1];

    let input = fs::read(file).unwrap();
    let tree = Tree::new(&input);

    let class = tree.get_root().child(0).unwrap();

    println!("{}", class.child_count());

    class.children().for_each(|c| println!("{:?}", c.kind()));
}
