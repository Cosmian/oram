#[cfg(test)]
mod tests {
    use cosmian_crypto_core::{reexport::rand_core::SeedableRng, CsRng};
    use rand::RngCore;

    use crate::{
        btree::{DataItem, Node},
        client::ClientOram,
        oram::{AccessType, Oram, BUCKET_SIZE},
    };

    fn _complete_tree_size(node: Option<&Box<Node>>) -> usize {
        if let Some(node) = node {
            return 1
                + _complete_tree_size(node.left.as_ref())
                + _complete_tree_size(node.right.as_ref());
        }
        0
    }

    #[test]
    fn complete_tree_size_zero() {
        let nb_items: usize = 0;

        let path_oram = Oram::new(&mut Vec::new(), nb_items);
        assert!(path_oram.is_err());
    }

    #[test]
    fn complete_tree_size_one() {
        let nb_items: usize = BUCKET_SIZE;

        let path_oram = Oram::new(&mut Vec::new(), nb_items);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, 1);
        }
    }

    #[test]
    fn complete_tree_size_pow_of_2() {
        let nb_items: usize = 32 * BUCKET_SIZE;

        let path_oram = Oram::new(&mut Vec::new(), nb_items);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, 63);
        }
    }

    #[test]
    fn complete_tree_size_exact() {
        let nb_items: usize = 15 * BUCKET_SIZE;

        let path_oram = Oram::new(&mut Vec::new(), nb_items);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, 15);
        }
    }

    #[test]
    fn complete_tree_size_rand() {
        let nb_items: usize = 26 * BUCKET_SIZE;

        let path_oram = Oram::new(&mut Vec::new(), nb_items);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, 31);
        }
    }

    #[test]
    fn access_bad_path() {
        let nb_items: usize = 15 * BUCKET_SIZE;

        let path_oram = Oram::new(&mut Vec::new(), nb_items);

        assert!(path_oram.is_ok());

        if let Ok(mut path_oram) = path_oram {
            let path = 1337;
            let res_access =
                path_oram.access(AccessType::Read, path, Option::None);

            assert!(res_access.is_err());
        }
    }

    #[test]
    fn access_bad_path2() {
        let nb_items: usize = 15 * BUCKET_SIZE;

        let path_oram = Oram::new(&mut Vec::new(), nb_items);

        assert!(path_oram.is_ok());

        if let Ok(mut path_oram) = path_oram {
            let path = 8;
            let res_access =
                path_oram.access(AccessType::Read, path, Option::None);

            assert!(res_access.is_err());
        }
    }

    #[test]
    fn access_valid_path1() {
        let nb_items: usize = 15 * BUCKET_SIZE;

        let res_oram = Oram::new(&mut Vec::new(), nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 3;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_access.is_ok());
    }

    #[test]
    fn access_valid_path2() {
        let nb_items: usize = 15 * BUCKET_SIZE;

        let res_oram = Oram::new(&mut Vec::new(), nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 7;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_access.is_ok());
    }

    #[test]
    fn access_valid_path3() {
        let nb_items: usize = 15 * BUCKET_SIZE;

        let res_oram = Oram::new(&mut Vec::new(), nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 0;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_access.is_ok());
    }

    #[test]
    fn access_valid_path_write_data_none() {
        let nb_items: usize = 15 * BUCKET_SIZE;

        let res_oram = Oram::new(&mut Vec::new(), nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 0;
        let res_access =
            path_oram.access(AccessType::Write, path, Option::None);
        assert!(res_access.is_err());
    }

    #[test]
    fn read_write_access() {
        let nb_items: usize = 15 * BUCKET_SIZE;
        let ct_size: usize = 16;

        let mut client = ClientOram::new(nb_items);

        let dummies_result = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = Oram::new(&mut dummies, nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 3;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();

        assert!(path_values_opt.is_some());
        let mut path_values = path_values_opt.unwrap();

        /*
         * Client side now.
         */
        let decryption_res = client.decrypt_items(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let mut csprng = CsRng::from_entropy();
        let mut new_value = vec![0; 3];
        csprng.fill_bytes(&mut new_value);

        client.position_map.insert(new_value.clone(), 0);
        let new_item = DataItem::new(new_value);

        let chg_res = client.change_element_position(&new_item);
        assert!(chg_res.is_ok());

        /* Manually insert an element in the stash to check if it empties after
         * ordering elements.
         */
        client.stash.push(DataItem::new(vec![0; 10]));

        let mut ordered_elements = client.order_elements_for_writing(
            &mut [path_values.as_slice(), &[new_item]].concat(),
            path,
            path_oram.tree().height() as usize,
        );

        assert_eq!(ordered_elements.len(), path_oram.tree().height() as usize);
        assert_eq!(ordered_elements[0].len(), BUCKET_SIZE);
        // Since ordering elements for writing also sets the stash, it should be
        // empty here.
        assert!(client.stash.is_empty());

        let encryption_res = client.encrypt_items(&mut ordered_elements);
        assert!(encryption_res.is_ok());

        assert_ne!(path_values.len(), 0);

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(&mut ordered_elements),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_none());
    }

    #[test]
    fn read_write_access_no_change() {
        let nb_items: usize = 15 * BUCKET_SIZE;
        let ct_size: usize = 16;

        let mut client = ClientOram::new(nb_items);

        let dummies_result = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = Oram::new(&mut dummies, nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 3;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();

        assert!(path_values_opt.is_some());
        let mut path_values = path_values_opt.unwrap();

        assert_ne!(path_values.len(), 0);

        /*
         * Client side now.
         * After decryption, don't change nothing.
         */
        let decryption_res = client.decrypt_items(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        /* Manually insert an element in the stash to check if it empties after
         * ordering elements.
         */
        client.stash.push(DataItem::new(vec![0; 10]));

        let mut ordered_elements = client.order_elements_for_writing(
            &mut path_values,
            path,
            path_oram.tree().height() as usize,
        );

        assert_eq!(ordered_elements.len(), path_oram.tree().height() as usize);
        assert_eq!(ordered_elements[0].len(), BUCKET_SIZE);
        // Since ordering elements for writing also sets the stash, it should be
        // empty here.
        assert!(client.stash.is_empty());

        let encryption_res = client.encrypt_items(&mut ordered_elements);
        assert!(encryption_res.is_ok());

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(&mut ordered_elements),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();

        assert!(path_values_opt.is_none());
    }

    #[test]
    fn read_write_acess_few_changes() {
        let nb_items: usize = 15 * BUCKET_SIZE;
        let ct_size: usize = 16;

        let mut client = ClientOram::new(nb_items);

        let dummies_result = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = Oram::new(&mut dummies, nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 3;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();

        assert!(path_values_opt.is_some());
        let mut path_values = path_values_opt.unwrap();

        assert_ne!(path_values.len(), 0);

        /*
         * Client side now.
         * After decryption, change item number 0, 3, 6 and 9.
         */
        let decryption_res = client.decrypt_items(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let mut csprng = CsRng::from_entropy();
        let mut new_values = Vec::with_capacity(4);

        for _ in 0..4 {
            let mut rand_value = vec![0; 3];
            csprng.fill_bytes(&mut rand_value);

            client.position_map.insert(rand_value.clone(), 0);

            let data_item = DataItem::new(rand_value);
            let res_chg = client.change_element_position(&data_item);
            assert!(res_chg.is_ok());

            new_values.push(data_item);
        }

        /* Manually insert an element in the stash to check if it empties after
         * ordering elements.
         */
        client.stash.push(DataItem::new(vec![0; 10]));

        let mut ordered_elements = client.order_elements_for_writing(
            &mut [path_values.as_slice(), new_values.as_slice()].concat(),
            path,
            path_oram.tree().height() as usize,
        );

        assert_eq!(ordered_elements.len(), path_oram.tree().height() as usize);
        assert_eq!(ordered_elements[0].len(), BUCKET_SIZE);
        // Since ordering elements for writing also sets the stash, it should be
        // empty here.
        assert!(client.stash.is_empty());

        let encryption_res = client.encrypt_items(&mut ordered_elements);
        assert!(encryption_res.is_ok());

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(&mut ordered_elements),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_none());
    }

    #[test]
    fn read_write_acess_all_changes() {
        let nb_items: usize = 15 * BUCKET_SIZE;
        let ct_size: usize = 16;

        let mut client = ClientOram::new(nb_items);

        let dummies_result = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = Oram::new(&mut dummies, nb_items);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 3;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();

        assert!(path_values_opt.is_some());
        let mut path_values = path_values_opt.unwrap();
        assert_ne!(path_values.len(), 0);

        /*
         * Client side now.
         */
        let decryption_res = client.decrypt_items(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let mut csprng = CsRng::from_entropy();
        let mut new_values = Vec::with_capacity(path_values.len());

        for _ in 0..path_values.len() + 2 {
            let mut rand_value = vec![0; 3];
            csprng.fill_bytes(&mut rand_value);

            client.position_map.insert(rand_value.clone(), 0);

            let data_item = DataItem::new(rand_value);
            let res_chg = client.change_element_position(&data_item);
            assert!(res_chg.is_ok());
            new_values.push(data_item);
        }

        let mut ordered_elements = client.order_elements_for_writing(
            &mut [path_values.as_slice(), new_values.as_slice()].concat(),
            path,
            path_oram.tree().height() as usize,
        );

        assert_eq!(ordered_elements.len(), path_oram.tree().height() as usize);
        assert_eq!(ordered_elements[0].len(), BUCKET_SIZE);

        // Stash should not be empty. The contrary could happen but with
        // negligible probability.
        assert!(!client.stash.is_empty());

        let encryption_res = client.encrypt_items(&mut ordered_elements);
        assert!(encryption_res.is_ok());

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(&mut ordered_elements),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_none());
    }

    #[test]
    fn generate_zero_dummies() {
        let nb_items: usize = 0;
        let ct_size = 16;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_items);
    }

    #[test]
    fn generate_dummies_random_number() {
        let nb_items: usize = 173;
        let ct_size = 16;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
    }

    #[test]
    fn generate_dummies_small() {
        let nb_items: usize = 15;
        let ct_size = 16;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_items);
    }

    #[test]
    fn generate_dummies_big() {
        let nb_items: usize = (1 << 11) - 1;
        let ct_size = 16;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_items);
    }

    #[test]
    fn generate_dummies_null_ct_size() {
        let nb_items: usize = 15;
        let ct_size = 0;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_items);
        // Nonce + tag length.
        assert_eq!(dummies[0].data().len(), 12 + 16);
    }

    #[test]
    fn generate_dummies_tremendous_ct_size() {
        let nb_items: usize = 15;
        let ct_size = 1000;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_items);
        // Nonce + tag length.
        assert_eq!(dummies[0].data().len(), ct_size + 12 + 16);
    }

    #[test]
    fn change_position_bad1() {
        let nb_items: usize = 15;
        let mut client = ClientOram::new(nb_items);

        let secret_message: Vec<u8> = [
            66, 114, 117, 99, 101, 32, 83, 99, 104, 110, 101, 105, 101, 114,
            32, 107, 101, 101, 112, 115, 32, 99, 111, 110, 115, 116, 97, 110,
            116, 32, 116, 105, 109, 101,
        ]
        .to_vec();

        let secret_message2: Vec<u8> = [
            66, 114, 117, 99, 101, 32, 83, 99, 104, 110, 101, 105, 101, 114,
            32, 107, 101, 101, 112, 115, 32, 99, 111, 110, 115, 116, 97, 110,
            116, 32, 115, 105, 122, 101,
        ]
        .to_vec();

        let data_item_secret2 = DataItem::new(secret_message2);

        client.position_map.insert(secret_message, 2);

        let res_chg = client.change_element_position(&data_item_secret2);
        assert!(res_chg.is_err());
        println!("{:?}", res_chg.unwrap_err().to_string());
    }

    #[test]
    fn change_position_bad2() {
        let nb_items: usize = 15;
        let mut client = ClientOram::new(nb_items);

        let secret_message: Vec<u8> = [
            66, 114, 117, 99, 101, 32, 83, 99, 104, 110, 101, 105, 101, 114,
            32, 107, 101, 101, 112, 115, 32, 99, 111, 110, 115, 116, 97, 110,
            116, 32, 116, 105, 109, 101,
        ]
        .to_vec();

        client.position_map.insert(secret_message, 2);

        let data_item_empty = DataItem::new(Vec::new());

        let res_chg = client.change_element_position(&data_item_empty);
        assert!(res_chg.is_err());
    }

    #[test]
    fn change_position() {
        let nb_items: usize = 60;
        let mut client = ClientOram::new(nb_items);

        let secret_message: Vec<u8> = [
            66, 114, 117, 99, 101, 32, 83, 99, 104, 110, 101, 105, 101, 114,
            32, 107, 101, 101, 112, 115, 32, 99, 111, 110, 115, 116, 97, 110,
            116, 32, 116, 105, 109, 101,
        ]
        .to_vec();

        client.position_map.insert(secret_message.clone(), 100);
        let data_item_secret1 = DataItem::new(secret_message.clone());

        let res_chg = client.change_element_position(&data_item_secret1);

        assert!(res_chg.is_ok());

        let new_value = client.position_map.get(&secret_message).unwrap();

        assert_ne!(*new_value, 100);
        assert!(*new_value < 8);
    }

    #[test]
    fn change_position_repeatedely() {
        let nb_items: usize = 100;
        let mut client = ClientOram::new(nb_items);

        let secret_message: Vec<u8> = [
            66, 114, 117, 99, 101, 32, 83, 99, 104, 110, 101, 105, 101, 114,
            32, 107, 101, 101, 112, 115, 32, 99, 111, 110, 115, 116, 97, 110,
            116, 32, 116, 105, 109, 101,
        ]
        .to_vec();

        client.position_map.insert(secret_message.clone(), 1337);
        let data_item_secret1 = DataItem::new(secret_message.clone());

        for _ in 0..1000 {
            let res_chg = client.change_element_position(&data_item_secret1);

            assert!(res_chg.is_ok());

            let new_value = client.position_map.get(&secret_message).unwrap();

            assert_ne!(*new_value, 1337);
            assert!(*new_value < 16);
        }
    }

    #[test]
    fn order_elements() {
        let nb_items: usize = 48;
        let mut client = ClientOram::new(nb_items);

        let elt1: Vec<u8> = [66, 114, 117].to_vec();
        let elt2: Vec<u8> = [99, 101, 32].to_vec();
        let elt3: Vec<u8> = [32, 83, 99].to_vec();
        let elt4: Vec<u8> = [104, 110, 101].to_vec();
        let elt5: Vec<u8> = [105, 101, 114].to_vec();
        let elt6: Vec<u8> = [32, 107, 101].to_vec();
        let elt7: Vec<u8> = [101, 112, 115].to_vec();
        let elt8: Vec<u8> = [32, 99, 111].to_vec();
        let elt9: Vec<u8> = [110, 115, 116].to_vec();
        let elt10: Vec<u8> = [97, 110, 116].to_vec();
        let elt11: Vec<u8> = [32, 116, 105].to_vec();

        client.position_map.insert(elt1.clone(), 0);
        client.position_map.insert(elt2.clone(), 1);
        client.position_map.insert(elt3.clone(), 4);
        client.position_map.insert(elt4.clone(), 4);
        client.position_map.insert(elt5.clone(), 2);
        client.position_map.insert(elt6.clone(), 3);
        client.position_map.insert(elt7.clone(), 6);
        client.position_map.insert(elt8.clone(), 1);
        client.position_map.insert(elt9.clone(), 3);
        client.position_map.insert(elt10.clone(), 0);
        client.position_map.insert(elt11.clone(), 5);

        client.stash =
            [DataItem::new(elt1.clone()), DataItem::new(elt2.clone())].to_vec();

        let path_values_decrypted = vec![
            DataItem::new(elt3.clone()),
            DataItem::new(elt4.clone()),
            DataItem::new(elt5.clone()),
            DataItem::new(elt6.clone()),
            DataItem::new(elt7.clone()),
            DataItem::new(elt8.clone()),
            DataItem::new(elt9.clone()),
            DataItem::new(elt10.clone()),
            DataItem::new(elt11.clone()),
        ];

        let path = 3;
        let tree_height = 4;
        let ordered_buckets = client.order_elements_for_writing(
            &mut [client.stash.as_slice(), path_values_decrypted.as_slice()]
                .concat(),
            path,
            tree_height,
        );

        assert_eq!(ordered_buckets.len(), tree_height);
        assert_eq!(ordered_buckets[0].len(), BUCKET_SIZE);
        assert_eq!(ordered_buckets[1].len(), BUCKET_SIZE);
        assert_eq!(ordered_buckets[2].len(), BUCKET_SIZE);
        assert_eq!(ordered_buckets[3].len(), BUCKET_SIZE);

        let empty = DataItem::new(vec![0; 3]);

        assert_eq!(
            ordered_buckets,
            vec![
                [
                    DataItem::new(elt3),
                    DataItem::new(elt4),
                    DataItem::new(elt7),
                    DataItem::new(elt11)
                ],
                [
                    DataItem::new(elt1),
                    DataItem::new(elt2),
                    DataItem::new(elt8),
                    DataItem::new(elt10)
                ],
                [
                    DataItem::new(elt5),
                    empty.clone(),
                    empty.clone(),
                    empty.clone()
                ],
                [
                    DataItem::new(elt6),
                    DataItem::new(elt9),
                    empty.clone(),
                    empty
                ],
            ]
        )
    }

    #[test]
    fn decrypt_dummies() {
        let nb_items: usize = 15;
        let ct_size = 16;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
        let mut dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_items);

        // Nonce + tag length.
        assert_eq!(dummies[0].data().len(), ct_size + 12 + 16);

        let decrypt_res = client.decrypt_items(&mut dummies);
        assert!(decrypt_res.is_ok());

        let null_vector: Vec<u8> = vec![0; ct_size];

        dummies
            .iter()
            .for_each(|dummy| assert_eq!(dummy.data(), &null_vector));
    }

    #[test]
    fn client_encrypt_decrypt_stash() {
        let nb_items: usize = 15;
        let ct_size = 16;
        let mut client = ClientOram::new(nb_items);

        client.stash = vec![DataItem::new(vec![0; ct_size]); 4];

        let stash_encrypt_res = client.encrypt_stash();
        assert!(stash_encrypt_res.is_ok());

        let null_vector: Vec<u8> = vec![0; ct_size];

        assert_ne!(client.stash[0].data(), &null_vector);

        let stash_decrypt_res = client.decrypt_stash();
        assert!(stash_decrypt_res.is_ok());

        assert_eq!(client.stash[0].data(), &null_vector);
    }

    #[test]
    fn client_encrypt_decrypt_empty_stash() {
        let nb_items: usize = 15;
        let mut client = ClientOram::new(nb_items);

        let stash_encrypt_res = client.encrypt_stash();
        assert!(stash_encrypt_res.is_ok());

        let stash_decrypt_res = client.decrypt_stash();
        assert!(stash_decrypt_res.is_ok());
    }

    #[test]
    fn general_behavior() {
        /*
         * Example of use for 183 items stored and a ciphertext size of 16 bytes.
         * This means that there will be ceil(183/4) = 46 nodes to hold those
         * items which completes to 63 nodes for the tree. There will then be 32
         * leaves.
         */
        let nb_items: usize = 183;
        let ct_size: usize = 16;

        /*
         * Client.
         */
        let mut client = ClientOram::new(nb_items);

        let res_dummies = client.generate_dummy_items(nb_items, ct_size);
        assert!(res_dummies.is_ok());
        let mut dummies = res_dummies.unwrap();

        /*
         * Server.
         */
        let res_oram = Oram::new(&mut dummies, nb_items);
        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        // Let's read path number 22.
        let path = 22;

        let res_access = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_access.is_ok());
        let opt_access = res_access.unwrap();
        assert!(opt_access.is_some());

        // This is the data we read from the ORAM, only dummies at this point.
        let mut read_data = opt_access.unwrap();

        /*
         * We received server response. Moving client side...
         */

        // Decrypt them nonetheless.
        let decrypt_res = client.decrypt_items(&mut read_data);
        assert!(decrypt_res.is_ok());

        // Decrypt client stash.
        let stsh_dec_res = client.decrypt_stash();
        assert!(stsh_dec_res.is_ok());

        // Now read_data contains plaintext values. Decrypted dummy is null.
        assert!(!read_data[9].data().is_empty());
        let null_vector: Vec<u8> = vec![0; ct_size];
        assert_eq!(read_data[9].data(), &null_vector);

        // Let's add some real data to our position map now.
        let mut csprng = CsRng::from_entropy();
        let mut new_values = Vec::with_capacity(
            path_oram.tree().height() as usize * BUCKET_SIZE + 2,
        );

        for _ in 0..(path_oram.tree().height() as usize * BUCKET_SIZE) + 2 {
            let mut rand_value = vec![0; ct_size];
            csprng.fill_bytes(&mut rand_value);
            let data_item = DataItem::new(rand_value.clone());

            client.position_map.insert(rand_value.clone(), 0);

            let res_chg = client.change_element_position(&data_item);
            assert!(res_chg.is_ok());

            new_values.push(data_item);
        }
        // Push a witness value for later.
        let witness: Vec<u8> = [
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
        ]
        .to_vec();
        client.position_map.insert(witness.clone(), path);

        new_values.push(DataItem::new(witness.clone()));

        /* We ordered elements and put the ones that could not be written in the
         * stash. Since we want to write 26 elements and a path can only contain
         * tree.height * BUCKET_SIZE = 6 * 4 (here) = 24 elements, the stash has
         * to be not empty. However, its size can vary since we assigned paths
         * at random.
         */
        let mut ordered_elements = client.order_elements_for_writing(
            &mut [new_values.as_slice(), read_data.as_slice()].concat(),
            path,
            path_oram.tree().height() as usize,
        );

        assert!(!client.stash.is_empty());

        assert_eq!(ordered_elements.len(), path_oram.tree().height() as usize);
        assert_eq!(ordered_elements[0].len(), BUCKET_SIZE);

        // Encrypt read items to write them back to the ORAM.
        let enc_res = client.encrypt_items(&mut ordered_elements);
        assert!(enc_res.is_ok());

        // Encrypt back the stash.
        let stsh_enc_res = client.encrypt_stash();
        assert!(stsh_enc_res.is_ok());

        /*
         * Sending data to the server for writing...
         */

        // Write ordered elements to the same path.
        let res_write = path_oram.access(
            AccessType::Write,
            path,
            Some(&mut ordered_elements),
        );

        assert!(res_write.is_ok());
        let opt_write = res_write.unwrap();
        assert!(opt_write.is_none());

        /*
         * Let's read the same path again to check if values read are the same.
         */
        let res_read = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_read.is_ok());
        let opt_read = res_read.unwrap();
        assert!(opt_read.is_some());

        let mut read_values = opt_read.unwrap();

        let dec_res = client.decrypt_items(&mut read_values);
        assert!(dec_res.is_ok());

        // Check that the path we read from contains the witness we inserted.
        read_values
            .iter()
            .any(|data_item| data_item.data() == &witness);
    }
}
