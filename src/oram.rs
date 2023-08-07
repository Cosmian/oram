use crate::btree::{BTree, Node};

#[derive(Debug, Default, Clone)]
pub struct Stash {
    stash: Vec<u32>,
}

impl Stash {
    pub fn new(size: usize) -> Stash {
        Stash {
            stash: vec![0; size],
        }
    }

    pub fn stash(&self) -> &Vec<u32> {
        &self.stash
    }
}

pub struct ORAM {
    tree: BTree,
    stash: Stash,
}

impl ORAM {
    pub fn new(stash: Stash, nb_blocks: usize, block_size: usize) -> ORAM {
        ORAM {
            tree: BTree::new_empty_complete(nb_blocks, block_size),
            stash,
        }
    }

    pub fn read_path(&self, path: u16) -> Vec<u32> {
        let mut path_values = Vec::new();

        ORAM::path_traversal(self.tree.root(), &mut path_values, path, self.tree.height());

        path_values.extend_from_slice(self.stash.stash());
        path_values
    }

    fn path_traversal(node: Option<&Box<Node>>, path_values: &mut Vec<u32>, path: u16, level: u32) {
        if let Some(node) = node {
            path_values.push(node.value());

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
}
