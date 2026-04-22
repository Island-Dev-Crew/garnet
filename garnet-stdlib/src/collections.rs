//! Collections operations (no caps — pure compute).
//!
//! The interpreter's `Value::Array` and `Value::Map` are the runtime
//! carriers; these functions operate on Rust types that the bridge
//! layer translates to/from Value.

use crate::StdError;

pub fn array_insert<T: Clone>(arr: &mut Vec<T>, idx: usize, v: T) -> Result<(), StdError> {
    if idx > arr.len() {
        return Err(StdError::InvalidInput(format!(
            "insert index {idx} exceeds array length {}",
            arr.len()
        )));
    }
    arr.insert(idx, v);
    Ok(())
}

pub fn array_remove<T: Clone>(arr: &mut Vec<T>, idx: usize) -> Result<T, StdError> {
    if idx >= arr.len() {
        return Err(StdError::InvalidInput(format!(
            "remove index {idx} out of bounds (len {})",
            arr.len()
        )));
    }
    Ok(arr.remove(idx))
}

pub fn array_sort<T: Ord>(arr: &mut Vec<T>) {
    arr.sort();
}

pub fn array_contains<T: PartialEq>(arr: &[T], v: &T) -> bool {
    arr.contains(v)
}

pub fn array_index_of<T: PartialEq>(arr: &[T], v: &T) -> Option<usize> {
    arr.iter().position(|x| x == v)
}

pub fn array_slice<T: Clone>(arr: &[T], start: usize, end: usize) -> Result<Vec<T>, StdError> {
    if start > end || end > arr.len() {
        return Err(StdError::InvalidInput(format!(
            "slice({start}, {end}) out of bounds for len {}",
            arr.len()
        )));
    }
    Ok(arr[start..end].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_happy() {
        let mut v = vec![1, 2, 3];
        array_insert(&mut v, 1, 99).unwrap();
        assert_eq!(v, vec![1, 99, 2, 3]);
    }

    #[test]
    fn insert_at_end_ok() {
        let mut v = vec![1, 2];
        array_insert(&mut v, 2, 3).unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn insert_past_end_rejected() {
        let mut v = vec![1, 2];
        assert!(array_insert(&mut v, 5, 3).is_err());
    }

    #[test]
    fn remove_returns_and_shifts() {
        let mut v = vec![10, 20, 30];
        assert_eq!(array_remove(&mut v, 1).unwrap(), 20);
        assert_eq!(v, vec![10, 30]);
    }

    #[test]
    fn remove_out_of_bounds() {
        let mut v: Vec<i32> = vec![];
        assert!(array_remove(&mut v, 0).is_err());
    }

    #[test]
    fn sort_stable_ascending() {
        let mut v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        array_sort(&mut v);
        assert_eq!(v, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn contains_and_index_of() {
        let v = vec!["a", "b", "c"];
        assert!(array_contains(&v, &"b"));
        assert!(!array_contains(&v, &"x"));
        assert_eq!(array_index_of(&v, &"c"), Some(2));
        assert_eq!(array_index_of(&v, &"x"), None);
    }

    #[test]
    fn slice_happy() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(array_slice(&v, 1, 4).unwrap(), vec![2, 3, 4]);
    }

    #[test]
    fn slice_empty_range_ok() {
        let v = vec![1, 2, 3];
        assert_eq!(array_slice(&v, 2, 2).unwrap(), Vec::<i32>::new());
    }

    #[test]
    fn slice_out_of_bounds_rejected() {
        let v = vec![1, 2, 3];
        assert!(array_slice(&v, 1, 10).is_err());
        assert!(array_slice(&v, 3, 1).is_err()); // start > end
    }
}
