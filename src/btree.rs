use crate::oram::BUCKET_SIZE;
use cosmian_crypto_core::reexport::rand_core::CryptoRngCore;

#[derive(Debug, Clone, Default)]
pub struct BTree {
    pub root: Option<Box<Node>>,
    nb_blocks: usize,
    block_bytes_size: usize,
    height: u32,
}

impl BTree {
    pub fn new_random_complete<CSPRNG: CryptoRngCore>(
        csprng: &mut CSPRNG,
        nb_blocks: usize,
        block_bytes_size: usize,
    ) -> BTree {
        let mut height = nb_blocks.ilog2();
        if !nb_blocks.is_power_of_two() {
            height += 1;
        }

        let mut tree = BTree {
            root: Option::None,
            nb_blocks,
            height,
            block_bytes_size,
        };

        let mut root = Node::new(csprng, block_bytes_size);

        BTree::complete_tree(
            csprng,
            &mut root,
            tree.height,
            0,
            block_bytes_size,
        );
        tree.root = Some(Box::new(root));
        tree
    }

    fn complete_tree<CSPRNG: CryptoRngCore>(
        csprng: &mut CSPRNG,
        node: &mut Node,
        height: u32,
        level: u32,
        block_bytes_size: usize,
    ) {
        // -1 is to avoid constructing 1 extra level.
        if level < height - 1 {
            let mut left: Box<Node> =
                Box::new(Node::new(csprng, block_bytes_size));
            let mut right = Box::new(Node::new(csprng, block_bytes_size));

            BTree::complete_tree(
                csprng,
                &mut left,
                height,
                level + 1,
                block_bytes_size,
            );
            BTree::complete_tree(
                csprng,
                &mut right,
                height,
                level + 1,
                block_bytes_size,
            );

            node.left = Some(left);
            node.right = Some(right);
        }
    }

    pub fn nb_blocks(&self) -> usize {
        self.nb_blocks
    }

    pub fn block_bytes_size(&self) -> usize {
        self.block_bytes_size
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

/* Bucket ciphertexts are not size bounded. Might want to fix this later. */
#[derive(Debug, Clone, Default)]
pub struct Node {
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
    bucket: [DataItem; BUCKET_SIZE],
}

impl Node {
    fn new<CSPRNG: CryptoRngCore>(
        csprng: &mut CSPRNG,
        block_bytes_size: usize,
    ) -> Node {
        let bucket = [
            DataItem::new_random(csprng, block_bytes_size),
            DataItem::new_random(csprng, block_bytes_size),
            DataItem::new_random(csprng, block_bytes_size),
            DataItem::new_random(csprng, block_bytes_size),
        ];

        Node {
            left: Option::None,
            right: Option::None,
            bucket,
        }
    }

    pub fn bucket(&self) -> &[DataItem; BUCKET_SIZE] {
        &self.bucket
    }

    pub fn set_bucket_element(&mut self, elt: DataItem, i: usize) {
        self.bucket[i] = elt;
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataItem {
    data: Vec<u8>,
    path: u16,
}

impl DataItem {
    pub fn new(data: Vec<u8>, path: u16) -> DataItem {
        DataItem { data, path }
    }

    fn new_random<CSPRNG: CryptoRngCore>(
        csprng: &mut CSPRNG,
        block_bytes_size: usize,
    ) -> DataItem {
        let mut data = vec![0; block_bytes_size];
        csprng.fill_bytes(&mut data);

        // XXX - Not oblivious to put all with path 0: can distinguish dummies.
        DataItem { data, path: 0 }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn path(&self) -> u16 {
        self.path
    }
}
