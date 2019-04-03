use bitintr::{Andn, Bextr};
use num_traits::{AsPrimitive, PrimInt, WrappingSub};

pub trait IntExt {
    fn bit(self, bit: Self) -> bool;

    fn set_mask_from(self, mask: Self, condition: bool) -> Self;

    #[inline(always)]
    fn set_mask_from_assign(&mut self, mask: Self, condition: bool)
    where
        Self: PrimInt,
    {
        *self = self.set_mask_from(mask, condition);
    }
}

impl<T: 'static> IntExt for T
where
    T: PrimInt + WrappingSub + Bextr + Andn,
    bool: AsPrimitive<Self>,
{
    #[inline(always)]
    fn bit(self, bit: Self) -> bool {
        self.bextr(bit, Self::one()) == Self::one()
    }

    #[inline(always)]
    fn set_mask_from(self, mask: Self, condition: bool) -> Self {
        mask.andn(self) | ((Self::zero().wrapping_sub(&condition.as_())) & mask)
    }
}

#[cfg(test)]
mod tests {
    use super::IntExt;

    #[test]
    fn test_set_mask_from() {
        assert_eq!(0b1100.set_mask_from(0b0100, false), 0b1000);
        assert_eq!(0b1100.set_mask_from(0b0011, true), 0b1111);
    }
}
