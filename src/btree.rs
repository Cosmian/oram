use crate::oram::BUCKET_SIZE;

/// XXX - ceiling method for division but usize may overflow.
pub fn udiv_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[derive(Debug, Clone, Default)]
pub struct BTree {
    pub(super) root: Option<Box<Node>>,
    height: u16,
}

impl BTree {
    pub fn init_new(data_items: &mut Vec<DataItem>, nb_items: usize) -> BTree {
        let mut tree = BTree {
            root: Option::None,
            height: udiv_ceil(nb_items, BUCKET_SIZE).ilog2() as u16 + 1,
        };

        let mut root = Node::new();

        tree.complete_tree(&mut root, data_items, 0);
        tree.root = Some(Box::new(root));

        tree
    }

    fn complete_tree(
        &self,
        node: &mut Node,
        data_items: &mut Vec<DataItem>,
        level: u16,
    ) {
        // -1 is to avoid constructing 1 extra level.
        if level < self.height - 1 {
            let mut left: Box<Node> = Box::new(Node::new());
            let mut right = Box::new(Node::new());

            self.complete_tree(&mut left, data_items, level + 1);
            self.complete_tree(&mut right, data_items, level + 1);

            node.left = Some(left);
            node.right = Some(right);
        }

        /*
         * Greedily filling buckets following a right side visit to fill leaves
         * first.
         */
        for i in 0..BUCKET_SIZE {
            // data_items must be a stack of items.
            if let Some(data_item) = data_items.pop() {
                node.set_bucket_element(data_item, i);
            }
        }
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub(crate) left: Option<Box<Node>>,
    pub(crate) right: Option<Box<Node>>,
    bucket: [DataItem; BUCKET_SIZE],
}

impl Node {
    fn new() -> Node {
        Node {
            left: Option::None,
            right: Option::None,
            bucket: [
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
                DataItem::default(),
            ],
        }
    }

    pub fn bucket(&self) -> &[DataItem; BUCKET_SIZE] {
        &self.bucket
    }

    pub fn set_bucket(&mut self, bucket: [DataItem; BUCKET_SIZE]) {
        self.bucket = bucket;
    }

    pub fn set_bucket_element(&mut self, elt: DataItem, i: usize) {
        self.bucket[i] = elt;
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataItem {
    data: Vec<u8>,
}

impl PartialEq for DataItem {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl DataItem {
    pub fn new(data: Vec<u8>) -> DataItem {
        DataItem { data }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn data_as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}
