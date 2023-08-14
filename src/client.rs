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
    cipher: Aes256Gcm,
}

impl ClientORAM {
    pub fn new() -> ClientORAM {
        let mut csprng = CsRng::from_entropy();
        let key = SymmetricKey::new(&mut csprng);

        ClientORAM {
            /* Empty stash at initialization as described in
             * `https://eprint.iacr.org/2013/280`.
             */
            stash: Vec::new(),
            csprng,
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn generate_dummy_items(
        &mut self,
        nb_dummy_items: usize,
        ct_size: usize,
    ) -> Result<Vec<DataItem>, Error> {
        if !(nb_dummy_items + 1).is_power_of_two() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Number of dummy items shall be power of 2 minus one"
                    .to_string(),
            ));
        }

        // Number of leaves is 2^(l-1) if number of items is 2^l - 1.
        let nb_leaves = (nb_dummy_items + 1) / 2;
        let mut dummy_items = Vec::new();

        // FIXME - encrypt fixed dummy value instead of encrypting randoms.
        for _ in 0..nb_dummy_items * BUCKET_SIZE {
            // Generate new random dummy data.
            let mut dummy_data = vec![0; ct_size];
            self.csprng.fill_bytes(&mut dummy_data);

            // Generate new nonce for encryption.
            let nonce = Nonce::new(&mut self.csprng);

            // Encrypt dummies to provide correct MAC for later decryption.
            let ciphertext_res =
                self.cipher.encrypt(&nonce, &dummy_data, Option::None);

            if ciphertext_res.is_err() {
                return Err(Error::new(
                    ErrorKind::Interrupted,
                    Result::unwrap_err(ciphertext_res).to_string(),
                ));
            }

            let encrypted_dummy =
                [nonce.as_bytes(), ciphertext_res.unwrap().as_slice()].concat();

            // FIXME- uniform generation for now. Is fine but dummies' paths do
            // not necessarily need to be generated at random.
            let path = self.csprng.gen_range(0..nb_leaves);

            dummy_items.push(DataItem::new(encrypted_dummy, path as u16));
        }

        Ok(dummy_items)
    }

    pub fn encrypt_items(
        &mut self,
        items: &mut Vec<DataItem>,
        changed_items_idx: Vec<usize>,
        max_path: usize,
    ) -> Result<(), CryptoCoreError> {
        let mut i = 0;
        while i < items.len() {
            // Generate new nonce for encryption.
            let nonce = Nonce::new(&mut self.csprng);

            let ciphertext_res =
                self.cipher.encrypt(&nonce, items[i].data(), Option::None);
            if ciphertext_res.is_err() {
                return Err(Result::unwrap_err(ciphertext_res));
            }

            // Change element data to plaintext.
            items[i].set_data(
                [nonce.as_bytes(), ciphertext_res.unwrap().as_slice()].concat(),
            );

            /*
             * If the block is among the ones to change, change its path by
             * sampling a random uniform distribution.
             */
            if changed_items_idx.contains(&i) {
                items[i].set_path(self.csprng.gen_range(0..max_path as u16))
            }

            i += 1;
        }

        Ok(())
    }

    pub fn decrypt_items(
        &self,
        items: &mut Vec<DataItem>,
    ) -> Result<(), CryptoCoreError> {
        let mut i = 0;
        while i < items.len() {
            // Edge-case where dummies left cells uninitialized.
            if items[i].data().is_empty() {
                i += 1;
                continue;
            }

            let nonce_res = Nonce::try_from_slice(
                &items[i].data()[..Aes256Gcm::NONCE_LENGTH],
            );
            if nonce_res.is_err() {
                return Err(Result::unwrap_err(nonce_res));
            }

            let nonce = nonce_res.unwrap();
            let plaintext_res = self.cipher.decrypt(
                &nonce,
                &items[i].data()[Aes256Gcm::NONCE_LENGTH..],
                Option::None,
            );

            if plaintext_res.is_err() {
                return Err(Result::unwrap_err(plaintext_res));
            }

            items[i].set_data(plaintext_res.unwrap());

            i += 1;
        }

        Ok(())
    }
}
