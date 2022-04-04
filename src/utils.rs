use std::ops::{RangeBounds, Range, Bound};
use num_traits::*;


///
/// Aligns a value to a alignment for example: 
///
/// ```rust, ignore
/// let i: i32 = 5;
/// let c = Align::align_ciell(4);
///
/// assert_eq!(c, 8);
/// assert_eq!(f, 4);
/// ```
///
pub trait Align{
    fn align_ceil(self, alignment: Self) -> Self;
    fn align_floor(self, alignment: Self) -> Self;
}

macro_rules! align_macro{
    ($($ty:ident)+) => {
        $(
            impl Align for $ty{
                #[inline]
                fn align_ceil(self, alignment: Self) -> Self{
                    (self + alignment - 1).align_floor(alignment)
                }
                #[inline]
                fn align_floor(self, alignment: Self) -> Self{
                    (self / alignment) * alignment
                }
            }
        )+
    }
}
align_macro!(i8 u8 i16 u16 i32 u32 i64 u64 i128 u128);

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_align(){
        let i: i32 = 5;
        let c = i.align_ceil(4);
        let f = i.align_floor(4);
        println!("{}", c);

        assert_eq!(c, 8);
        assert_eq!(f, 4);
    }
}

///
/// A trait implemented on RangeBounds to clamp them inside another range.
///
/// ```rust
/// use ewgpu::utils::*;
///
/// let bound = 1..3;
///
/// let clamped = bound.clamp(0..2);
///
/// assert_eq!(clamped, 1..2);
/// ```
///
pub trait RangeClamp<T>{
    fn clamp(self, range: Range<T>) -> Range<T>;
}

impl<T: PrimInt, B: RangeBounds<T>> RangeClamp<T> for B{
    fn clamp(self, range: Range<T>) -> Range<T> {
        // Evaluate the bounds of the RangeBound.
        let start_bound = match self.start_bound(){
            Bound::Unbounded => range.start,
            Bound::Included(offset) => {(*offset).max(range.start)},
            Bound::Excluded(offset) => {(*offset + T::one()).max(range.start)},
        };

        // Evaluate the bounds of the RangeBound.
        let end_bound = match self.end_bound(){
            Bound::Unbounded => {range.end},
            Bound::Included(offset) => {(*offset + T::one()).min(range.end)},
            Bound::Excluded(offset) => {(*offset).min(range.end)},
        };

        start_bound..end_bound
    }
}
