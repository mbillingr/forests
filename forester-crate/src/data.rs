//! Module for describing data.
//!
//! This module defines the traits required to define data sets for use with the forester crate.

use rand::distributions::range::SampleRange;
use rand::thread_rng;

use array_ops::{Partition, resample};
use criterion::SplitCriterion;
use split::Split;
use split_between::SplitBetween;

/// Data sample used with decision trees
pub trait SampleDescription {
    /// Type used to parametrize split features
    type ThetaSplit: Clone;

    /// Type used to parametrize leaf predictors
    type ThetaLeaf: Clone;

    /// Type of a split feature
    type Feature: PartialOrd + SampleRange + SplitBetween;

    type Target;

    /// Type of predicted values; this can be the same as `Self::Y` (e.g. regression) or
    /// something different (e.g. class probabilities).
    type Prediction;

    /// Get target value of sample
    fn target(&self) -> Self::Target;

    /// Compute the value of a leaf feature for a given sample
    fn sample_as_split_feature(&self, theta: &Self::ThetaSplit) -> Self::Feature;

    /// Compute the leaf prediction for a given sample
    fn sample_predict(&self, w: &Self::ThetaLeaf) -> Self::Prediction;
}

/// Data set that can be used for training decision trees
pub trait TrainingData<Sample>: DataSet<Sample>
    where Sample: SampleDescription
{
    type Criterion: SplitCriterion<Sample::Target>;

    /// Return number of samples in the data set
    fn n_samples(&self) -> usize;

    /// Generate a new split feature (typically, this will be randomized)
    fn gen_split_feature(&self) -> Sample::ThetaSplit;

    /// Return an iterator over all features.
    ///
    /// Data sets where this is not possible (infinite feature space, anyone?) should return `None`
    /// instead, which is also the default impl.
    fn all_split_features(&self) -> Option<Box<Iterator<Item=Sample::ThetaSplit>>> { None }

    /// Train a new leaf predictor
    fn train_leaf_predictor(&self) -> Sample::ThetaLeaf;

    /// Return minimum and maximum value of a feature
    fn feature_bounds(&self, theta: &Sample::ThetaSplit) -> (Sample::Feature, Sample::Feature);
}

/// A data set is a collection of samples.
pub trait DataSet<Sample>
    where Sample: SampleDescription
{
    /// Partition data set in-place according to a split
    fn partition_data(&mut self, split: &Split<Sample::ThetaSplit, Sample::Feature>) -> (&mut Self, &mut Self);

    /// Sort data set in-place by feature
    fn sort_data(&mut self, theta: &Sample::ThetaSplit);

    /// Draw `n` samples from this data set with replacement
    fn bootstrap_resample(&self, n: usize) -> Vec<Sample>;

    /// call `visitor` for each sample in the data set
    fn visit_samples<F: FnMut(&Sample)>(&self, visitor: F);
}

impl<Sample> DataSet<Sample> for [Sample]
    where Sample: SampleDescription + Clone
{
    fn partition_data(&mut self, split: &Split<Sample::ThetaSplit, Sample::Feature>) -> (&mut Self, &mut Self) {
        let i = self.partition(|sample| sample.sample_as_split_feature(&split.theta) <= split.threshold);
        self.split_at_mut(i)
    }

    fn sort_data(&mut self, theta: &Sample::ThetaSplit) {
        self.sort_unstable_by(|a, b| {
            let fa = a.sample_as_split_feature(theta);
            let fb = b.sample_as_split_feature(theta);
            match fa.partial_cmp(&fb) {
                Some(ordering) => ordering,
                None => panic!("Could not compare samples (this is likely caused by a NaN feature"),
            }
        })
    }

    fn bootstrap_resample(&self, n: usize) -> Vec<Sample> {
        resample(self, n, &mut thread_rng())
    }

    fn visit_samples<F: FnMut(&Sample)>(&self, mut visitor: F) {
        for sample in self.iter() {
            visitor(sample);
        }
    }
}
