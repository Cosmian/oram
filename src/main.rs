mod btree;
mod oram;

fn main() {
    let path_oram = oram::ORAM::new(128, 32, 32);
    println!("Hello Path-ORAM!");

    let path = 49;
    //let height = &path_oram.get_tree().height();
    oram::path_traversal(Some(Box::new(path_oram.get_tree().root())), path, 7);
}

#[cfg(test)]
mod tests {
    use crate::oram;

    #[test]
    fn complete_tree_test_values() {
        let path_oram = oram::ORAM::new(128, 32, 32);
        //assert_eq!(
        //    path_oram
        //        .get_tree()
        //        .get_root()
        //        .left()
        //        .left()
        //        .right()
        //        .get_value(),
        //    2
        //);
    }
}
