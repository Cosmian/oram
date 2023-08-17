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
         * After decryption, change item number 6.
         */
        let decryption_res = client.decrypt_items(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let mut csprng = CsRng::from_entropy();
        let mut new_value = vec![0; 3];
        csprng.fill_bytes(&mut new_value);

        let new_item = DataItem::new(new_value.clone());
        client.position_map.insert(new_value, 0);

        let chg_res = client.change_element_position(&new_item);
        assert!(chg_res.is_ok());

        let mut ordered_elements = client.order_elements_for_writing(
            &[path_values.as_slice(), &[new_item]].concat(),
            path,
            path_oram.tree().height() as usize,
        );

        assert_eq!(ordered_elements[0].len(), 1);
        assert_eq!(ordered_elements.len(), 4);

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

        let mut ordered_elements = client.order_elements_for_writing(
            &path_values,
            path,
            path_oram.tree().height() as usize,
        );

        assert_eq!(ordered_elements[0].len(), 0);
        assert_eq!(ordered_elements.len(), 4);

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

        let mut ordered_elements = client.order_elements_for_writing(
            &[path_values.as_slice(), new_values.as_slice()].concat(),
            path,
            path_oram.tree().height() as usize,
        );

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
         * After decryption, RESTORE EVERYTHING.
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
            &[path_values.as_slice(), new_values.as_slice()].concat(),
            path,
            path_oram.tree().height() as usize,
        );

        let encryption_res = client.encrypt_items(&mut ordered_elements);
        assert!(encryption_res.is_ok());

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(&mut ordered_elements),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_some());
        //let stash = path_values_opt.unwrap();
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
            &path_values_decrypted,
            path,
            tree_height,
        );

        assert_eq!(ordered_buckets.len(), tree_height);
        assert_eq!(ordered_buckets[0].len(), 4);
        assert_eq!(ordered_buckets[1].len(), 4);
        assert_eq!(ordered_buckets[2].len(), 1);
        assert_eq!(ordered_buckets[3].len(), 2);

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
         * Example of use for 18 items stored and a ciphertext size of 16 bytes.
         */
        let nb_items: usize = 18;
        let ct_size: usize = 16;

        /*
         * Client.
         */
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

        let values = vec![
            DataItem::new(elt1),
            DataItem::new(elt2),
            DataItem::new(elt3),
            DataItem::new(elt4),
            DataItem::new(elt5),
            DataItem::new(elt6),
            DataItem::new(elt7),
            DataItem::new(elt8),
            DataItem::new(elt9),
            DataItem::new(elt10),
            DataItem::new(elt11),
        ];

        let dummies_result =
            client.generate_dummy_items(nb_items - 12, ct_size);

        assert!(dummies_result.is_ok());
        let dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let mut path_oram = Oram::new(
            &mut [values.as_slice(), dummies.as_slice()].concat(),
            nb_items,
        )
        .unwrap();

        // Let's read path 3, of all 16 paths from 0 to 15 included.
        let path = 3;

        let path_values_result =
            path_oram.access(AccessType::Read, path, Option::None);

        assert!(path_values_result.is_ok());
        let path_values_option = path_values_result.unwrap();

        assert!(path_values_option.is_some());
        let mut path_values = path_values_option.unwrap();

        /*
         * Client side now.
         * After decryption, change item number 6.
         */
        let decrypt_res = client.decrypt_items(&mut path_values);
        assert!(decrypt_res.is_ok());

        // Here path_values is a vector containing plaintexts.
        let res_chg = client.change_element_position(&path_values[6]);
        assert!(res_chg.is_ok());

        let mut ordered_elements = client.order_elements_for_writing(
            &path_values,
            path,
            path_oram.tree().height() as usize,
        );

        let encrypt_res = client.encrypt_items(&mut ordered_elements);
        assert!(encrypt_res.is_ok());

        /*
         * Each Read operation **must** be followed by a write operation on the
         * same path. Client sends new DataItems to write to the tree alongside
         * with his current stash.
         * Server side.
         */
        let path_values_remnants_res = path_oram.access(
            AccessType::Write,
            path,
            Some(&mut ordered_elements),
        );

        assert!(path_values_remnants_res.is_ok());
        let path_values_remnants_opt = path_values_remnants_res.unwrap();

        assert!(path_values_remnants_opt.is_some());
        let stash = path_values_remnants_opt.unwrap();

        /*
         * Server sends back remnants items it could not load into the tree.
         * They constitute the new stash.
         */

        client.stash = stash;

        // Path-Oram success.
        assert!(
            client
                .stash
                .iter()
                .filter(|remnant| !remnant.data().is_empty())
                .collect::<Vec<_>>()
                .len()
                < path_oram.tree().height() as usize * BUCKET_SIZE
        );
    }
}
