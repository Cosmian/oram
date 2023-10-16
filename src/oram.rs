use crate::btree::{BTree, DataItem, Node};
use std::io::{Error, ErrorKind};

pub const BUCKET_SIZE: usize = 4;

pub enum AccessType {
    Read,
    Write(Vec<[DataItem; BUCKET_SIZE]>),
}

pub struct Oram {
    tree: BTree,
}

impl Oram {
    pub fn new(mut data_items: Vec<DataItem>, nb_items: usize) -> Result<Oram, Error> {
        if nb_items == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Number of items shall not be null".to_string(),
            ));
        }

        Ok(Oram {
            tree: BTree::init_new(&mut data_items, nb_items),
        })
    }

    pub fn access(&mut self, op: AccessType, path: usize) -> Result<Option<Vec<DataItem>>, Error> {
        if path > (1 << (self.tree.height() - 1)) - 1 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid path access. Got {}, expected in range 0..{}",
                    path,
                    (1 << (self.tree.height() - 1)) - 1
                ),
            ));
        }

        match op {
            AccessType::Read => {
                Ok(self.tree.root.as_ref().map(|root| {
                    // TODO (TBZ): Allocate the correct amount of memory or return it as a result
                    // from `read_path`.
                    let mut path_items = Vec::new();
                    Oram::read_path(root, &mut path_items, path, self.tree.height(), 0);
                    path_items
                }))
            }
            AccessType::Write(mut data) => {
                let height = self.tree.height();
                if let Some(root) = self.tree.root.as_mut() {
                    Oram::write(root, &mut data, path, height, 0);
                }
                Ok(None)
            }
        }
    }

    /// Determine if the next node to select is the left one.
    fn get_is_next_left(path: usize, level: u16, height: u16) -> bool {
        // Notice that:
        // - `height - 1` is used since `level < height`;
        // - `level + 1` is used since the information concerns the next level.
        let shift = ((height - 1) as i16 - (level + 1) as i16).max(0);
        (path >> shift) % 2 == 0
    }

    fn read_path(node: &Node, data: &mut Vec<DataItem>, path: usize, height: u16, level: u16) {
        data.extend(node.slots().cloned());

        let next_node = if Self::get_is_next_left(path, level, height) {
            node.left.as_ref()
        } else {
            node.right.as_ref()
        };

        if let Some(node) = next_node {
            Oram::read_path(node, data, path, height, level + 1);
        }
    }

    fn write(
        node: &mut Box<Node>,
        data: &mut Vec<[DataItem; BUCKET_SIZE]>,
        path: usize,
        height: u16,
        level: u16,
    ) {
        let next_node = if Self::get_is_next_left(path, level, height) {
            node.left.as_mut()
        } else {
            node.right.as_mut()
        };

        if let Some(node) = next_node {
            Oram::write(node, data, path, height, level + 1);
        }

        if let Some(new_bucket) = data.pop() {
            node.set_bucket(new_bucket);
        }
    }

    pub fn tree(&self) -> &BTree {
        &self.tree
    }
}
