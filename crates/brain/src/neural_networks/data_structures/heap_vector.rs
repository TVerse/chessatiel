use crate::neural_networks::data_structures::HeapMatrix;
use num_traits::{Num, Zero};
use rand::{Error, Fill, Rng};
use serde_derive::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, Sub};

/// Equivalent to [T;N]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct HeapVector<T, const N: usize> {
    inner: Vec<T>,
}

impl<T, const N: usize> HeapVector<T, N> {
    pub fn new(inner: Vec<T>) -> Self {
        assert_eq!(
            inner.len(),
            N,
            "HeapVector size was not equal to passed-in vector"
        );
        Self { inner }
    }

    pub fn len(&self) -> usize {
        N
    }

    pub fn is_empty(&self) -> bool {
        N == 0
    }

    pub fn matrix(self) -> HeapMatrix<T, 1, N> {
        HeapMatrix::new(vec![self])
    }

    pub fn apply(mut self, f: impl Fn(T) -> T) -> Self {
        self.inner = self.inner.into_iter().map(f).collect();
        self
    }

    pub fn to_vec(&self) -> &Vec<T> {
        &self.inner
    }
}

impl<T, const N: usize> Index<usize> for HeapVector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for HeapVector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T: Zero + Copy, const N: usize> HeapVector<T, N> {
    pub fn zeroed() -> Self {
        Self {
            inner: vec![T::zero(); N],
        }
    }
}

impl<T: Num + Copy, const N: usize> HeapVector<T, N> {
    pub fn hadamard(mut self, other: &HeapVector<T, N>) -> HeapVector<T, N> {
        for i in 0..N {
            self.inner[i] = self.inner[i] * other.inner[i]
        }
        self
    }

    pub fn dot(&self, other: &HeapVector<T, N>) -> T {
        let mut sum = T::zero();
        for s in self
            .inner
            .iter()
            .zip(other.inner.iter())
            .map(|(a, b)| *a * *b)
        {
            sum = sum + s
        }
        sum
    }

    pub fn product_to_matrix<const M: usize>(
        &self,
        other: &HeapVector<T, M>,
    ) -> HeapMatrix<T, N, M> {
        let mut v = Vec::with_capacity(M);
        for i in 0..N {
            v.push(other.clone() * self.inner[i])
        }
        HeapMatrix::new(v)
    }

    pub fn squared_size(&self) -> T {
        let mut res = T::zero();
        for i in self.inner.iter().map(|f| *f * *f) {
            res = res + i
        }
        res
    }

    pub fn sum(&self) -> T {
        let mut res = T::zero();
        for i in self.inner.iter() {
            res = res + *i
        }
        res
    }
}

impl<T: Num + Copy, const N: usize> Sub<&HeapVector<T, N>> for HeapVector<T, N> {
    type Output = HeapVector<T, N>;

    fn sub(mut self, rhs: &HeapVector<T, N>) -> Self::Output {
        for i in 0..N {
            self.inner[i] = self.inner[i] - rhs.inner[i]
        }
        self
    }
}

impl<T: Num + Copy + Mul, const N: usize> Add<T> for HeapVector<T, N> {
    type Output = HeapVector<T, N>;

    fn add(mut self, rhs: T) -> Self::Output {
        for i in self.inner.iter_mut() {
            *i = *i + rhs
        }
        self
    }
}

impl<T: Num + Copy + Mul, const N: usize> Add<&HeapVector<T, N>> for HeapVector<T, N> {
    type Output = HeapVector<T, N>;

    fn add(mut self, rhs: &HeapVector<T, N>) -> Self::Output {
        for i in 0..N {
            self.inner[i] = self.inner[i] + rhs.inner[i]
        }
        self
    }
}

impl<T: Num + Copy + Mul, const N: usize> AddAssign<&HeapVector<T, N>> for HeapVector<T, N> {
    fn add_assign(&mut self, rhs: &HeapVector<T, N>) {
        for i in 0..N {
            self.inner[i] = self.inner[i] + rhs.inner[i]
        }
    }
}

impl<T: Num + Copy + Mul, const N: usize> Sub<T> for HeapVector<T, N> {
    type Output = HeapVector<T, N>;

    fn sub(mut self, rhs: T) -> Self::Output {
        for i in self.inner.iter_mut() {
            *i = *i - rhs
        }
        self
    }
}

impl<T: Num + Copy + Mul, const N: usize> Mul<T> for HeapVector<T, N> {
    type Output = HeapVector<T, N>;

    fn mul(mut self, rhs: T) -> Self::Output {
        for i in self.inner.iter_mut() {
            *i = *i * rhs
        }
        self
    }
}

impl<T: Num + Copy + Div, const N: usize> Div<T> for HeapVector<T, N> {
    type Output = HeapVector<T, N>;

    fn div(mut self, rhs: T) -> Self::Output {
        for i in self.inner.iter_mut() {
            *i = *i / rhs
        }
        self
    }
}

impl<T: Num + Copy + Div, const N: usize> DivAssign<T> for HeapVector<T, N> {
    fn div_assign(&mut self, rhs: T) {
        for i in self.inner.iter_mut() {
            *i = *i / rhs
        }
    }
}

impl<T: Num + Copy + Div, const N: usize> DivAssign<T> for &mut HeapVector<T, N> {
    fn div_assign(&mut self, rhs: T) {
        for i in self.inner.iter_mut() {
            *i = *i / rhs
        }
    }
}

impl<T, const N: usize> Fill for HeapVector<T, N>
where
    [T]: Fill,
{
    fn try_fill<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), Error> {
        self.inner.try_fill(rng)
    }
}

impl<T, const N: usize> From<[T; N]> for HeapVector<T, N> {
    fn from(arr: [T; N]) -> Self {
        HeapVector::new(arr.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_fn() {
        let hv = HeapVector::from([1, 2, 3, 4]);
        let res = hv.apply(|i| i + 1).inner;
        let expected = vec![2, 3, 4, 5];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_dot() {
        let hv1 = HeapVector::from([1, 2, 3, 4]);
        let hv2 = HeapVector::from([1, 2, 3, 4]);
        assert_eq!(hv1.dot(&hv2), 30)
    }

    #[test]
    fn test_hadamard() {
        let hv1 = HeapVector::from([1, 2, 3, 4]);
        let hv2 = HeapVector::from([1, 2, 3, 4]);
        let expected = HeapVector::from([1, 4, 9, 16]);
        assert_eq!(hv1.hadamard(&hv2), expected)
    }

    #[test]
    fn test_product_to_matrix() {
        let hv1 = HeapVector::from([1, 2, 3, 4]);
        let hv2 = HeapVector::from([1, 2, 3]);
        let expected = HeapMatrix::from([[1, 2, 3], [2, 4, 6], [3, 6, 9], [4, 8, 12]]);
        assert_eq!(hv1.product_to_matrix(&hv2), expected)
    }

    #[test]
    fn test_sum() {
        let hv1 = HeapVector::from([1, 2, 3, -4]);
        assert_eq!(hv1.sum(), 2)
    }

    #[test]
    fn test_squared_size() {
        let hv1 = HeapVector::from([1, 2, 3, -4]);
        assert_eq!(hv1.squared_size(), 30)
    }

    #[test]
    fn test_add_sub_mul_div_scalar() {
        let hv1 = HeapVector::from([1, 2, 3, -4]);
        let expected = HeapVector::from([0, 1, 2, -5]);
        assert_eq!(hv1.clone() - 1, expected);
        let hv1 = HeapVector::from([1, 2, 3, -4]);
        let expected = HeapVector::from([2, 3, 4, -3]);
        assert_eq!(hv1 + 1, expected);
        let hv1 = HeapVector::from([1, 2, 3, -4]);
        let expected = HeapVector::from([2, 4, 6, -8]);
        assert_eq!(hv1 * 2, expected);
        let hv1 = HeapVector::from([2, 4, 6, -8]);
        let expected = HeapVector::from([1, 2, 3, -4]);
        assert_eq!(hv1 / 2, expected);
    }

    #[test]
    fn test_sub_vector() {
        let hv1 = HeapVector::from([1, 2, 3, -4]);
        let hv2 = HeapVector::from([1, 2, 3, 4]);
        let expected = HeapVector::from([0, 0, 0, -8]);
        assert_eq!(hv1 - &hv2, expected)
    }
}
