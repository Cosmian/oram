use crate::btree;

struct Stash {
    size: u32,
    stash: Vec<u32>,
}

impl Stash {
    fn new(size_: u32) -> Stash {
        Stash {
            size: size_,
            stash: vec![0; size_.try_into().unwrap()],
        }
    }
}

pub struct ORAM {
    tree: btree::BTree,
    stash: Stash,
    nb_blocks: u64,
    stash_size: u32,
    block_size: u16,
}

impl ORAM {
    pub fn new(nb_blocks_: u64, stash_size_: u32, block_size_: u16) -> ORAM {
        ORAM {
            tree: btree::BTree::new_empty_complete(nb_blocks_, block_size_),
            stash: Stash::new(stash_size_),
            nb_blocks: nb_blocks_,
            stash_size: stash_size_,
            block_size: block_size_,
        }
    }

    pub fn get_nb_blocks(self) -> u64 {
        self.nb_blocks
    }

    pub fn get_tree(self) -> btree::BTree {
        self.tree
    }

    //pub fn get_stash(self) -> Stash {
    //    self.stash
    //}
}

pub fn path_traversal(node: Option<Box<btree::Node>>, path: u16, level: u32) {
    if let Some(node) = node {
        // Left-to-right bitwise analysis.
        if (path >> (level - 1)) % 2 == 0 {
            println!("left");
            path_traversal((*node).left(), path, level - 1);
        } else {
            println!("right");
            path_traversal((*node).right(), path, level - 1);
        }
    }
}
