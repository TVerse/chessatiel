use num_traits::{Num, Zero};
use rand::{Error, Fill, Rng};
use serde_derive::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, Sub, SubAssign};

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
    pub fn hadamard(&self, other: &HeapVector<T, N>) -> HeapVector<T, N> {
        let mut res = HeapVector::zeroed();
        for i in 0..N {
            res.inner[i] = self.inner[i] * other.inner[i]
        }
        res
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
        let mut out = HeapMatrix::zeroed();
        for i in 0..N {
            for j in 0..M {
                out.inner[i].inner[j] = self.inner[i] * other.inner[j]
            }
        }
        out
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

impl<T: Num + Copy, const N: usize> Sub for &HeapVector<T, N> {
    type Output = HeapVector<T, N>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = HeapVector::zeroed();
        for i in 0..res.len() {
            res.inner[i] = self.inner[i] - rhs.inner[i]
        }
        res
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

// impl<T, const N: usize> Deref for HeapVector<T, N> {
//     type Target = [T; N];
//
//     fn deref(&self) -> &Self::Target {
//         let s: &[T] = &self.inner;
//         s.try_into().unwrap()
//     }
// }
//
// impl<T, const N: usize> DerefMut for HeapVector<T, N> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         let s: &mut [T] = &mut self.inner;
//         s.try_into().unwrap()
//     }
// }

// TODO
// impl<T, const N: usize> Index<usize> for HeapVector<T, N> {
//     type Output = T;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         &self.inner[index]
//     }
// }
//
// impl<T, const N: usize> IndexMut<usize> for HeapVector<T, N> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         &mut self.inner[index]
//     }
// }

impl<T, const N: usize> Fill for HeapVector<T, N>
where
    [T]: Fill,
{
    fn try_fill<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), Error> {
        self.inner.try_fill(rng)
    }
}

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
        for j in 0..M {
            let mut sum = T::zero();
            for k in 0..N {
                sum = sum + self.inner[k].inner[j] * rhs.inner[k]
            }
            out.inner[j] = sum
        }
        out
    }
}

// TODO
// impl<T, const M: usize, const N: usize> Deref for HeapMatrix<T, M, N> {
//     type Target = [HeapVector<T, N>; M];
//
//     fn deref(&self) -> &Self::Target {
//         let s: &[HeapVector<T, N>] = &self.inner;
//         s.try_into().unwrap()
//     }
// }
//
// impl<T, const M: usize, const N: usize> DerefMut for HeapMatrix<T, M, N> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         let s: &mut [HeapVector<T, N>] = &mut self.inner;
//         s.try_into().unwrap()
//     }
// }

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
                self.inner[i].inner[j] = self.inner[i].inner[j] - rhs.inner[i].inner[j]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_fn() {
        let hv = HeapVector::<_, 4>::new(vec![1, 2, 3, 4]);
        let res = hv.apply(|i| i + 1).inner;
        let expected = vec![2, 3, 4, 5];
        assert_eq!(res, expected);
    }
}
