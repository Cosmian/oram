mod btree;
mod oram;
mod oram_tests;

use std::io::{Error, ErrorKind};

use crate::{
    btree::DataItem,
    oram::{AccessType, BUCKET_SIZE, ORAM},
};

use rand::Rng;

const CIPHERTEXTS: [[u8; 8]; 60] = [
    [135, 247, 125, 248, 107, 81, 175, 245],
    [89, 99, 72, 36, 12, 187, 16, 206],
    [35, 230, 207, 24, 1, 110, 185, 209],
    [168, 82, 19, 123, 7, 166, 72, 165],
    [98, 76, 87, 149, 48, 249, 81, 17],
    [71, 89, 103, 97, 26, 136, 94, 180],
    [171, 172, 219, 85, 225, 113, 114, 140],
    [110, 64, 136, 115, 174, 233, 94, 151],
    [17, 196, 232, 37, 204, 89, 210, 168],
    [166, 158, 164, 215, 114, 159, 218, 200],
    [93, 141, 183, 158, 214, 92, 95, 71],
    [108, 55, 24, 42, 201, 149, 202, 236],
    [222, 234, 65, 207, 183, 17, 99, 84],
    [232, 110, 187, 61, 203, 32, 134, 31],
    [70, 158, 34, 94, 243, 67, 164, 111],
    [238, 13, 170, 23, 80, 85, 164, 227],
    [234, 104, 212, 170, 49, 53, 117, 114],
    [90, 142, 177, 230, 4, 65, 4, 14],
    [14, 75, 230, 53, 69, 158, 102, 252],
    [38, 206, 195, 199, 10, 39, 126, 194],
    [186, 254, 236, 20, 110, 82, 76, 74],
    [60, 233, 47, 6, 221, 76, 120, 79],
    [106, 163, 217, 170, 236, 3, 121, 151],
    [122, 53, 69, 58, 181, 172, 77, 77],
    [42, 109, 52, 118, 228, 23, 211, 164],
    [36, 87, 168, 121, 234, 113, 89, 251],
    [168, 94, 238, 198, 94, 83, 75, 132],
    [34, 140, 116, 207, 241, 78, 55, 225],
    [52, 84, 78, 89, 48, 5, 81, 243],
    [41, 46, 187, 21, 52, 59, 181, 187],
    [119, 15, 147, 173, 199, 50, 210, 60],
    [207, 54, 129, 248, 111, 172, 102, 206],
    [219, 181, 75, 86, 60, 16, 96, 65],
    [155, 62, 204, 108, 93, 240, 144, 250],
    [151, 210, 105, 155, 36, 200, 45, 13],
    [8, 44, 59, 175, 40, 244, 100, 253],
    [253, 181, 44, 9, 4, 53, 58, 160],
    [100, 195, 33, 181, 187, 188, 149, 136],
    [235, 58, 229, 252, 101, 102, 14, 68],
    [183, 206, 146, 255, 210, 80, 52, 31],
    [26, 181, 143, 141, 28, 158, 103, 133],
    [111, 69, 84, 200, 64, 54, 43, 43],
    [209, 203, 110, 41, 85, 60, 121, 131],
    [86, 201, 17, 22, 235, 181, 206, 27],
    [90, 176, 18, 63, 226, 59, 86, 46],
    [230, 45, 157, 186, 146, 72, 0, 196],
    [27, 231, 28, 122, 82, 90, 237, 177],
    [251, 217, 82, 149, 139, 203, 212, 203],
    [138, 190, 185, 186, 60, 210, 101, 240],
    [49, 161, 141, 149, 125, 87, 21, 180],
    [33, 76, 163, 115, 225, 148, 68, 109],
    [21, 140, 152, 106, 109, 130, 34, 250],
    [78, 84, 191, 232, 229, 94, 150, 50],
    [136, 188, 157, 150, 63, 34, 226, 157],
    [197, 65, 138, 161, 135, 86, 22, 155],
    [55, 145, 12, 130, 39, 255, 79, 28],
    [47, 61, 78, 73, 83, 244, 107, 191],
    [4, 243, 43, 100, 222, 128, 57, 189],
    [150, 241, 247, 214, 239, 57, 226, 161],
    [163, 102, 112, 243, 1, 68, 53, 172],
];

fn main() -> Result<(), Error> {
    println!("Hello, Path-ORAM!");
    let mut rng: rand::rngs::ThreadRng = rand::thread_rng();

    /* Client. */

    let nb_blocks: usize = 15;
    let nb_leaves = (1 << (nb_blocks.ilog2() + 1)) / 2;
    let mut dummies = Vec::new();
    for ct in CIPHERTEXTS[0..nb_blocks * BUCKET_SIZE].to_vec() {
        dummies.push(DataItem::new(ct.to_vec(), rng.gen_range(0..nb_leaves)));
    }
    println!("Number of leaves: {:?}", nb_leaves);

    /* Server. */
    let mut path_oram = ORAM::new(dummies.as_mut(), nb_blocks).unwrap();

    let path = 3;

    let path_values = path_oram
        .access(AccessType::Read, path, Option::None)
        .unwrap();

    if path_values.is_none() {
        println!("INVALID PATH");
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "Invalid path access. Got {}, expected in range 0..{}",
                path,
                path_oram.tree().height() - 1
            ),
        ));
    }

    // After decryption, change block number 6.
    if let Some(mut path_values_write) = path_values {
        let block_change = 6;

        path_values_write[block_change]
            .set_data([21, 188, 234, 210, 25, 251, 146, 34].to_vec());
        path_values_write[block_change].set_path(rng.gen_range(0..nb_leaves));

        let path_values_remnants = path_oram
            .access(AccessType::Write, path, Some(path_values_write.as_mut()))
            .unwrap();

        if let Some(path_values_remnants) = path_values_remnants {
            println!(
                "Number of remnants: {:?}",
                path_values_remnants
                    .iter()
                    .filter(|r| !r.data().is_empty())
                    .collect::<Vec<_>>()
                    .len()
            );
        }
    }

    Ok(())
}
