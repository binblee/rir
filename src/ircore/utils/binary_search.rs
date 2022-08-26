use super::super::common::TermOffset;
pub fn binary_search(
    positions: &Vec<TermOffset> , low:usize, high: usize, current: u32,
    test_fn: fn(u32, u32) -> bool, retval_fn: fn(usize, usize) -> usize) -> usize {

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
