#[cfg(test)]
mod tests {
    use crate::{
        btree::Node,
        client::ClientORAM,
        oram::{AccessType, BUCKET_SIZE, ORAM},
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
        let nb_blocks: usize = 0;

        let path_oram = ORAM::new(&mut Vec::new(), nb_blocks);
        assert!(path_oram.is_err());
    }

    #[test]
    fn complete_tree_size_one() {
        let nb_blocks: usize = 1;

        let path_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, 1);
        }
    }

    #[test]
    fn complete_tree_size_pow_of_2() {
        let nb_blocks: usize = 32;

        let path_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, 63);
        }
    }

    #[test]
    fn complete_tree_size_exact() {
        let nb_blocks: usize = 15;

        let path_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, nb_blocks);
        }
    }

    #[test]
    fn complete_tree_size_rand() {
        let nb_blocks: usize = 26;

        let path_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        if let Ok(path_oram) = path_oram {
            let tree_size = _complete_tree_size(path_oram.tree().root.as_ref());
            assert_eq!(tree_size, 31);
        }
    }

    #[test]
    fn access_bad_path() {
        let nb_blocks: usize = 15;

        let path_oram = ORAM::new(&mut Vec::new(), nb_blocks);

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
        let nb_blocks: usize = 15;

        let path_oram = ORAM::new(&mut Vec::new(), nb_blocks);

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
        let nb_blocks: usize = 15;

        let res_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 3;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_access.is_ok());
    }

    #[test]
    fn access_valid_path2() {
        let nb_blocks: usize = 15;

        let res_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 7;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_access.is_ok());
    }

    #[test]
    fn access_valid_path3() {
        let nb_blocks: usize = 15;

        let res_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 0;
        let res_access = path_oram.access(AccessType::Read, path, Option::None);
        assert!(res_access.is_ok());
    }

    #[test]
    fn access_valid_path_write_data_none() {
        let nb_blocks: usize = 15;

        let res_oram = ORAM::new(&mut Vec::new(), nb_blocks);

        assert!(res_oram.is_ok());
        let mut path_oram = res_oram.unwrap();

        let path = 0;
        let res_access =
            path_oram.access(AccessType::Write, path, Option::None);
        assert!(res_access.is_err());
    }

    #[test]
    fn read_write_access() {
        let nb_blocks: usize = 15;
        let nb_leaves = (1 << (nb_blocks.ilog2() + 1)) / 2;
        let block_size: usize = 16;

        let mut client = ClientORAM::new();

        let dummies_result = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = ORAM::new(&mut dummies, nb_blocks);

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
         * After decryption, change block number 6.
         */
        let decryption_res = client.decrypt_blocks(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let encryption_res =
            client.encrypt_blocks(&mut path_values, [6].to_vec(), nb_leaves);
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
        let nb_blocks: usize = 15;
        let nb_leaves = (1 << (nb_blocks.ilog2() + 1)) / 2;
        let block_size: usize = 16;

        let mut client = ClientORAM::new();

        let dummies_result = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = ORAM::new(&mut dummies, nb_blocks);

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
        let decryption_res = client.decrypt_blocks(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let encryption_res =
            client.encrypt_blocks(&mut path_values, [].to_vec(), nb_leaves);
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
        let nb_blocks: usize = 15;
        let nb_leaves = (1 << (nb_blocks.ilog2() + 1)) / 2;
        let block_size: usize = 16;

        let mut client = ClientORAM::new();

        let dummies_result = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = ORAM::new(&mut dummies, nb_blocks);

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
         * After decryption, change block number 0, 3, 6 and 9.
         */
        let decryption_res = client.decrypt_blocks(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let encryption_res = client.encrypt_blocks(
            &mut path_values,
            [0, 3, 6, 9].to_vec(),
            nb_leaves,
        );
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
        let nb_blocks: usize = 15;
        let nb_leaves = (1 << (nb_blocks.ilog2() + 1)) / 2;
        let block_size: usize = 16;

        let mut client = ClientORAM::new();

        let dummies_result = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let res_oram = ORAM::new(&mut dummies, nb_blocks);

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
        let decryption_res = client.decrypt_blocks(&mut path_values);
        assert!(decryption_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let number_of_blocks_read = path_values.len();
        let encryption_res = client.encrypt_blocks(
            &mut path_values,
            (0..number_of_blocks_read).into_iter().collect(),
            nb_leaves,
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
        let nb_blocks = 0;
        let block_size = 16;
        let mut client = ClientORAM::new();

        let dummies_res = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_blocks);
    }

    #[test]
    fn generate_dummies_bad_number() {
        let nb_blocks = 173;
        let block_size = 16;
        let mut client = ClientORAM::new();

        let dummies_res = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_res.is_err());
    }

    #[test]
    fn generate_dummies_small() {
        let nb_blocks = 15;
        let block_size = 16;
        let mut client = ClientORAM::new();

        let dummies_res = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_blocks * BUCKET_SIZE);
    }

    #[test]
    fn generate_dummies_big() {
        let nb_blocks = (1 << 11) - 1;
        let block_size = 16;
        let mut client = ClientORAM::new();

        let dummies_res = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_blocks * BUCKET_SIZE);
    }

    #[test]
    fn generate_dummies_null_block_size() {
        let nb_blocks = 15;
        let block_size = 0;
        let mut client = ClientORAM::new();

        let dummies_res = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_blocks * BUCKET_SIZE);
        // Nonce + tag length.
        assert_eq!(dummies[0].data().len(), 12 + 16);
    }

    #[test]
    fn generate_dummies_tremendous_block_size() {
        let nb_blocks = 15;
        let block_size = 1000;
        let mut client = ClientORAM::new();

        let dummies_res = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_res.is_ok());
        let dummies = dummies_res.unwrap();
        assert_eq!(dummies.len(), nb_blocks * BUCKET_SIZE);
        // Nonce + tag length.
        assert_eq!(dummies[0].data().len(), block_size + 12 + 16);
    }

    #[test]
    fn client_decrypt_encrypt_blocks() {
        let nb_blocks: usize = 15;
        let nb_leaves = (1 << (nb_blocks.ilog2() + 1)) / 2;
        let block_size = 16;
        let mut client = ClientORAM::new();

        let dummies_res = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_res.is_ok());
        let mut dummies = dummies_res.unwrap();

        let decryption_res = client.decrypt_blocks(&mut dummies);
        assert!(decryption_res.is_ok());

        // Get witness of plaintext dummies.
        let mut witness = Vec::new();
        dummies.iter().for_each(|e| witness.push(e.clone()));

        let encryption_res =
            client.encrypt_blocks(&mut dummies, [].to_vec(), nb_leaves);
        assert!(encryption_res.is_ok());

        assert_ne!(dummies[0].data(), witness[0].data());

        let decryption_res = client.decrypt_blocks(&mut dummies);
        assert!(decryption_res.is_ok());

        assert_eq!(dummies[0].data(), witness[0].data());
    }

    #[test]
    fn general_behavior() {
        /*
         * Example of use for 15 items stored and a ciphertext size of 16 bytes.
         */
        let nb_blocks: usize = 15;
        let nb_leaves = (nb_blocks + 1) / 2;
        let block_size: usize = 16;

        /*
         * Client.
         */
        let mut client = ClientORAM::new();

        let dummies_result = client.generate_dummies(nb_blocks, block_size);

        assert!(dummies_result.is_ok());
        let mut dummies = dummies_result.unwrap();

        /*
         * Server.
         */
        let mut path_oram = ORAM::new(dummies.as_mut(), nb_blocks).unwrap();

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
         * After decryption, change block number 6.
         */
        let decrypt_res = client.decrypt_blocks(&mut path_values);
        assert!(decrypt_res.is_ok());

        // Here path_values is a vector containing plaintexts.

        let encrypt_res =
            client.encrypt_blocks(&mut path_values, [6].to_vec(), nb_leaves);
        assert!(encrypt_res.is_ok());

        /*
         * Each Read operation **must** be followed by a write operation on the same
         * path. Client sends new DataItems to write to the tree alongside with his
         * current stash.
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
         * Server sends back remnants items it could not load into the tree. They
         * constitute the new stash.
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
