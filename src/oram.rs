use crate::btree::{BTree, DataItem, Node};
use std::io::{Error, ErrorKind};
use std::ops::{Deref, DerefMut};

pub const BUCKET_SIZE: usize = 4;

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
}

impl ORAM {
    pub fn new(
        dummies: &mut Vec<DataItem>,
        nb_blocks: usize,
    ) -> Result<ORAM, Error> {
        if nb_blocks == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Number of blocks shall not be null".to_string(),
            ));
        }
        Ok(ORAM {
            tree: BTree::init_new(dummies, nb_blocks),
        })
    }

    pub fn access(
        &mut self,
        op: AccessType,
        path: u16,
        data: Option<&mut Vec<DataItem>>,
    ) -> Result<Option<Vec<DataItem>>, Error> {
        if path > self.tree.height() - 1 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid path access. Got {}, expected in range 0..{}",
                    path,
                    self.tree.height() - 1
                ),
            ));
        }

        match op {
            AccessType::Read => {
                let mut path_items = Vec::new();

                ORAM::read_path(
                    self.tree.root.as_ref(),
                    &mut path_items,
                    path,
                    self.tree.height(),
                    0,
                );

                // Returning values from tree visit.
                Ok(Some(path_items))
            }
            AccessType::Write => {
                if let Some(data) = data {
                    let tree_height = self.tree.height();
                    ORAM::write_path(
                        self.tree.root.as_mut(),
                        data,
                        path,
                        tree_height,
                        0,
                    );

                    /*
                     * Remnants elements could not be stored in the tree, they
                     * represent the new stash (client-side).
                     */
                    return Ok(Some(data.to_vec()));
                }

                Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Invalid data to write. Got None, expected Some"
                        .to_string(),
                ))
            }
        }
    }

    fn read_path(
        node: Option<&Box<Node>>,
        path_data: &mut Vec<DataItem>,
        path: u16,
        height: u16,
        level: u16,
    ) {
        // Check if not out of the binary tree.
        if let Some(node) = node {
            // Push elements in the node in the vector.
            node.bucket().iter().for_each(|data_item| {
                path_data.push(data_item.clone());
            });

            // Left-to-right bitwise analysis. Substraction of 2 because one is for height being one more than path bit length.
            // The other one is because we want to see the bit corresponding to the next level.
            // check overflow here.
            let mut bit_shift = (height - level) as i16 - 2;
            if bit_shift < 0 {
                bit_shift = 0;
            }

            if (path >> bit_shift) % 2 == 0 {
                ORAM::read_path(
                    node.left.as_ref(),
                    path_data,
                    path,
                    height,
                    level + 1,
                );
            } else {
                ORAM::read_path(
                    node.right.as_ref(),
                    path_data,
                    path,
                    height,
                    level + 1,
                );
            }
        }
    }

    fn write_path(
        node: Option<&mut Box<Node>>,
        path_data: &mut Vec<DataItem>,
        path: u16,
        height: u16,
        level: u16,
    ) {
        // Check if not out of the binary tree.
        if let Some(node) = node {
            // Left-to-right bitwise analysis.

            /*
             * Left-to-right bitwise analysis. Substraction of 2 because one is
             * for height being one more than path bit length. The other one is
             * because we want to see the bit corresponding to the next level.
             * Below condition checks overflow.
             */
            let mut bit_shift = (height - level) as i16 - 2;
            if bit_shift < 0 {
                bit_shift = 0;
            }

            if (path >> bit_shift) % 2 == 0 {
                ORAM::write_path(
                    node.left.as_mut(),
                    path_data,
                    path,
                    height,
                    level + 1,
                );
            } else {
                ORAM::write_path(
                    node.right.as_mut(),
                    path_data,
                    path,
                    height,
                    level + 1,
                );
            }

            /*
             * Write element to the path. Right-side view method to greedily
             * fill the buckets. Elements can only be written on the path if
             * their new path is at an intersection with the old path.
             */
            // FIXME - one-liner possible ?
            for i in 0..BUCKET_SIZE {
                for j in 0..path_data.len() {
                    if path_data[j].path() >> (height - level)
                        == path >> (height - level)
                        && !path_data[j].data().is_empty()
                    {
                        // Remove element from vector once inserted.
                        node.set_bucket_element(path_data.remove(j), i);
                        break;
                    }
                }
            }
        }
    }

    pub fn tree(&self) -> &BTree {
        &self.tree
    }
}
