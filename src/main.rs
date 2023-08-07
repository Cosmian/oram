use crate::oram::{Stash, ORAM};

mod btree;
mod oram;

const STASH_SIZE: usize = 32;

fn main() {
    let stash = Stash::new(STASH_SIZE);
    let nb_blocks = 128;
    let block_size = 64;
    let path_oram = ORAM::new(stash, nb_blocks, block_size);
    println!("Hello Path-ORAM!");

    let path = 49;
    let path_values = path_oram.read_path(path);
    println!("{:?}", path_values);
}

#[cfg(test)]
mod tests {
    use crate::oram::{Stash, ORAM};
    use crate::STASH_SIZE;

    #[test]
    fn complete_tree_test_values() {
        let stash = Stash::new(STASH_SIZE);
        let nb_blocks = 129;
        let block_size = 64;
        let path_oram = ORAM::new(stash.clone(), nb_blocks, block_size);
        println!("Hello Path-ORAM!");

        let path = 49;
        let path_values = path_oram.read_path(path);

        let mut expected_path_values = vec![0, 1, 2, 3, 4, 5, 6, 7];
        expected_path_values.extend_from_slice(stash.stash());

        assert_eq!(path_values, expected_path_values);
    }
}
