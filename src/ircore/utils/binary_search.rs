use super::super::common::{TermOffset};
pub fn binary_search(
    positions: &Vec<TermOffset> , low:usize, high: usize, current: TermOffset,
    test_fn: fn(TermOffset, TermOffset) -> bool, retval_fn: fn(usize, usize) -> usize) -> usize {

    let mut mid:usize;
    let mut low_index = low;
    let mut high_index = high;
    while high_index - low_index > 1 {
        mid = (high_index + low_index)/2;
        if test_fn(positions[mid], current) {
            low_index = mid;
        }else{
            high_index = mid;
        }
    }
    retval_fn(low_index, high_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search() {
        let positions = vec![5, 20, 35, 50];
        let target = binary_search(&positions, 0, positions.len() -1 ,
            19, 
            |v1, v2 | v1 <= v2, 
            |_, v2 | v2);
        assert_eq!(target, 1);

        let target = binary_search(&positions, 0, positions.len() -1 ,
            19, 
            |v1, v2 | v1 < v2, 
            |v1, _ | v1);
        assert_eq!(target, 0);
    }
}