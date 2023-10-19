use crate::oram::BUCKET_SIZE;
use std::slice::Iter;

/// Performs ceiling division on usize type.
pub fn udiv_ceil(a: usize, b: usize) -> usize {
    (a as f32 / b as f32).ceil() as usize
}

/// The size of a complete tree is the next power of two minus one of the number
/// of items stored.
pub fn get_complete_tree_size(nb_items: usize, node_capacity: usize) -> usize {
    let nb_nodes_necessary = udiv_ceil(nb_items, node_capacity);

    // Getting next power of two strictly greater than `nb_nodes_necessary`.
    (1 << (nb_nodes_necessary.ilog2() + 1)) - 1
}

/// There are two times less leaves than there are nodes + 1 in a complete tree.
pub fn get_complete_tree_leaves_number(
    nb_items: usize,
    node_capacity: usize,
) -> usize {
    (get_complete_tree_size(nb_items, node_capacity) + 1) >> 1
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

        tree.root = tree.complete_tree(data_items, 0);
        tree
    }

    fn complete_tree(
        &self,
        data_items: &mut Vec<DataItem>,
        level: u16,
    ) -> Option<Box<Node>> {
        if level == self.height {
            return None;
        }

        let mut node = Node::new();
        node.fill_slots(data_items);
        node.left = self.complete_tree(data_items, level + 1);
        node.right = self.complete_tree(data_items, level + 1);

        Some(Box::new(node))
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

    pub fn slots(&self) -> Iter<DataItem> {
        self.bucket.iter()
    }

    pub fn fill_slots(&mut self, data: &mut Vec<DataItem>) {
        for slot in &mut self.bucket {
            if let Some(data_item) = data.pop() {
                *slot = data_item
            } else {
                break;
            }
        }
    }

    pub fn set_bucket(&mut self, bucket_value: [DataItem; BUCKET_SIZE]) {
        self.bucket = bucket_value;
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
