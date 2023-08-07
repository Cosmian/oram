use crate::oram::{AccessType, Stash, ORAM};

mod btree;
mod oram;

fn main() {
    let nb_blocks = 128;
    let block_size = 64;
    let stash = Stash::new();
    let mut path_oram = ORAM::new(stash, nb_blocks, block_size);
    println!("Hello, Path-ORAM!");

    let path = 49;
    let path_values = path_oram.access(AccessType::Read, path, Option::None);
    println!("{:?}", path_values);
}
