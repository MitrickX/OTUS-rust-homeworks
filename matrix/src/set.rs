use crate::matrix::Matrix;
use std::iter::Product;
use std::iter::Sum;
use std::ops::Add;

pub struct Set<'a, T, const N: usize>(&'a [&'a Matrix<T, N>]);

impl<'a, T, const N: usize> Set<'a, T, N> {
    pub fn new(matrix: &'a [&'a Matrix<T, N>]) -> Self {
        Set(matrix)
    }
}

impl<'a, T, const N: usize> Set<'a, T, N> {
    pub fn get(&self, idx: usize) -> &'a Matrix<T, N> {
        self.0[idx]
    }
}

impl<'a, T: Add<Output = T> + Copy + for<'b> Sum<&'b T>, const N: usize> Set<'a, T, N> {
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

impl<'a, T: Add<Output = T> + Copy + for<'b> Product<&'b T>, const N: usize> Set<'a, T, N> {
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
        let mut m1 = Matrix::new([1, 2, 3]);
        let m2 = Matrix::new([4, 5, 6]);
        let m3 = Matrix::new([7, 8, 9]);

        let m: &mut Matrix<i32, 3>;

        {
            let binding = [&m1, &m2, &m3];
            let s = Set::new(&binding);

            assert_eq!(&m1, s.get(0));
            assert_eq!(&m2, s.get(1));
            assert_eq!(&m3, s.get(2));

            m = &mut m1;
        }

        // Надо убедиться, что полученная ссылка живёт дольше экземпляра набора матриц
        m.scale(2);
        assert_eq!(m, m);
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
