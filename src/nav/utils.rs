use std::collections::HashSet;

pub trait ToHashSet<T> {
    fn to_hashset(&self) -> HashSet<T>;
    fn difference(&self, other: &[T]) -> Vec<T>;
    fn difference_as_set(&self, other: &[T]) -> HashSet<T>;
}
impl<T> ToHashSet<T> for Vec<T>
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    fn to_hashset(&self) -> HashSet<T> {
        self.iter().cloned().collect::<HashSet<_>>()
    }
    fn difference(&self, other: &[T]) -> Vec<T> {
        let x = self.to_hashset();
        let y = &other.to_vec().to_hashset();

        x.difference(y).cloned().collect::<Vec<_>>()
    }
    fn difference_as_set(&self, other: &[T]) -> HashSet<T> {
        let x = self.to_hashset();
        let y = &other.to_vec().to_hashset();

        x.difference(y).cloned().collect::<HashSet<_>>()
    }
}
