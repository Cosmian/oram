mod btree;
mod client;
mod oram;
mod oram_tests;

use std::io::Error;

use crate::{
    client::ClientORAM,
    oram::{AccessType, ORAM},
};

use rand::{Rng, RngCore};
fn main() -> Result<(), Error> {
    println!("Hello, Path-ORAM!");

    /*
     * Example of use for 15 items stored and a ciphertext size of 16 bytes.
     */
    let nb_blocks: usize = 15;
    let nb_leaves = (nb_blocks + 1) / 2;
    let block_size: usize = 16;

    /*
     * Client.
     */
    let mut client = ClientORAM::new();
    client.gen_key();

    let dummies_result = client.generate_dummies(nb_blocks, block_size);

    assert!(dummies_result.is_ok());
    let mut dummies = dummies_result.unwrap();

    /*
     * Server.
     */
    let mut path_oram = ORAM::new(dummies.as_mut(), nb_blocks).unwrap();

    // Let's read path 3, of all 16 paths from 0 to 15 included.
    let path = 3;

    let path_values_result =
        path_oram.access(AccessType::Read, path, Option::None);

    assert!(path_values_result.is_ok());
    let path_values_option = path_values_result.unwrap();

    assert!(path_values_option.is_some());
    let mut path_values = path_values_option.unwrap();

    /*
     * Client side now.
     * After decryption, change block number 6.
     */
    client.change_block(&mut path_values, [6].to_vec(), block_size, nb_leaves);

    /*
     * Each Read operation **must** be followed by a write operation on the same
     * path. Client sends new DataItems to write to the tree alongside with his
     * current stash.
     * Server side.
     */
    let path_values_remnants_res = path_oram.access(
        AccessType::Write,
        path,
        Some(&mut [path_values, client.stash].concat()),
    );

    assert!(path_values_remnants_res.is_ok());
    let path_values_remnants_opt = path_values_remnants_res.unwrap();

    assert!(path_values_remnants_opt.is_some());
    let path_values_remnants = path_values_remnants_opt.unwrap();

    /*
     * Server sends back remnants items it could not load into the tree. They
     * constitute the new stash.
     */

    client.stash = path_values_remnants;

    Ok(())
}
