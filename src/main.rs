mod btree;
mod client;
mod oram;
mod oram_tests;

use crate::{
    client::ClientOram,
    oram::{AccessType, Oram, BUCKET_SIZE},
};
use std::io::{Error, ErrorKind};

fn main() -> Result<(), Error> {
    println!("Hello, Path-Oram!");

    /*
     * Implementation from https://eprint.iacr.org/2013/280.
     *
     * Example of use for 60 items (it fills 15 nodes which is the size of a
     * complete tree) and a ciphertext size of 16 bytes.
     */
    let nb_items: usize = 60;
    let ct_size: usize = 16;

    /*
     * Client.
     */
    let mut client = ClientOram::new(nb_items);

    let mut dummies = client
        .generate_dummy_items(nb_items, ct_size)
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    /*
     * Server.
     */
    let mut path_oram = Oram::new(&mut dummies, nb_items)?;

    // Let's read path 3, of all 16 paths from 0 to 15 included.
    let path = 3;

    let path_values_option =
        path_oram.access(AccessType::Read, path, Option::None)?;

    assert!(path_values_option.is_some());
    let mut path_values = path_values_option.unwrap();

    /*
     * Client side now.
     * After decryption, change item number 6.
     */

    // Decrypt values from tree here.
    client
        .decrypt_items(&mut path_values)
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    // Decrypt client stash.
    client
        .decrypt_stash()
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    /*
     * Here path_values is a vector containing plaintexts.
     * ...
     * Do changes here.
     */

    // Encrypt read items to write them back to the ORAM.
    client
        .encrypt_items(&mut path_values, [6].to_vec())
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    // Encrypt back the stash.
    client
        .encrypt_stash()
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    /*
     * Each Read operation **must** be followed by a write operation on the same
     * path. Client sends new DataItems to write to the tree alongside with his
     * current stash.
     * Server side.
     */
    let path_values_remnants_opt = path_oram.access(
        AccessType::Write,
        path,
        Some(&mut [client.stash, path_values].concat()),
    )?;

    assert!(path_values_remnants_opt.is_some());
    let path_values_remnants = path_values_remnants_opt.unwrap();

    /*
     * Server sends back remnants items it could not load into the tree. They
     * constitute the new stash.
     */
    client.stash = path_values_remnants;

    Ok(())
}
