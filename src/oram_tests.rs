#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::{
        btree::{DataItem, Node},
        oram::{AccessType, BUCKET_SIZE, ORAM},
        CIPHERTEXTS,
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
}
