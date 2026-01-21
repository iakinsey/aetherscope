pub trait HashSet {
    fn contains_entities(entities: Vec<String>) -> Vec<(String, bool)>;
}
