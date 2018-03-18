extern crate rand;

use std::cmp;
use std::f64;
use std::iter;
use std::marker::PhantomData;
use std::ops;
use std::slice;

use rand::Rng;

mod criteria;
mod features;
mod predictors;
mod splitters;
mod tree;
mod vec2d;

/// The side of a split
pub enum Side {
    Left,
    Right,
}

pub trait Data {
    type Sample: Sample;
    fn n_rows(&self) -> usize;
}

pub trait Sample {
    type Theta;
    type Feature;
    fn get_feature(&self, theta: &Self::Theta) -> Self::Feature;
}

pub trait FeatureSet {
    type Item: ?Sized + Sample;
    fn n_samples(&self) -> usize;
    fn get_sample(&self, n: usize) -> &Self::Item;
    fn random_feature<R: Rng>(&self, rng: &mut R) -> <Self::Item as Sample>::Theta;
    fn minmax(&self, theta: &<Self::Item as Sample>::Theta) -> Option<(<Self::Item as Sample>::Feature, <Self::Item as Sample>::Feature)>;

    fn for_each_mut<F: FnMut(&Self::Item)>(&self, f: F);
    #[inline] fn for_each<F: Fn(&Self::Item)>(&self, f: F) { self.for_each_mut(f) }
}

pub trait OutcomeVariable {
    type Item: ?Sized;
    fn n_samples(&self) -> usize;
    fn for_each_mut<F: FnMut(&Self::Item)>(&self, f: F);
    #[inline] fn for_each<F: Fn(&Self::Item)>(&self, f: F) { self.for_each_mut(f) }
}

/// Type has a length
pub trait FixedLength {
    fn len(&self) -> usize;
}

pub trait Shape2D {
    fn n_rows(&self) -> usize;
    fn n_cols(&self) -> usize;
}

impl<'a, T> FixedLength for &'a [T] {
    fn len(&self) -> usize {
        (self as &[T]).len()
    }
}

impl<'a, T> FixedLength for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

/// For comparing splits
pub trait SplitCriterion<'a> {
    type Y: ?Sized;
    type C: ?Sized + cmp::PartialOrd + Copy;
    fn calc_presplit(y: &'a Self::Y) -> Self::C;
    fn calc_postsplit(yl: &'a Self::Y, yr: &'a Self::Y) -> Self::C;
}

/// Prediction of the final Leaf value.
pub trait LeafPredictor
{
    type X: FeatureSet;
    type Y: OutcomeVariable;

    /// predicted value
    fn predict(&self, x: &<Self::X as FeatureSet>::Item) -> <Self::Y as OutcomeVariable>::Item;

    /// fit predictor to data
    fn fit(x: &Self::X, y: &Self::Y) -> Self;
}

/// The probabilistic leaf predictor models uncertainty in the prediction.
pub trait ProbabilisticLeafPredictor: LeafPredictor
{
    /// probability of given output `p(y|x)`
    fn prob(&self, x: &<Self::X as FeatureSet>::Item, y: &<Self::Y as OutcomeVariable>::Item) -> f64;
}

/// Splits data at a tree node. This is a marker trait, shared by more specialized Splitters.
pub trait Splitter {
    type X: FeatureSet;
    fn new_random<R: Rng>(x: &Self::X, rng: &mut R) -> Self;
}

/// Assigns a sample to either side of the split.
pub trait DeterministicSplitter: Splitter {
    //fn split(&self, f: &<Self::F as FeatureSet>::Sample::Output) -> Side;
    fn split(&self, f: &<Self::X as FeatureSet>::Item) -> Side;
}

/// Assigns a sample to both sides of the split with some probability each.
pub trait ProbabilisticSplitter: Splitter {
    /// Probability that the sample belongs to the left side of the split
    fn p_left(&self, f: &<Self::X as FeatureSet>::Item) -> f64;

    /// Probability that the sample belongs to the right side of the split
    fn p_right(&self, f: &<Self::X as FeatureSet>::Item) -> f64 { 1.0 - self.p_left(f) }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trig() {
    }
}
