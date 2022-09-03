use crate::ircore::common::TermId;
use std::collections::HashMap;

pub type SparseVector = HashMap<TermId, f32>;

pub trait SparseVectorOp {
    fn vec_len(&self) -> f32;
    fn vec_set(&mut self, id:TermId, value:f32) -> (TermId, f32);
    fn vec_get(&self, id:TermId) -> f32;
    fn vec_normalize(&mut self);
    fn vec_dot(&self, other: &SparseVector) -> f32;
}
impl SparseVectorOp for SparseVector {
    fn vec_len(&self) -> f32 {
        let mut length = 0.0f32;
        for v in self.values(){
            length += v*v;
        }
        length.sqrt()
    }
    fn vec_set(&mut self, id:TermId, value:f32) -> (TermId, f32){
        self.entry(id).or_insert(value);
        (id, value)
    }
    fn vec_get(&self, id:TermId) -> f32{
        let res;
        match self.get(&id){
            Some(value) => res = *value,
            None => res = f32::default(),
        }
        res
    }
    fn vec_normalize(&mut self) {
        let length = self.vec_len();
        for (_, value) in self.iter_mut() {
            *value = *value / length;
        }
    }
    fn vec_dot(&self, other: &SparseVector) -> f32 {
        let sv1;
        let sv2;
        let mut result = 0.0f32;
        if self.len() <= other.len() {
            sv1 = self;
            sv2 = other;
        }else{
            sv1 = other;
            sv2 = self;
        }
        for (id, sv1_value) in sv1.iter() {
            if sv2.contains_key(id){
                let sv2_value = sv2.get(id).unwrap();
                result += sv1_value * sv2_value;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_vector() {
        let mut sv = SparseVector::new();
        assert_eq!(sv.vec_len(), 0.0);
        let res_set = sv.vec_set(1, 5.5);
        assert_eq!(res_set, (1, 5.5));
        let mut res = sv.vec_get(1);
        assert_eq!(res, 5.5);
        res = sv.vec_get(10000);
        assert_eq!(res, 0.0);
        assert_eq!(sv.len(), 1);
        sv.vec_set(100, 2.8);
        assert_eq!(sv.len(), 2);
        assert!((sv.vec_len()-6.17170964968).abs() <= f32::EPSILON);
    }

    #[test]
    fn test_sparse_vector_normalize() {
        //   1    2    3    4    5    6    7    8    9    10   11   12   13   14   15   16
        // [0.00,0.00,0.00,0.00,1.32,0.00,0.00,0.00,0.00,0.00,0.00,1.32,0.00,0.32,0.00,1.32]
        let mut sv1 = SparseVector::new();
        sv1.vec_set(5, 1.32);
        sv1.vec_set(12, 1.32);
        sv1.vec_set(14, 0.32);
        sv1.vec_set(16, 1.32);
        sv1.vec_normalize();
        let epsilon = 0.005f32;
        assert!( (sv1.vec_get(5) - 0.57).abs() <= epsilon );
        assert!( (sv1.vec_get(12) - 0.57).abs() <= epsilon );
        assert!( (sv1.vec_get(14) - 0.14).abs() <= epsilon );
        assert!( (sv1.vec_get(16) - 0.57).abs() <= epsilon );
        assert_eq!( sv1.len(), 4);
    }

    #[test]
    fn test_sparse_vector_dot() {
        //   1    2    3    4    5    6    7    8    9    10   11   12   13   14   15   16
        // [0.00,0.00,0.00,0.00,0.57,0.00,0.00,0.00,0.00,0.00,0.00,0.57,0.00,0.14,0.00,0.57]
        let mut sv1 = SparseVector::new();
        sv1.vec_set(5, 0.57);
        sv1.vec_set(12, 0.57);
        sv1.vec_set(14, 0.14);
        sv1.vec_set(16, 0.57);

        //   1    2    3    4    5    6    7    8    9    10   11   12   13   14   15   16
        // [0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.00,0.97,0.00,0.24,0.00,0.00]
        let mut sv2 = SparseVector::new();
        sv2.vec_set(12, 0.97);
        sv2.vec_set(14, 0.24);

        let res = sv1.vec_dot(&sv2);
        assert!((res - 0.59).abs() <= 0.005);
    }

}