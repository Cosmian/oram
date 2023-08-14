mod btree;
mod client;
mod oram;
mod oram_tests;

use std::io::{Error, ErrorKind};

use crate::{
    client::ClientORAM,
    oram::{AccessType, ORAM},
};

fn main() -> Result<(), Error> {
    println!("Hello, Path-ORAM!");

    /*
     * Example of use for 15 items stored and a ciphertext size of 16 bytes.
     */
    let nb_items: usize = 15;
    let nb_leaves = (nb_items + 1) / 2;
    let ct_size: usize = 16;

    /*
     * Client.
     */
    let mut client = ClientORAM::new();

    let dummies_result = client.generate_dummy_items(nb_items, ct_size);

    assert!(dummies_result.is_ok());
    let mut dummies = dummies_result.unwrap();

    /*
     * Server.
     */
    let mut path_oram = ORAM::new(dummies.as_mut(), nb_items).unwrap();

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
     * After decryption, change item number 6.
     */
    let decrypt_res = client.decrypt_items(&mut path_values);
    if decrypt_res.is_err() {
        return Err(Error::new(
            ErrorKind::Interrupted,
            Result::unwrap_err(decrypt_res).to_string(),
        ));
    }

    // Here path_values is a vector containing plaintexts.

    let encrypt_res =
        client.encrypt_items(&mut path_values, [6].to_vec(), nb_leaves);
    if encrypt_res.is_err() {
        return Err(Error::new(
            ErrorKind::Interrupted,
            Result::unwrap_err(encrypt_res).to_string(),
        ));
    }

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
