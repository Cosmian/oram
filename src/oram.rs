use crate::btree::{BTree, DataItem, Node};
use cosmian_crypto_core::{reexport::rand_core::SeedableRng, CsRng};
use std::ops::{Deref, DerefMut};

pub const BUCKET_SIZE: usize = 4;

// pub type Stashh = Vec<Vec<u8>>;

pub enum AccessType {
    Read,
    Write,
}

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
}

pub struct ORAM {
    tree: BTree,
    stash: Stash,
}

impl ORAM {
    pub fn new(stash: Stash, nb_blocks: usize, block_size: usize) -> ORAM {
        let mut csprng = CsRng::from_entropy();

        ORAM {
            tree: BTree::new_random_complete(
                &mut csprng,
                nb_blocks,
                block_size,
            ),
            stash,
        }
    }

    pub fn access(
        &mut self,
        op: AccessType,
        path: u16,
        data: Option<&mut Vec<Vec<u8>>>,
    ) -> Option<Vec<Vec<u8>>> {
        let mut path_data = Vec::new();

        ORAM::read_path(
            self.tree.root.as_ref(),
            &mut path_data,
            path,
            self.tree.height(),
        );

        match op {
            AccessType::Read => {
                // Returning values from tree and stash if there are any.
                Some([path_data, self.stash.to_vec()].concat())
            }
            AccessType::Write => {
                // TODO
                if let Some(data) = data {
                    // XXX - TODO - OUCH FIXMEEEEEE.
                    let tree_height = self.tree.height();
                    ORAM::write_path(
                        self.tree.root.as_mut(),
                        data,
                        path,
                        tree_height,
                    )
                    // TODO: write data remnants to stash.
                }
                None
            }
        }
    }

    fn read_path(
        node: Option<&Box<Node>>,
        path_data: &mut Vec<Vec<u8>>,
        path: u16,
        level: u32,
    ) {
        // Check if not out of the binary tree.
        if let Some(node) = node {
            node.bucket().iter().for_each(|data_item| {
                // Only add the element if it belongs to our path.
                if data_item.path() == path {
                    path_data.push(data_item.data().to_vec());
                }
            });

            // Left-to-right bitwise analysis.
            if (path >> (level - 1)) % 2 == 0 {
                println!("left");
                ORAM::read_path(node.left.as_ref(), path_data, path, level - 1);

                // shall we collapse values here ?
            } else {
                println!("right");
                ORAM::read_path(
                    node.right.as_ref(),
                    path_data,
                    path,
                    level - 1,
                );
            }
        }
    }

    fn write_path(
        node: Option<&mut Box<Node>>,
        path_data: &mut Vec<Vec<u8>>,
        path: u16,
        level: u32,
    ) {
        // Check if not out of the binary tree.
        if let Some(node) = node {
            // Left-to-right bitwise analysis.
            if (path >> (level - 1)) % 2 == 0 {
                ORAM::write_path(
                    node.left.as_mut(),
                    path_data,
                    path,
                    level - 1,
                );
            } else {
                ORAM::write_path(
                    node.right.as_mut(),
                    path_data,
                    path,
                    level - 1,
                );
            }

            // one-liner possible ?
            for i in 0..BUCKET_SIZE {
                if let Some(data) = path_data.pop() {
                    // TODO: check which element to overwrite ? or ok ?
                    if node.bucket()[i].path() == path {
                        node.set_bucket_element(DataItem::new(data, path), i);
                    }
                }
            }
        }
    }

    pub fn tree(&self) -> &BTree {
        &self.tree
    }

    pub fn stash(&self) -> &Stash {
        &self.stash
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn complete_tree_test_values() {
        assert_eq!(1, 1);
    }
}
