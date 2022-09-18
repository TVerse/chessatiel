use num_traits::{Num, Zero};
use rand::{Error, Fill, Rng};
use std::ops::{Deref, DerefMut, Index, IndexMut};

/// Equivalent to [T;N]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeapArray<T, const N: usize> {
    inner: Vec<T>,
}

impl<T, const N: usize> HeapArray<T, N> {
    pub fn new(inner: Vec<T>) -> Self {
        assert_eq!(
            inner.len(),
            N,
            "HeapArray size was not equal to passed-in vector"
        );
        Self { inner }
    }

    pub fn len(&self) -> usize {
        N
    }
}

impl<T: Zero + Copy, const N: usize> HeapArray<T, N> {
    pub fn zeroed() -> Self {
        Self {
            inner: vec![T::zero(); N],
        }
    }
}

impl<T: Num + Copy, const N: usize> HeapArray<T, N> {
    pub fn hadamard(&self, other: &HeapArray<T, N>) -> HeapArray<T, N> {
        let mut res = HeapArray::zeroed();
        for i in 0..N {
            res[i] = self[i] * other[i]
        }
        res
    }

    pub fn dot(&self, other: &HeapArray<T, N>) -> T {
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
}

impl<T, const N: usize> Deref for HeapArray<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        let s: &[T] = &self.inner;
        s.try_into().unwrap()
    }
}

impl<T, const N: usize> DerefMut for HeapArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let s: &mut [T] = &mut self.inner;
        s.try_into().unwrap()
    }
}

impl<T, const N: usize> Index<usize> for HeapArray<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for HeapArray<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T, const N: usize> Fill for HeapArray<T, N>
where
    [T]: Fill,
{
    fn try_fill<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), Error> {
        self.inner.try_fill(rng)
    }
}

/// Equivalent to [[T; N]; M]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeapMatrix<T, const M: usize, const N: usize> {
    inner: Vec<HeapArray<T, N>>,
}

impl<T, const M: usize, const N: usize> HeapMatrix<T, M, N> {
    pub fn new(inner: Vec<HeapArray<T, N>>) -> Self {
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
}

impl<T: Zero + Copy, const M: usize, const N: usize> HeapMatrix<T, M, N> {
    pub fn zeroed() -> Self {
        let row = HeapArray::zeroed();
        let mut inner = Vec::with_capacity(M);
        for _ in 0..M {
            inner.push(row.clone());
        }
        Self::new(inner)
    }
}

impl<T, const M: usize, const N: usize> Deref for HeapMatrix<T, M, N> {
    type Target = [HeapArray<T, N>; M];

    fn deref(&self) -> &Self::Target {
        let s: &[HeapArray<T, N>] = &self.inner;
        s.try_into().unwrap()
    }
}

impl<T, const M: usize, const N: usize> DerefMut for HeapMatrix<T, M, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let s: &mut [HeapArray<T, N>] = &mut self.inner;
        s.try_into().unwrap()
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
