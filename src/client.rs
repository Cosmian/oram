use cosmian_crypto_core::{
    reexport::rand_core::SeedableRng, Aes256Gcm, CryptoCoreError, CsRng, Dem,
    FixedSizeCBytes, Instantiable, Nonce, RandomFixedSizeCBytes, SymmetricKey,
};
use rand::{Rng, RngCore};
use std::io::{Error, ErrorKind};

use crate::{btree::DataItem, oram::BUCKET_SIZE};

pub struct ClientORAM {
    pub stash: Vec<DataItem>,
    csprng: CsRng,
    key: SymmetricKey<{ Aes256Gcm::KEY_LENGTH }>,
}

impl ClientORAM {
    pub fn new() -> ClientORAM {
        let mut csprng = CsRng::from_entropy();
        let key = SymmetricKey::new(&mut csprng);

        ClientORAM {
            // Empty stash at initialization.
            stash: Vec::new(),
            csprng,
            key,
        }
    }

    pub fn gen_key(&mut self) {
        self.key = SymmetricKey::new(&mut self.csprng);
    }

    pub fn generate_dummies(
        &mut self,
        nb_dummies: usize,
        block_size: usize,
    ) -> Result<Vec<DataItem>, Error> {
        if !(nb_dummies + 1).is_power_of_two() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Number of blocks shall be power of 2 minus one".to_string(),
            ));
        }

        // Number of leaves is 2^(l-1) if number of blocks is 2^l - 1.
        let nb_leaves = (nb_dummies + 1) / 2;
        let mut dummies = Vec::new();

        let cipher = Aes256Gcm::new(&self.key);

        // FIXME - encrypt fixed dummy value instead of encrypting randoms.
        for _ in 0..nb_dummies * BUCKET_SIZE {
            // Generate new random dummy data.
            let mut dummy_data = vec![0; block_size];
            self.csprng.fill_bytes(&mut dummy_data);

            // Generate new nonce for encryption.
            let nonce = Nonce::new(&mut self.csprng);

            // Encrypt dummies to provide correct MAC for later decryption.
            let ciphertext_res =
                cipher.encrypt(&nonce, &dummy_data, Option::None);

            if ciphertext_res.is_err() {
                return Err(Error::new(
                    ErrorKind::Interrupted,
                    Result::unwrap_err(ciphertext_res).to_string(),
                ));
            }

            let encrypted_dummy =
                [nonce.as_bytes(), ciphertext_res.unwrap().as_slice()].concat();

            // FIXME- uniform generation for now. Is fine but dummies' paths do
            // not need to be generated at random.
            let path = self.csprng.gen_range(0..nb_leaves);

            dummies.push(DataItem::new(encrypted_dummy, path as u16));
        }

        Ok(dummies)
    }

    pub fn encrypt_blocks(
        &mut self,
        blocks: &mut Vec<DataItem>,
        block_ids: Vec<usize>,
        max_path: usize,
    ) -> Result<(), CryptoCoreError> {
        self.gen_key();
        let cipher = Aes256Gcm::new(&self.key);

        for i in 0..blocks.len() {
            // Generate new nonce for encryption.
            let nonce = Nonce::new(&mut self.csprng);

            let ciphertext_res =
                cipher.encrypt(&nonce, blocks[i].data(), Option::None);
            if ciphertext_res.is_err() {
                return Err(Result::unwrap_err(ciphertext_res));
            }

            // Change element data to plaintext.
            blocks[i].set_data(
                [nonce.as_bytes(), ciphertext_res.unwrap().as_slice()].concat(),
            );

            /*
             * If the block is among the ones to change, change its path by
             * sampling a random uniform distribution.
             */
            if block_ids.contains(&i) {
                blocks[i].set_path(self.csprng.gen_range(0..max_path as u16))
            }
        }

        Ok(())
    }

    pub fn decrypt_blocks(
        &self,
        blocks: &mut Vec<DataItem>,
    ) -> Result<(), CryptoCoreError> {
        let cipher = Aes256Gcm::new(&self.key);

        for i in 0..blocks.len() {
            // Edge-case where dummies left cells uninitialized.
            if blocks[i].data().is_empty() {
                continue;
            }

            let nonce_res = Nonce::try_from_slice(
                &blocks[i].data()[..Aes256Gcm::NONCE_LENGTH],
            );
            if nonce_res.is_err() {
                return Err(Result::unwrap_err(nonce_res));
            }

            let nonce = nonce_res.unwrap();
            let plaintext_res = cipher.decrypt(
                &nonce,
                &blocks[i].data()[Aes256Gcm::NONCE_LENGTH..],
                Option::None,
            );

            if plaintext_res.is_err() {
                return Err(Result::unwrap_err(plaintext_res));
            }

            blocks[i].set_data(plaintext_res.unwrap());
        }

        Ok(())
    }
}
