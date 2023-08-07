use cosmian_crypto_core::reexport::rand_core::CryptoRngCore;

use crate::oram::BUCKET_SIZE;

#[derive(Debug, Clone, Default)]
pub struct BTree {
    root: Option<Box<Node>>,
    nb_blocks: usize,
    block_size: usize,
    height: u32,
}

impl BTree {
    pub fn new_empty_complete<CSPRNG: CryptoRngCore>(
        csprng: &mut CSPRNG,
        nb_blocks: usize,
        block_size: usize,
    ) -> BTree {
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

        let mut root = Node::new(csprng);

        BTree::complete_tree(csprng, &mut root, tree.height, 0);
        tree.root = Some(Box::new(root));
        tree
    }

    fn complete_tree<CSPRNG: CryptoRngCore>(
        csprng: &mut CSPRNG,
        node: &mut Node,
        height: u32,
        level: u32,
    ) {
        // -1 is to avoid constructing 1 extra level.
        if level < height - 1 {
            let mut left = Box::new(Node::new(csprng));
            let mut right = Box::new(Node::new(csprng));

            BTree::complete_tree(csprng, &mut left, height, level + 1);
            BTree::complete_tree(csprng, &mut right, height, level + 1);

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

/* Bucket ciphertexts are not size bounded. Might want to fix this later. */
#[derive(Debug, Clone, Default)]
pub struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    bucket: [DataItem; BUCKET_SIZE],
}

impl Node {
    fn new<CSPRNG: CryptoRngCore>(csprng: &mut CSPRNG) -> Node {
        let bucket = [
            DataItem::new_random(csprng),
            DataItem::new_random(csprng),
            DataItem::new_random(csprng),
            DataItem::new_random(csprng),
        ];

        Node {
            left: Option::None,
            right: Option::None,
            bucket,
        }
    }

    pub fn left(&self) -> Option<&Box<Node>> {
        self.left.as_ref()
    }

    pub fn right(&self) -> Option<&Box<Node>> {
        self.right.as_ref()
    }

    pub fn bucket(&self) -> &[DataItem; BUCKET_SIZE] {
        &self.bucket
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataItem {
    data: Vec<u8>,
    path: u16,
}

impl DataItem {
    fn new(data: Vec<u8>, path: u16) -> DataItem {
        DataItem { data, path }
    }

    fn new_random<CSPRNG: CryptoRngCore>(csprng: &mut CSPRNG) -> DataItem {
        let mut data = vec![0; 16];
        csprng.fill_bytes(&mut data);

        DataItem { data, path: 0 }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn path(&self) -> u16 {
        self.path
    }

    pub fn set_path(&mut self, path: u16) {
        self.path = path;
    }
}
