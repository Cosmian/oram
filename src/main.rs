mod btree;
mod client;
mod oram;
#[cfg(test)]
mod tests;

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

    let dummy_items = client
        .generate_dummy_items(nb_items, ct_size)
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    /*
     * Server.
     */
    let mut path_oram = Oram::new(dummy_items, nb_items)?;

    // Let's read path number 22.
    let path = 22;

    // TODO (TBZ): `path_data` should not need mutability.
    let mut path_data = path_oram.access(AccessType::Read, path)?.unwrap();

    /*
     * We received server response. Moving client side...
     */

    // Decrypt them nonetheless.
    client
        .decrypt_items(&mut path_data)
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    assert_eq!(path_data[9].data(), &vec![0; ct_size]);

    // Let's add some real data to our position map now.
    let mut csprng = CsRng::from_entropy();
    let mut new_values = Vec::with_capacity(50);

    for _ in 0..26 {
        let mut rand_value = vec![0; ct_size];
        csprng.fill_bytes(&mut rand_value);
        let data_item = DataItem::new(rand_value.clone());

        client.position_map.insert(rand_value.clone(), 0);

        let res_chg = client.change_element_position(&data_item);
        assert!(res_chg.is_ok());

        new_values.push(data_item);
    }

    // Stash and elements read from path are combined and ordered.
    let mut ordered_elements = client.order_elements_for_writing(
        &[new_values.as_slice(), path_data.as_slice()].concat(),
        path,
        path_oram.tree().height() as usize,
    );

    /* We ordered elements and put the ones that could not be written in the
     * stash. Since we want to write 26 elements and a path can only contain
     * tree.height * BUCKET_SIZE = 6 * 4 (here) = 24 elements, the stash has
     * to be not empty. However, its size can vary since we assigned paths
     * at random.
     */
    assert!(!client.stash.is_empty());

    assert_eq!(ordered_elements.len(), path_oram.tree().height() as usize);
    assert_eq!(ordered_elements[0].len(), BUCKET_SIZE);

    // Encrypt read items to write them back to the ORAM.
    client
        .encrypt_items(&mut ordered_elements)
        .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

    /*
     * Sending data to the server for writing...
     * Each Read operation **must** be followed by a write operation on the same
     * path. Client sends new DataItems to write to the tree alongside with his
     * current stash (this is done during order_element_for_writing()).
     */
    let opt_write = path_oram.access(AccessType::Write(ordered_elements), path)?;
    assert!(opt_write.is_none());

    Ok(())
}
