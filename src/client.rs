use crate::btree::DataItem;
use cosmian_crypto_core::{
    reexport::rand_core::SeedableRng, Aes256Gcm, CryptoCoreError, CsRng, Dem,
    FixedSizeCBytes, Instantiable, Nonce, RandomFixedSizeCBytes, SymmetricKey,
};
use rand::{Rng, RngCore};

pub struct ClientOram {
    pub stash: Vec<DataItem>,
    csprng: CsRng,
    cipher: Aes256Gcm,
}

impl ClientOram {
    pub fn new() -> ClientOram {
        let mut csprng = CsRng::from_entropy();
        let key = SymmetricKey::new(&mut csprng);

        ClientOram {
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
    ) -> Result<Vec<DataItem>, CryptoCoreError> {
        // Number of leaves is 2^(l-1) if number of items is 2^l - 1.
        let nb_leaves = (nb_dummy_items + 1) / 2;
        let mut dummy_items = Vec::with_capacity(nb_dummy_items);

        // FIXME - encrypt fixed dummy value instead of encrypting randoms.
        for _ in 0..nb_dummy_items {
            // Generate new random dummy data.
            let mut dummy_data = vec![0; ct_size];
            self.csprng.fill_bytes(&mut dummy_data);

            // Generate new nonce for encryption.
            let nonce = Nonce::new(&mut self.csprng);

            // Encrypt dummies to provide correct MAC for later decryption.
            let encrypted_data =
                self.cipher.encrypt(&nonce, &dummy_data, None)?;

            let encrypted_dummy =
                [nonce.as_bytes(), encrypted_data.as_slice()].concat();

            // FIXME- uniform generation for now. Is fine but dummies' paths do
            // not necessarily need to be generated at random.
            let path = self.csprng.gen_range(0..nb_leaves);

            dummy_items.push(DataItem::new(encrypted_dummy, path));
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

            let ciphertext =
                self.cipher.encrypt(&nonce, items[i].data(), Option::None)?;

            // Change element data to ciphertext.
            items[i]
                .set_data([nonce.as_bytes(), ciphertext.as_slice()].concat());

            /*
             * If the block is among the ones to change, change its path by
             * sampling a random uniform distribution.
             */
            if changed_items_idx.contains(&i) {
                items[i].set_path(self.csprng.gen_range(0..max_path))
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

            let nonce = Nonce::try_from_slice(
                &items[i].data()[..Aes256Gcm::NONCE_LENGTH],
            )?;

            let plaintext = self.cipher.decrypt(
                &nonce,
                &items[i].data()[Aes256Gcm::NONCE_LENGTH..],
                Option::None,
            )?;

            items[i].set_data(plaintext);

            i += 1;
        }

        Ok(())
    }
}
