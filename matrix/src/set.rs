use crate::matrix::Matrix;
use std::iter::Product;
use std::iter::Sum;
use std::ops::Add;

pub struct Set<'set, 'elem, T, const N: usize>(&'set [&'elem Matrix<T, N>]);

impl<'set, 'elem, T, const N: usize> Set<'set, 'elem, T, N> {
    pub fn new(matrix: &'set [&'elem Matrix<T, N>]) -> Self {
        Set(matrix)
    }
}

impl<'set, 'elem, T, const N: usize> Set<'set, 'elem, T, N> {
    pub fn get(&self, idx: usize) -> &'elem Matrix<T, N> {
        self.0[idx]
    }
}

impl<'set, 'elem, T: Add<Output = T> + Copy + for<'sum> Sum<&'sum T>, const N: usize>
    Set<'set, 'elem, T, N>
{
    pub fn sum(&self) -> T {
        let mut sums = Vec::<T>::with_capacity(N);

        for v in self.0 {
            let m = *v;
            let s = m.sum();
            sums.push(s)
        }

        sums.iter().sum()
    }
}

impl<'set, 'elem, T: Add<Output = T> + Copy + for<'sum> Product<&'sum T>, const N: usize>
    Set<'set, 'elem, T, N>
{
    pub fn product(&self) -> T {
        let mut products = Vec::<T>::with_capacity(N);

        for v in self.0 {
            let m = *v;
            let s = m.product();
            products.push(s)
        }

        products.iter().product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_works() {
        let m1 = Matrix::new([1, 2, 3]);
        let m2 = Matrix::new([4, 5, 6]);
        let m3 = Matrix::new([7, 8, 9]);

        let m_long_live: &Matrix<i32, 3>;

        {
            let binding = [&m1, &m2, &m3];
            let s = Set::new(&binding);

            assert_eq!(&m1, s.get(0));
            assert_eq!(&m2, s.get(1));
            assert_eq!(&m3, s.get(2));

            m_long_live = s.get(0)
        }

        // МОЖНО ИСПЛОЬЗОВАТЬ m_long_live
        assert_eq!(Matrix::<i32, 3>::new([1, 2, 3]), *m_long_live)
    }

    #[test]
    fn sum_works() {
        let m1 = Matrix::new([1, 2, 3]);
        let m2 = Matrix::new([4, 5, 6]);
        let m3 = Matrix::new([7, 8, 9]);

        let binding = [&m1, &m2, &m3];
        let s = Set::new(&binding);

        let result = s.sum();

        assert_eq!(45, result);

        let binding: [&Matrix<i32, 3>; 0] = [];
        let s = Set::new(&binding);

        let result = s.sum();

        assert_eq!(0, result);
    }

    #[test]
    fn product_works() {
        let m1 = Matrix::new([1, 2, 3]);
        let m2 = Matrix::new([4, 5, 6]);

        let binding = [&m1, &m2];
        let s = Set::new(&binding);

        let result = s.product();

        assert_eq!(720, result);

        let binding: [&Matrix<i32, 3>; 0] = [];
        let s = Set::new(&binding);

        let result = s.product();

        assert_eq!(1, result);
    }
}
