use crate::{btree::DataItem, oram::BUCKET_SIZE};
use cosmian_crypto_core::{
    reexport::rand_core::SeedableRng, Aes256Gcm, CryptoCoreError, CsRng, Dem,
    FixedSizeCBytes, Instantiable, Nonce, RandomFixedSizeCBytes, SymmetricKey,
};
use rand::Rng;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

pub struct ClientOram {
    pub stash: Vec<DataItem>,
    pub position_map: HashMap<Vec<u8>, usize>,
    nb_items: usize,
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
            position_map: HashMap::with_capacity(nb_items),
            nb_items,
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

    /// Orders elements stack wise. Elements to be placed first will be last in
    /// the vector.
    pub fn order_elements_for_writing(
        &mut self,
        elts: &Vec<DataItem>,
        path: usize,
        tree_height: usize,
    ) -> Vec<[DataItem; BUCKET_SIZE]> {
        let mut ordered_elements: Vec<[DataItem; BUCKET_SIZE]> =
            Vec::with_capacity(tree_height);

        let size = elts[0].data().len();

        let mut elements = [self.stash.as_slice(), elts.as_slice()].concat();

        for level in (0..tree_height).rev() {
            let mut bucket = [
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
            ];

            for i in 0..BUCKET_SIZE {
                for (j, data_item) in elements.iter().enumerate() {
                    if let Some(&data_item_path) =
                        self.position_map.get(data_item.data())
                    {
                        if data_item_path >> level == path >> level {
                            bucket[i] = elements.remove(j);
                            break;
                        }
                    }
                }
            }

            bucket.iter_mut().for_each(|data_item| {
                if data_item.data().is_empty() {
                    data_item.set_data(vec![0; size]);
                }
            });

            ordered_elements.push(bucket);
        }

        /*
         * The elements that are not inserted at this point is because there are
         * more elements to write than slots in the path. They consitute the new
         * stash.
         */

        self.stash = elements
            .into_iter()
            .filter(|data_item| {
                self.position_map.contains_key(data_item.data())
            })
            .collect();

        ordered_elements
    }

    pub fn change_element_position(
        &mut self,
        elt: &DataItem,
    ) -> Result<(), Error> {
        /*
         * Number of leaves (max_path) is the previous power of two of the
         * number of elements.
         */
        let max_path = 1 << (self.nb_items / BUCKET_SIZE).ilog2();

        let position =
            self.position_map.get_mut(elt.data()).ok_or(Error::new(
                ErrorKind::Interrupted,
                format!("Error: element {:?} is not in the position map.", elt),
            ))?;

        *position = self.csprng.gen_range(0..max_path);

        Ok(())
    }

    pub fn encrypt_items(
        &mut self,
        buckets: &mut Vec<[DataItem; BUCKET_SIZE]>,
    ) -> Result<(), CryptoCoreError> {
        for bucket in buckets {
            for item in bucket {
                // Generate new nonce for encryption.
                let nonce = Nonce::new(&mut self.csprng);

                let ciphertext =
                    self.cipher.encrypt(&nonce, item.data(), Option::None)?;

                // Change element data to ciphertext.
                item.set_data(
                    [nonce.as_bytes(), ciphertext.as_slice()].concat(),
                );
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
