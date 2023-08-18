mod btree;
mod client;
mod oram;
mod oram_tests;

use crate::{
    btree::DataItem,
    client::ClientOram,
    oram::{AccessType, Oram, BUCKET_SIZE},
};
use cosmian_crypto_core::{reexport::rand_core::SeedableRng, CsRng};
use rand::RngCore;
use std::io::{Error, ErrorKind};

fn main() -> Result<(), Error> {
    println!("Hello, Path-Oram!");

    /*
     * Implementation from https://eprint.iacr.org/2013/280.
     *
     * Example of use for 183 items stored and a ciphertext size of 16 bytes.
     * This means that there will be floor(183/4) = 46 nodes to hold those
     * items which completes to 63 nodes for the tree. There will then be 32
     * leaves.
     */
    let nb_items: usize = 182;
    let ct_size: usize = 16;

    /*
     * Client.
     */
    let mut client = ClientOram::new(nb_items);

    // Let's insert elements to position map.
    let mut csprng = CsRng::from_entropy();
    let mut new_values = Vec::with_capacity(50);

    for _ in 0..26 {
        let mut rand_value = vec![0; ct_size];
        csprng.fill_bytes(&mut rand_value);
        let data_item = DataItem::new(rand_value.clone());

        client.insert_element_in_position_map(&data_item);

        new_values.push(data_item);
    }

    let mut oram = client.setup_oram(ct_size)?;

    let path = 22;
    let mut read_data = client.read_from_path(&mut oram, path)?;

    /* Changing an element in the values obtained */
    /* -------------------------------------------*/
    let idx_data_item_to_change = 6;

    // First remove old element from position map.
    client
        .delete_element_from_position_map(&read_data[idx_data_item_to_change]);

    let data_changed = read_data[idx_data_item_to_change].data_as_mut();
    data_changed[0] = 255;

    // Insert changed element.
    client.insert_element_in_position_map(&read_data[idx_data_item_to_change]);
    /* -------------------------------------------*/

    // I don't wish to insert new elements there.
    client.write_to_path(&mut oram, &mut read_data, Option::None, path)?;

    Ok(())
}
