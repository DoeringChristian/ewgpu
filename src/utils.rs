
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


