mod btree;
mod oram;

fn main() {
    let path_oram = oram::ORAM::new(128, 32, 32);
    println!("Hello Path-ORAM!");

    //println!("{:#?}", path_oram.get_nb_blocks());
    println!(
        "{:#?}",
        path_oram
            .get_tree()
            .get_root()
            .left()
            .left()
            .right()
            .get_value()
    );
}

#[cfg(test)]
mod tests {
    use crate::oram;

    #[test]
    fn complete_tree_test_values() {
        let path_oram = oram::ORAM::new(128, 32, 32);
        assert_eq!(
            path_oram
                .get_tree()
                .get_root()
                .left()
                .left()
                .right()
                .get_value(),
            2
        );
    }
}
