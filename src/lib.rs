// ---------------------------------------------------------------------------
// Copyright:   (c) 2021 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

use core::fmt;
use core::ops::{Add, Div, Mul, Sub};
#[cfg(fpdec)]
use fpdec::{Dec, Decimal};
pub use si_prefixes::SIPrefix;
pub use unitless::{Unitless, NONUNIT};

mod macros;
mod si_prefixes;
mod unitless;

#[cfg(fpdec)]
pub type AmountT = Decimal;
#[cfg(all(not(fpdec), target_pointer_width = "64"))]
pub type AmountT = f64;
#[cfg(all(not(fpdec), target_pointer_width = "32"))]
pub type AmountT = f32;

pub trait Unit: Copy + PartialEq + Sized + Mul<AmountT> {
    const REF_UNIT: Option<Self>;

    fn name(&self) -> &'static str;

    fn symbol(&self) -> &'static str;

    fn si_prefix(&self) -> Option<SIPrefix>;

    #[inline]
    fn is_ref_unit(&self) -> bool {
        Self::REF_UNIT == Some(*self)
    }

    /// Returns `Some(factor)` so that `factor` * `Self::REFUNIT` == 1 * `self`,
    /// or `None` if `Self::REF_UNIT` is `None`.
    fn scale(&self) -> Option<AmountT>;

    /// Returns `Some(factor)` so that `factor` * `other` == 1 * `self`, or
    /// `None` if `Self::REF_UNIT` is `None`.
    fn ratio(&self, other: &Self) -> Option<AmountT> {
        match (self.scale(), other.scale()) {
            (Some(from), Some(into)) => Some(from / into),
            _ => None,
        }
    }
}

pub trait Quantity<U: Unit>:
    Copy + Sized + Add<Self> + Sub<Self> + Div<Self> + Mul<AmountT>
{
    // Returns a new instance of `Quantity<U>`.
    fn new(amount: AmountT, unit: U) -> Self;

    // Returns the amount of `self`.
    fn amount(&self) -> AmountT;

    // Returns the unit of `self`.
    fn unit(&self) -> U;

    // Returns `Some(factor)` so that `factor` * `unit` == `self`, or `None` if
    // such a factor can't be determined.
    fn equiv_amount(&self, unit: U) -> Option<AmountT> {
        if self.unit() == unit {
            Some(self.amount())
        } else {
            // TODO: add converters
            Some(self.unit().ratio(&unit)? * self.amount())
        }
    }

    /// Returns `Some(qty)` where `qty` == `self` and `qty.unit()` is `to_unit`,
    /// or `None` if the conversion can't be done.
    fn convert(&self, to_unit: U) -> Option<Self> {
        Some(Self::new(self.equiv_amount(to_unit)?, to_unit))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Qty<U: Unit> {
    amount: AmountT,
    unit: U,
}

impl<U: Unit> fmt::Display for Qty<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.unit.symbol() {
            "" => write!(f, "{}", self.amount),
            _ => write!(f, "{} {}", self.amount, self.unit.symbol()),
        }
    }
}

impl<U: Unit> Quantity<U> for Qty<U> {
    #[inline(always)]
    fn new(amount: AmountT, unit: U) -> Self {
        Self { amount, unit }
    }

    #[inline(always)]
    fn amount(&self) -> AmountT {
        self.amount
    }

    #[inline(always)]
    fn unit(&self) -> U {
        self.unit
    }
}

impl<U: Unit> Add<Self> for Qty<U> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match rhs.equiv_amount(self.unit) {
            Some(equiv) => Self::Output::new(self.amount + equiv, self.unit),
            None => panic!("Incompatible units!"),
        }
    }
}

impl<U: Unit> Sub<Self> for Qty<U> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs.equiv_amount(self.unit) {
            Some(equiv) => Self::Output::new(self.amount - equiv, self.unit),
            None => panic!("Incompatible units!"),
        }
    }
}

impl<U: Unit> Div<Self> for Qty<U> {
    type Output = Unitless;

    fn div(self, rhs: Self) -> Self::Output {
        match rhs.equiv_amount(self.unit) {
            Some(equiv) => Self::Output::new(self.amount / equiv, NONUNIT),
            None => panic!("Incompatible units!"),
        }
    }
}

impl<U: Unit> Mul<Qty<U>> for AmountT {
    type Output = Qty<U>;

    #[inline(always)]
    fn mul(self, rhs: Qty<U>) -> Self::Output {
        Self::Output::new(self * rhs.amount, rhs.unit)
    }
}

impl<U: Unit> Mul<AmountT> for Qty<U> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: AmountT) -> Self::Output {
        Self::Output::new(self.amount * rhs, self.unit)
    }
}
