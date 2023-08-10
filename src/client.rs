use cosmian_crypto_core::{reexport::rand_core::SeedableRng, CsRng};
use rand::{Rng, RngCore};
use std::io::{Error, ErrorKind};

use crate::{btree::DataItem, oram::BUCKET_SIZE};

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 16;

pub struct ClientORAM {
    pub stash: Vec<DataItem>,
    pub csprng: CsRng, // FIXME back to private when change_block is done.
    key: [u8; KEY_SIZE],
    nonce: [u8; NONCE_SIZE],
}

impl ClientORAM {
    pub fn new() -> ClientORAM {
        ClientORAM {
            // Empty stash at initialization.
            stash: Vec::new(),
            csprng: CsRng::from_entropy(),
            key: [0; KEY_SIZE],
            nonce: [0; NONCE_SIZE],
        }
    }

    pub fn gen_key(&mut self) {
        self.csprng.fill_bytes(&mut self.key);
        self.csprng.fill_bytes(&mut self.nonce);
    }

    pub fn generate_dummies(
        &mut self,
        nb_blocks: usize,
        block_size: usize,
    ) -> Result<Vec<DataItem>, Error> {
        if !(nb_blocks + 1).is_power_of_two() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Number of blocks shall be power of 2 minus one".to_string(),
            ));
        }

        // Number of leaves is 2^(l-1) if number of blocks is 2^l - 1.
        let nb_leaves = (nb_blocks + 1) / 2;
        let mut dummies = Vec::new();

        for _ in 0..nb_blocks * BUCKET_SIZE {
            let mut dummy_data = Vec::with_capacity(block_size);
            self.csprng.fill_bytes(&mut dummy_data);

            // FIXME- uniform generation for now. Is fine but dummies' paths do
            // not need to be generated at random.
            let path = self.csprng.gen_range(0..nb_leaves);

            dummies.push(DataItem::new(dummy_data, path as u16));
        }

        Ok(dummies)
    }

    pub fn change_block(
        &mut self,
        blocks: &mut Vec<DataItem>,
        block_ids: Vec<usize>,
        block_size: usize,
        max_path: usize,
    ) {
        // Decrypt block.
        // decrypt(blocks[block_id]);
        // XXX - for now just random sample u8 values to simulate encryption.
        block_ids.iter().for_each(|&i| {
            let mut new_ciphertext = Vec::with_capacity(block_size);
            self.csprng.fill_bytes(&mut new_ciphertext);
            blocks[i].set_data(new_ciphertext);

            // Assign new random uniform path to block.
            blocks[i].set_path(self.csprng.gen_range(0..max_path as u16));
        });

        // gen new ecnryption key
        self.gen_key()

        // Encrypt all blocks.
        // blocks.iter().for_each(|item| item.data().encrypt())
    }
}
