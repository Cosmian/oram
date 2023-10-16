use crate::{btree::DataItem, oram::BUCKET_SIZE};
use cosmian_crypto_core::{
    reexport::rand_core::SeedableRng, Aes256Gcm, CryptoCoreError, CsRng, Dem, FixedSizeCBytes,
    Instantiable, Nonce, RandomFixedSizeCBytes, SymmetricKey,
};
use rand::Rng;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    ops::DerefMut,
    sync::Mutex,
};

pub struct ClientOram {
    pub stash: Vec<DataItem>,
    pub position_map: HashMap<Vec<u8>, usize>,
    nb_items: usize,
    rng: Mutex<CsRng>,
    cipher: Aes256Gcm,
}

impl ClientOram {
    pub fn new(nb_items: usize) -> ClientOram {
        let mut csprng = CsRng::from_entropy();
        let key = SymmetricKey::new(&mut csprng);

        let stash_capacity = if nb_items == 0 {
            0
        } else {
            (nb_items.ilog2() + 1) as usize
        };

        ClientOram {
            /*
             * Empty stash at initialization as described in
             * `https://eprint.iacr.org/2013/280`.
             */
            stash: Vec::with_capacity(stash_capacity),
            position_map: HashMap::with_capacity(nb_items),
            nb_items,
            rng: Mutex::new(csprng),
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn generate_dummy_items(
        &self,
        nb_dummy_items: usize,
        ct_size: usize,
    ) -> Result<Vec<DataItem>, CryptoCoreError> {
        (0..nb_dummy_items)
            .map(|_| {
                let nonce = Nonce::new(self.rng.lock().expect("lock poisonned").deref_mut());
                self.cipher
                    .encrypt(&nonce, &vec![0; ct_size], None)
                    .map(|ciphertext| {
                        DataItem::new([nonce.as_bytes(), ciphertext.as_slice()].concat())
                    })
            })
            .collect()
    }

    /// Orders elements stack wise. Elements to be placed first will be last in
    /// the vector.
    pub fn order_elements_for_writing(
        &mut self,
        elts: &Vec<DataItem>,
        path: usize,
        tree_height: usize,
    ) -> Vec<[DataItem; BUCKET_SIZE]> {
        let mut ordered_elements: Vec<[DataItem; BUCKET_SIZE]> = Vec::with_capacity(tree_height);

        let size = elts[0].data().len();

        let mut elements = [self.stash.as_slice(), elts.as_slice()].concat();

        for level in 0..tree_height {
            let mut bucket = [
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
            ];

            for slot in bucket.iter_mut() {
                for (j, data_item) in elements.iter().enumerate() {
                    if let Some(&data_item_path) = self.position_map.get(data_item.data()) {
                        if data_item_path >> level == path >> level {
                            *slot = elements.remove(j);
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
            .filter(|data_item| self.position_map.contains_key(data_item.data()))
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

    pub fn change_element_position(&mut self, elt: &DataItem) -> Result<(), Error> {
        /*
         * Number of leaves (max_path) is the previous power of two of the
         * number of elements.
         */
        let max_path = 1 << (self.nb_items / BUCKET_SIZE).ilog2();

        let position = self.position_map.get_mut(elt.data()).ok_or(Error::new(
            ErrorKind::Interrupted,
            format!("Error: element {:?} is not in the position map.", elt),
        ))?;

        let mut rng = self.rng.lock().expect("lock poisonned");
        *position = rng.deref_mut().gen_range(0..max_path);

        Ok(())
    }

    pub fn encrypt_items(
        &self,
        buckets: &mut Vec<[DataItem; BUCKET_SIZE]>,
    ) -> Result<(), CryptoCoreError> {
        for bucket in buckets {
            for item in bucket {
                // Generate new nonce for encryption.
                let nonce = Nonce::new(self.rng.lock().expect("lock poisonned").deref_mut());

                let ciphertext = self.cipher.encrypt(&nonce, item.data(), Option::None)?;

                // Change element data to ciphertext.
                item.set_data([nonce.as_bytes(), ciphertext.as_slice()].concat());
            }
        }

        Ok(())
    }

    pub fn decrypt_items(&self, items: &mut [DataItem]) -> Result<(), CryptoCoreError> {
        for item in items {
            // Edge-case where dummies left cells uninitialized.
            if item.data().is_empty() {
                continue;
            }

            let nonce = Nonce::try_from_slice(&item.data()[..Aes256Gcm::NONCE_LENGTH])?;

            let plaintext = self.cipher.decrypt(
                &nonce,
                &item.data()[Aes256Gcm::NONCE_LENGTH..],
                Option::None,
            )?;

            item.set_data(plaintext);
        }

        Ok(())
    }
}
