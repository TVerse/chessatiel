use crate::neural_networks::data_structures::HeapVector;
use num_traits::{Num, Zero};
use rand::{Error, Fill, Rng};
use serde_derive::{Deserialize, Serialize};
use std::ops::{Div, DivAssign, Index, IndexMut, Mul, Sub, SubAssign};

/// Equivalent to [[T; N]; M]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct HeapMatrix<T, const M: usize, const N: usize> {
    inner: Vec<HeapVector<T, N>>,
}

impl<T, const M: usize, const N: usize> HeapMatrix<T, M, N> {
    pub fn new(inner: Vec<HeapVector<T, N>>) -> Self {
        assert_eq!(
            inner.len(),
            M,
            "HeapMatrix size was not equal to passed-in vector"
        );
        Self { inner }
    }

    pub fn len(&self) -> usize {
        M
    }

    pub fn is_empty(&self) -> bool {
        M == 0
    }
}

impl<T, const N: usize> HeapMatrix<T, 1, N> {
    pub fn vector(&self) -> &HeapVector<T, N> {
        &self.inner[0]
    }
}

impl<T: Zero + Copy, const M: usize, const N: usize> HeapMatrix<T, M, N> {
    pub fn zeroed() -> Self {
        let row = HeapVector::zeroed();
        let mut inner = Vec::with_capacity(M);
        for _ in 0..M {
            inner.push(row.clone());
        }
        Self::new(inner)
    }
}

// (lhs)T * rhs
impl<T: Num + Copy + Mul, const M: usize, const N: usize> HeapMatrix<T, N, M> {
    pub fn mul_transposed(&self, rhs: &HeapVector<T, N>) -> HeapVector<T, M> {
        let mut out = HeapVector::zeroed();
        for k in 0..N {
            for j in 0..M {
                out[j] = out[j] + self.inner[k][j] * rhs[k]
            }
        }
        out
    }
}

impl<T, const M: usize, const N: usize> Index<usize> for HeapMatrix<T, M, N> {
    type Output = HeapVector<T, N>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T, const M: usize, const N: usize> IndexMut<usize> for HeapMatrix<T, M, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T, const M: usize, const N: usize> Fill for HeapMatrix<T, M, N>
where
    [T]: Fill,
{
    fn try_fill<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), Error> {
        for ha in &mut self.inner {
            ha.try_fill(rng)?
        }
        Ok(())
    }
}

impl<T: Num + Copy + Div, const M: usize, const N: usize> Div<T> for HeapMatrix<T, M, N> {
    type Output = HeapMatrix<T, M, N>;

    fn div(mut self, rhs: T) -> Self::Output {
        for mut i in self.inner.iter_mut() {
            i /= rhs;
        }
        self
    }
}

impl<T: Num + Copy + Mul, const M: usize, const N: usize> Sub<T> for HeapMatrix<T, M, N> {
    type Output = HeapMatrix<T, M, N>;

    fn sub(mut self, rhs: T) -> Self::Output {
        self.inner = self.inner.into_iter().map(|arr| arr - rhs).collect();
        self
    }
}

impl<T: Num + Copy + Mul, const M: usize, const N: usize> SubAssign<&HeapMatrix<T, M, N>>
    for HeapMatrix<T, M, N>
{
    fn sub_assign(&mut self, rhs: &HeapMatrix<T, M, N>) {
        for i in 0..M {
            for j in 0..N {
                self.inner[i][j] = self.inner[i][j] - rhs.inner[i][j]
            }
        }
    }
}

impl<T: Num + Copy + Mul, const M: usize, const N: usize> Mul<T> for HeapMatrix<T, M, N> {
    type Output = HeapMatrix<T, M, N>;

    fn mul(mut self, rhs: T) -> Self::Output {
        self.inner = self.inner.into_iter().map(|arr| arr * rhs).collect();
        self
    }
}

impl<T: Num + Copy + Mul, const M: usize, const N: usize> Mul<&HeapVector<T, N>>
    for HeapMatrix<T, M, N>
{
    type Output = HeapVector<T, M>;

    fn mul(self, rhs: &HeapVector<T, N>) -> Self::Output {
        let mut out = HeapVector::zeroed();
        for i in 0..M {
            out[i] = self.inner[i].dot(rhs);
        }
        out
    }
}

impl<T: Num + Copy + Mul, const M: usize, const N: usize> Mul<&HeapVector<T, N>>
    for &HeapMatrix<T, M, N>
{
    type Output = HeapVector<T, M>;

    fn mul(self, rhs: &HeapVector<T, N>) -> Self::Output {
        let mut out = HeapVector::zeroed();
        for i in 0..M {
            out[i] = self.inner[i].dot(rhs);
        }
        out
    }
}

impl<T: Num + Copy + Div, const M: usize, const N: usize> DivAssign<T> for HeapMatrix<T, M, N> {
    fn div_assign(&mut self, rhs: T) {
        for mut arr in self.inner.iter_mut() {
            arr /= rhs
        }
    }
}

impl<T, const M: usize, const N: usize> From<[[T; N]; M]> for HeapMatrix<T, M, N> {
    fn from(arr: [[T; N]; M]) -> Self {
        let mut v = Vec::with_capacity(M);
        for sub in arr.into_iter() {
            v.push(sub.into())
        }
        HeapMatrix::new(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_transposed() {
        let hm1 = HeapMatrix::from([[1, 2, 3], [4, 5, 6]]);
        let hv1 = HeapVector::from([1, 2]);
        let expected = HeapVector::from([9, 12, 15]);
        assert_eq!(hm1.mul_transposed(&hv1), expected)
    }

    #[test]
    fn test_mul() {
        let hm1 = HeapMatrix::from([[1, 4], [2, 5], [3, 6]]);
        let hv1 = HeapVector::from([1, 2]);
        let expected = HeapVector::from([9, 12, 15]);
        assert_eq!(hm1 * &hv1, expected)
    }
}
