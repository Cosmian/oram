#[derive(Debug, Clone, Default)]
pub struct BTree {
    root: Option<Box<Node>>,
    nb_blocks: usize,
    block_size: usize,
    height: u32,
}

impl BTree {
    pub fn new_empty_complete(nb_blocks: usize, block_size: usize) -> BTree {
        let mut height = nb_blocks.ilog2();
        if !nb_blocks.is_power_of_two() {
            height += 1;
        }

        let mut tree = BTree {
            root: Option::None,
            nb_blocks,
            height,
            block_size,
        };

        let mut root = Node::default();

        BTree::complete_tree(&mut root, tree.height, 0);
        tree.root = Some(Box::new(root));
        tree
    }

    fn complete_tree(node: &mut Node, height: u32, level: u32) {
        // -1 is to avoid constructing 1 extra level.
        if level < height - 1 {
            let mut left = Box::new(Node::new(level + 1));
            let mut right = Box::new(Node::new(level + 1));

            BTree::complete_tree(&mut left, height, level + 1);
            BTree::complete_tree(&mut right, height, level + 1);

            node.left = Some(left);
            node.right = Some(right);
        }
    }

    pub fn root(&self) -> Option<&Box<Node>> {
        self.root.as_ref()
    }

    pub fn nb_blocks(&self) -> usize {
        self.nb_blocks
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    value: u32, // change to tuple (value, path) later.
}

impl Node {
    fn new(value: u32) -> Node {
        Node {
            left: Option::None,
            right: Option::None,
            value,
        }
    }

    pub fn left(&self) -> Option<&Box<Node>> {
        self.left.as_ref()
    }

    pub fn right(&self) -> Option<&Box<Node>> {
        self.right.as_ref()
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}
