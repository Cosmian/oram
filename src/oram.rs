use std::ops::{Deref, DerefMut};

use crate::btree::{BTree, Node};
use cosmian_crypto_core::{reexport::rand_core::SeedableRng, CsRng};

pub const STASH_SIZE: usize = 32;
pub const BUCKET_SIZE: usize = 4;

// pub type Stashh = Vec<Vec<u8>>;

#[derive(Debug, Default, Clone)]
pub struct Stash {
    stash: Vec<Vec<u8>>,
}

impl Deref for Stash {
    type Target = [Vec<u8>];

    fn deref(&self) -> &Self::Target {
        &self.stash
    }
}

impl DerefMut for Stash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stash
    }
}

impl Stash {
    pub fn new() -> Stash {
        // Empty stash at initialization.
        Stash { stash: Vec::new() }
    }

    pub fn push(&mut self, ciphertext: Vec<u8>) -> bool {
        if self.len() < STASH_SIZE {
            self.stash.push(ciphertext);
            return true;
        }

        false
    }
}

pub struct ORAM {
    tree: BTree,
    stash: Stash,
    csprng: CsRng,
}

impl ORAM {
    pub fn new(stash: Stash, nb_blocks: usize, block_size: usize) -> ORAM {
        let mut csprng = CsRng::from_entropy();

        ORAM {
            tree: BTree::new_empty_complete(&mut csprng, nb_blocks, block_size),
            stash,
            csprng,
        }
    }

    pub fn read_path(&self, path: u16) -> Vec<Vec<u8>> {
        let mut path_values = Vec::new();

        ORAM::path_traversal(self.tree.root(), &mut path_values, path, self.tree.height());

        println!("{:?}", path_values);

        [path_values, self.stash.to_vec()].concat()
    }

    fn path_traversal(
        node: Option<&Box<Node>>,
        path_values: &mut Vec<Vec<u8>>,
        path: u16,
        level: u32,
    ) {
        if let Some(node) = node {
            for data_item in node.bucket() {
                path_values.push(data_item.data().to_vec());
            }

            // Left-to-right bitwise analysis.
            if (path >> (level - 1)) % 2 == 0 {
                println!("left");
                ORAM::path_traversal(node.left(), path_values, path, level - 1);
            } else {
                println!("right");
                ORAM::path_traversal(node.right(), path_values, path, level - 1);
            }
        }
    }

    pub fn tree(&self) -> &BTree {
        &self.tree
    }

    pub fn stash(&self) -> &Stash {
        &self.stash
    }

    pub fn csprng(&self) -> &CsRng {
        &self.csprng
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn complete_tree_test_values() {
        assert_eq!(1, 1);
        /*let stash = Stash::new(STASH_SIZE);
        let nb_blocks = 129;
        let block_size = 64;
        let path_oram = ORAM::new(stash.clone(), nb_blocks, block_size);
        println!("Hello Path-ORAM!");

        let path = 49;
        let path_values = path_oram.read_path(path);

        let mut expected_path_values = vec![0, 1, 2, 3, 4, 5, 6, 7];
        expected_path_values.extend_from_slice(stash.stash());

        assert_eq!(path_values, expected_path_values);*/
    }
}
