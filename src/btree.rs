use crate::oram::BUCKET_SIZE;

#[derive(Debug, Clone, Default)]
pub struct BTree {
    pub(super) root: Option<Box<Node>>,
    height: u16,
}

impl BTree {
    pub fn init_new(data_items: &mut Vec<DataItem>, nb_items: usize) -> BTree {
        let mut tree = BTree {
            root: Option::None,
            height: nb_items.ilog2() as u16 + 1,
        };

        let path = 0;
        let mut root = Node::new();

        tree.complete_tree(&mut root, data_items, path, 0);
        tree.root = Some(Box::new(root));

        tree
    }

    fn complete_tree(
        &self,
        node: &mut Node,
        data_items: &mut Vec<DataItem>,
        path: usize,
        level: u16,
    ) {
        // -1 is to avoid constructing 1 extra level.
        if level < self.height - 1 {
            let mut left: Box<Node> = Box::new(Node::new());
            let mut right = Box::new(Node::new());

            self.complete_tree(&mut left, data_items, path * 2, level + 1);
            self.complete_tree(&mut right, data_items, path * 2 + 1, level + 1);

            node.left = Some(left);
            node.right = Some(right);
        }

        /*
         * Greedily filling buckets following a right side visit to fill leaves
         * first.
         */
        for i in 0..BUCKET_SIZE {
            for j in 0..data_items.len() {
                /* At this point path is only `level` bits long. We compare the
                 * MSB of the path of the element to insert, to see if the path
                 * is at an intersection with the current visit of the tree.
                 */
                if data_items[j].path() >> (self.height - level - 1) == path {
                    node.set_bucket_element(data_items.swap_remove(j), i);
                    break;
                }
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
                DataItem::new(Vec::new(), 0),
                DataItem::new(Vec::new(), 0),
                DataItem::new(Vec::new(), 0),
                DataItem::new(Vec::new(), 0),
            ],
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
    path: usize,
}

impl DataItem {
    pub fn new(data: Vec<u8>, path: usize) -> DataItem {
        DataItem { data, path }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn path(&self) -> usize {
        self.path
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn set_path(&mut self, path: usize) {
        self.path = path;
    }
}
