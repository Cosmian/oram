use crate::btree::DataItem;
use cosmian_crypto_core::{
    reexport::rand_core::SeedableRng, Aes256Gcm, CryptoCoreError, CsRng, Dem,
    FixedSizeCBytes, Instantiable, Nonce, RandomFixedSizeCBytes, SymmetricKey,
};
use rand::Rng;

pub struct ClientOram {
    pub stash: Vec<DataItem>,
    pub position_map: Vec<usize>,
    csprng: CsRng,
    cipher: Aes256Gcm,
}

impl ClientOram {
    pub fn new(nb_items: usize) -> ClientOram {
        let mut csprng = CsRng::from_entropy();
        let key = SymmetricKey::new(&mut csprng);

        let mut stash_capacity: usize = 0;
        if nb_items != 0 {
            stash_capacity = (nb_items.ilog2() + 1) as usize;
        }

        ClientOram {
            /*
             * Empty stash at initialization as described in
             * `https://eprint.iacr.org/2013/280`.
             */
            stash: Vec::with_capacity(stash_capacity),
            position_map: vec![0; nb_items],
            csprng,
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn generate_dummy_items(
        &mut self,
        nb_dummy_items: usize,
        ct_size: usize,
    ) -> Result<Vec<DataItem>, CryptoCoreError> {
        let mut dummy_items = Vec::with_capacity(nb_dummy_items);

        for _ in 0..nb_dummy_items {
            let dummy_data = vec![0; ct_size];

            // Generate new nonce for encryption.
            let nonce = Nonce::new(&mut self.csprng);

            // Encrypt null vector as dummies.
            let encrypted_data =
                self.cipher.encrypt(&nonce, &dummy_data, None)?;

            let encrypted_dummy =
                [nonce.as_bytes(), encrypted_data.as_slice()].concat();

            dummy_items.push(DataItem::new(encrypted_dummy));
        }

        Ok(dummy_items)
    }

    /// Orders elements stack wise. Elements to be places first will be last in
    /// the vector.
    pub fn order_elements_for_writing(&self, path: usize) {
        // TODO.
    }

    pub fn change_position(&mut self, i: usize) {
        /*
         * Number of leaves (max_path) is the previous power of two of the
         * number of elements.
         */
        let max_path = 1 << self.position_map.len().ilog2();
        self.position_map[i] = self.csprng.gen_range(0..max_path);
    }

    pub fn encrypt_items(
        &mut self,
        items: &mut [DataItem],
        changed_items_idx: Vec<usize>,
    ) -> Result<(), CryptoCoreError> {
        for (i, item) in items.iter_mut().enumerate() {
            // Generate new nonce for encryption.
            let nonce = Nonce::new(&mut self.csprng);

            let ciphertext =
                self.cipher.encrypt(&nonce, item.data(), Option::None)?;

            // Change element data to ciphertext.
            item.set_data([nonce.as_bytes(), ciphertext.as_slice()].concat());

            /*
             * If the block is among the ones to have changed, change its path
             * by sampling a random uniform distribution.
             */
            // FIXME - Belongs here ?
            if changed_items_idx.contains(&i) {
                self.change_position(i);
            }
        }

        Ok(())
    }

    pub fn decrypt_items(
        &self,
        items: &mut [DataItem],
    ) -> Result<(), CryptoCoreError> {
        for item in items {
            // Edge-case where dummies left cells uninitialized.
            if item.data().is_empty() {
                continue;
            }

            let nonce =
                Nonce::try_from_slice(&item.data()[..Aes256Gcm::NONCE_LENGTH])?;

            let plaintext = self.cipher.decrypt(
                &nonce,
                &item.data()[Aes256Gcm::NONCE_LENGTH..],
                Option::None,
            )?;

            item.set_data(plaintext);
        }

        Ok(())
    }

    pub fn encrypt_stash(&mut self) -> Result<(), CryptoCoreError> {
        for stash_item in self.stash.iter_mut() {
            let nonce = Nonce::new(&mut self.csprng);

            let ciphertext =
                self.cipher
                    .encrypt(&nonce, stash_item.data(), Option::None)?;

            // Change element data to ciphertext.
            stash_item
                .set_data([nonce.as_bytes(), ciphertext.as_slice()].concat());
        }

        Ok(())
    }

    pub fn decrypt_stash(&mut self) -> Result<(), CryptoCoreError> {
        for stash_item in self.stash.iter_mut() {
            let nonce = Nonce::try_from_slice(
                &stash_item.data()[..Aes256Gcm::NONCE_LENGTH],
            )?;

            let plaintext = self.cipher.decrypt(
                &nonce,
                &stash_item.data()[Aes256Gcm::NONCE_LENGTH..],
                Option::None,
            )?;

            stash_item.set_data(plaintext);
        }

        Ok(())
    }
}
