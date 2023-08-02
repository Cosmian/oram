pub struct BTree {
    root: Node,
    nb_blocks: u64,
    height: u32,
    block_size: u16,
}

impl BTree {
    pub fn new(nb_blocks_: u64, block_size_: u16) -> BTree {
        BTree {
            root: Node::new(0),
            nb_blocks: nb_blocks_,
            height: nb_blocks_.ilog2(), // todo rounding down bad
            block_size: block_size_,
        }
    }

    pub fn new_empty_complete(nb_blocks_: u64, block_size_: u16) -> BTree {
        let mut tree = BTree::new(nb_blocks_, block_size_);

        BTree::complete_tree(tree.height, 0, &mut tree.root);
        tree
    }

    fn complete_tree(max_height: u32, height: u32, node: &mut Node) {
        // -1 is to avoid constructing 1 extra level.
        if height < max_height - 1 {
            let mut left = Box::new(Node::new(height));
            let mut right = Box::new(Node::new(height));

            BTree::complete_tree(max_height, height + 1, &mut left);
            BTree::complete_tree(max_height, height + 1, &mut right);

            node.left = Some(left);
            node.right = Some(right);
        }
    }

    pub fn get_root(self) -> Node {
        self.root
    }
}

pub struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    value: u32, // change to tuple (value, path) later.
}

impl Node {
    fn new(value_: u32) -> Node {
        Node {
            left: Option::None,
            right: Option::None,
            value: value_,
        }
    }

    pub fn left(self) -> Option<Box<Node>> {
        self.left
    }

    pub fn right(self) -> Option<Box<Node>> {
        self.right
    }

    pub fn get_value(self) -> u32 {
        self.value
    }
}
