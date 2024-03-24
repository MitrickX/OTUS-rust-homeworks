use std::iter::Product;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Mul;

#[derive(PartialEq, Debug)]
pub struct Matrix<T, const N: usize>([T; N]);

impl<T, const N: usize> Matrix<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Matrix(data)
    }
}

impl<T: Add<Output = T> + Copy, const N: usize> Matrix<T, N> {
    pub fn increase(&mut self, t: T) {
        self.0 = self.0.map(|x| x + t)
    }
}

impl<T: Mul<Output = T> + Copy, const N: usize> Matrix<T, N> {
    pub fn scale(&mut self, t: T) {
        self.0 = self.0.map(|x| x * t)
    }
}

impl<T: Add<Output = T> + Copy + for<'a> Sum<&'a T>, const N: usize> Matrix<T, N> {
    pub fn sum(&self) -> T {
        self.0.iter().sum()
    }
}

impl<T: Add<Output = T> + Copy + for<'a> Product<&'a T>, const N: usize> Matrix<T, N> {
    pub fn product(&self) -> T {
        self.0.iter().product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increase_works() {
        let mut m = Matrix::new([1, 2, 3, 4, 5]);
        m.increase(10);
        assert_eq!(Matrix::new([11, 12, 13, 14, 15]), m);

        let mut m = Matrix::new([]);
        m.increase(10);
        assert_eq!(Matrix::new([]), m);
    }

    #[test]
    fn scale_works() {
        let mut m = Matrix::new([1, 2, 3, 4, 5]);
        m.scale(2);
        assert_eq!(Matrix::new([2, 4, 6, 8, 10]), m);

        let mut m = Matrix([]);
        m.scale(2);
        assert_eq!(Matrix([]), m);
    }

    #[test]
    fn sum_works() {
        let m = Matrix::new([1, 2, 3, 4, 5]);
        let result = m.sum();

        assert_eq!(15, result);

        let m = Matrix::new([]);
        let result = m.sum();

        assert_eq!(0, result);
    }

    #[test]
    fn product_works() {
        let m = Matrix::new([1, 2, 3, 4]);
        let result = m.product();

        assert_eq!(24, result);

        let m = Matrix::new([]);
        let result = m.product();

        assert_eq!(1, result);
    }
}
