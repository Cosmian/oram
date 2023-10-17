use crate::{
    btree::{udiv_ceil, DataItem},
    oram::{AccessType, Oram, BUCKET_SIZE},
};
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

    /// Orders `elts` into `tree.height` buckets of size `BUCKET_SIZE` in a
    /// stackwise position. This is later given to the server on a write op.
    /// For each level of the tree, computes a bucket of DataItem from `elts`
    /// matching the given path. If less than `BUCKET_SIZE` elements match, fill
    /// the bucket with dummy items (null vector).
    pub fn order_elements_for_writing(
        &mut self,
        elts: &mut Vec<DataItem>,
        path: usize,
        tree_height: usize,
    ) -> Vec<[DataItem; BUCKET_SIZE]> {
        let mut ordered_elements: Vec<[DataItem; BUCKET_SIZE]> =
            Vec::with_capacity(tree_height);

        let elt_size = elts[0].data().len();

        for level in 0..tree_height {
            let mut bucket = [
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
            ];

            for i in 0..BUCKET_SIZE {
                for (j, data_item) in elts.iter().enumerate() {
                    // Because dummies are not in the position map, they don't
                    // match the condition.
                    if let Some(&data_item_path) =
                        self.position_map.get(data_item.data())
                    {
                        if data_item_path >> level == path >> level {
                            bucket[i] = elts.remove(j);
                            break;
                        }
                    }
                }
            }

            bucket.iter_mut().for_each(|data_item| {
                if data_item.data().is_empty() {
                    data_item.set_data(vec![0; elt_size]);
                }
            });

            ordered_elements.push(bucket);
        }

        /*
         * The elements that are not inserted at this point is because there are
         * more elements to write than slots in the path. They consitute the new
         * stash.
         */

        self.stash = elts
            .iter()
            .filter(|data_item| {
                self.position_map.contains_key(data_item.data())
            })
            .cloned()
            .collect();

        /*
         * List is reversed here since when writing to the tree, elements are
         * popped stackwise. I doubt this can be directly constructed in reverse
         * since elements are pushed in the vector following less and less
         * strict conditions (path right shifting more and more).
         */
        ordered_elements.reverse();

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
        let max_path = 1 << udiv_ceil(self.nb_items, BUCKET_SIZE).ilog2();

        let position =
            self.position_map.get_mut(elt.data()).ok_or(Error::new(
                ErrorKind::Interrupted,
                format!("Error: element {:?} is not in the position map.", elt),
            ))?;

        *position = self.csprng.gen_range(0..max_path);

        Ok(())
    }

    /// Inserting an element provides him with a uniformly random generated path
    pub fn insert_element_in_position_map(&mut self, elt: &DataItem) {
        let max_path = 1 << udiv_ceil(self.nb_items, BUCKET_SIZE).ilog2();

        self.position_map
            .insert(elt.data().clone(), self.csprng.gen_range(0..max_path));
    }

    pub fn delete_element_from_position_map(&mut self, elt: &DataItem) {
        self.position_map.remove(elt.data());
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

    pub fn setup_oram(&mut self, ct_size: usize) -> Result<Oram, Error> {
        // Computes the number of slots in the complete tree.
        // We need ceil(nb_items / BUCKET_SIZE) buckets to store our elements.
        // the next power of two minus one of this number is the size of the
        // necessary complete tree.
        let slots_complete_tree =
            ((1 << (udiv_ceil(self.nb_items, BUCKET_SIZE).ilog2() + 1)) - 1)
                * BUCKET_SIZE;

        let mut dummy_items = self
            .generate_dummy_items(slots_complete_tree, ct_size)
            .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

        // Creating a new oram with potential prepended data.
        let oram = Oram::new(&mut dummy_items, self.nb_items)?;

        Ok(oram)
    }

    pub fn read_from_path(
        &mut self,
        oram: &mut Oram,
        path: usize,
    ) -> Result<Vec<DataItem>, Error> {
        if path > (1 << (oram.tree().height() - 1)) - 1 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid path access. Got {}, expected in range 0..{}",
                    path,
                    (1 << (oram.tree().height() - 1)) - 1
                ),
            ));
        }

        // Read values from path located in ORAM.
        let mut read_data = oram
            .access(AccessType::Read, path, Option::None)?
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Interrupted,
                    format!("No value returned from read at path {}", path),
                )
            })?;

        // Decrypt items read and client stash.
        self.decrypt_items(&mut read_data)
            .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;
        self.decrypt_stash()
            .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

        // Return decrypted data from path concatenated to stash.
        Ok([self.stash.as_slice(), read_data.as_slice()].concat())
    }

    pub fn write_to_path(
        &mut self,
        oram: &mut Oram,
        write_elts: &mut Vec<DataItem>,
        insert_new_elts: Option<&mut Vec<DataItem>>,
        path: usize,
    ) -> Result<(), Error> {
        if path > (1 << (oram.tree().height() - 1)) - 1 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid path access. Got {}, expected in range 0..{}",
                    path,
                    (1 << (oram.tree().height() - 1)) - 1
                ),
            ));
        }

        // Insert new elements to position map if specified.
        if let Some(insert_new_elts) = insert_new_elts {
            for data_item in insert_new_elts.iter() {
                self.position_map.insert(data_item.data().clone(), 0);
                self.change_element_position(data_item)?;
            }

            // Add inserted elements to elements to write.
            write_elts.append(insert_new_elts);
        }

        /* Stash and elements read from path are ordered in buckets.
         * Update stash with extra elements that could not be written.
         */
        let mut ordered_elements = self.order_elements_for_writing(
            write_elts,
            path,
            oram.tree().height() as usize,
        );

        // Encrypt read items to write them back to the ORAM.
        self.encrypt_items(&mut ordered_elements)
            .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

        // Encrypt back the stash.
        self.encrypt_stash()
            .map_err(|e| Error::new(ErrorKind::Interrupted, e.to_string()))?;

        oram.access(AccessType::Write, path, Some(&mut ordered_elements))?;

        Ok(())
    }
}
