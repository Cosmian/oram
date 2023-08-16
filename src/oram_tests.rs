#[cfg(test)]
mod tests {
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

        let encryption_res =
            client.encrypt_items(&mut path_values, [6].to_vec());
        assert!(encryption_res.is_ok());

        assert_ne!(path_values.len(), 0);

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(path_values).as_mut(),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_some());
        let stash = path_values_opt.unwrap();

        // Path-Oram success.
        assert!(
            stash
                .iter()
                .filter(|remnant| !remnant.data().is_empty())
                .collect::<Vec<_>>()
                .len()
                < path_oram.tree().height() as usize * BUCKET_SIZE
        );
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

        let encryption_res =
            client.encrypt_items(&mut path_values, [].to_vec());
        assert!(encryption_res.is_ok());

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(path_values).as_mut(),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_some());
        let stash = path_values_opt.unwrap();

        // Path-Oram success.
        assert!(
            stash
                .iter()
                .filter(|remnant| !remnant.data().is_empty())
                .collect::<Vec<_>>()
                .len()
                < path_oram.tree().height() as usize * BUCKET_SIZE
        );
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

        let encryption_res =
            client.encrypt_items(&mut path_values, [0, 3, 6, 9].to_vec());
        assert!(encryption_res.is_ok());

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(path_values).as_mut(),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_some());
        let stash = path_values_opt.unwrap();

        // Path-Oram success.
        assert!(
            stash
                .iter()
                .filter(|remnant| !remnant.data().is_empty())
                .collect::<Vec<_>>()
                .len()
                < path_oram.tree().height() as usize * BUCKET_SIZE
        );
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

        let number_of_items_read = path_values.len();
        let encryption_res = client.encrypt_items(
            &mut path_values,
            (0..number_of_items_read).into_iter().collect(),
        );
        assert!(encryption_res.is_ok());

        let res_access = path_oram.access(
            AccessType::Write,
            path,
            Some(path_values.as_mut()),
        );

        assert!(res_access.is_ok());
        let path_values_opt = res_access.unwrap();
        assert!(path_values_opt.is_some());
        let stash = path_values_opt.unwrap();

        // Path-Oram success.
        assert!(
            stash
                .iter()
                .filter(|remnant| !remnant.data().is_empty())
                .collect::<Vec<_>>()
                .len()
                < path_oram.tree().height() as usize * BUCKET_SIZE
        );
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
    fn client_decrypt_encrypt_items() {
        let nb_items: usize = 15;
        let ct_size = 16;
        let mut client = ClientOram::new(nb_items);

        let dummies_res = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_res.is_ok());
        let mut dummies = dummies_res.unwrap();

        let decryption_res = client.decrypt_items(&mut dummies);
        assert!(decryption_res.is_ok());

        // Get witness of plaintext dummies.
        let mut witness = Vec::new();
        dummies.iter().for_each(|e| witness.push(e.clone()));

        let encryption_res = client.encrypt_items(&mut dummies, [].to_vec());
        assert!(encryption_res.is_ok());

        assert_ne!(dummies[0].data(), witness[0].data());

        let decryption_res = client.decrypt_items(&mut dummies);
        assert!(decryption_res.is_ok());

        assert_eq!(dummies[0].data(), witness[0].data());
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
         * Example of use for 44 items stored and a ciphertext size of 16 bytes.
         */
        let nb_items: usize = 44;
        let ct_size: usize = 16;

        /*
         * Client.
         */
        let mut client = ClientOram::new(nb_items);

        let dummies_result = client.generate_dummy_items(nb_items, ct_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let mut path_oram = Oram::new(dummies.as_mut(), nb_items).unwrap();

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

        let encrypt_res = client.encrypt_items(&mut path_values, [6].to_vec());
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
            Some(&mut [path_values, client.stash].concat()),
        );

        assert!(path_values_remnants_res.is_ok());
        let path_values_remnants_opt = path_values_remnants_res.unwrap();

        assert!(path_values_remnants_opt.is_some());
        let path_values_remnants = path_values_remnants_opt.unwrap();

        /*
         * Server sends back remnants items it could not load into the tree.
         * They constitute the new stash.
         */

        client.stash = path_values_remnants;

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
