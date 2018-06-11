extern crate forester;
extern crate openml;
extern crate rand;

mod common;

use std::fmt;

use rand::{thread_rng, Rng};

use forester::array_ops::Partition;
use forester::data::{SampleDescription, TrainingData};
use forester::dforest::DeterministicForestBuilder;
use forester::dtree::DeterministicTreeBuilder;
use forester::split::{BestRandomSplit, Split};
use forester::categorical::{Categorical, CatCount};

use openml::{OpenML, Array, ArrayCastInto, ArrayCastFrom};

use common::dig_classes::{Digit, ClassCounts};

struct Sample<'a> {
    x: &'a [u8],
    y: Digit,
}

impl<'a> fmt::Debug for Sample<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} : {:?}", self.x, self.y)
    }
}

impl<'a> SampleDescription for Sample<'a> {
    type ThetaSplit = usize;
    type ThetaLeaf = ClassCounts;
    type Feature = u8;
    type Prediction = ClassCounts;

    fn sample_as_split_feature(&self, theta: &Self::ThetaSplit) -> Self::Feature {
        // We use the data columns directly as features
        self.x[*theta]
    }

    fn sample_predict(&self, c: &Self::ThetaLeaf) -> Self::Prediction {
        c.clone()
    }
}

impl<'a> TrainingData<Sample<'a>> for [Sample<'a>] {
    fn n_samples(&self) -> usize {
        self.len()
    }

    fn gen_split_feature(&self) -> usize {
        // The data set has four feature columns
        thread_rng().gen_range(0, 28*28)
    }

    fn train_leaf_predictor(&self) -> ClassCounts {
        // count the number of samples in each class. This is possible
        // because there exists an `impl iter::Sum for ClassCounts`.
        self.iter().map(|sample| sample.y).sum()
    }

    fn partition_data(&mut self, split: &Split<usize, u8>) -> (&mut Self, &mut Self) {
        // partition the data set over the split
        let i = self.partition(|sample| sample.sample_as_split_feature(&split.theta) <= split.threshold);
        // return two disjoint subsets
        self.split_at_mut(i)
    }

    fn split_criterion(&self) -> f64 {
        // This is a classification task, so we use the gini criterion.
        // In the future there will be a function provided by the library for this.
        let counts: ClassCounts = self
            .iter()
            .map(|sample| sample.y)
            .sum();

        let gini = (0..10)
            .map(|c| counts.probability(c))
            .map(|p| p * (1.0 - p))
            .sum();

        gini
    }

    fn feature_bounds(&self, theta: &usize) -> (u8, u8) {
        // find minimum and maximum of a feature
        self.iter()
            .map(|sample| sample.sample_as_split_feature(theta))
            .fold((255, 0),
                  |(min, max), x| {
                      (if x < min {x} else {min},
                       if x > max {x} else {max})
                  })
        //(0, 255)
    }
}

pub fn main() {
    #[cfg(feature = "cpuprofiler")] {
        extern crate cpuprofiler;

        let task = OpenML::new().task(146825).unwrap();
        println!("Task: {}", task.name());

        cpuprofiler::PROFILER.lock().unwrap().start("task.profile").unwrap();

        let measure = task.perform(|x_train, y_train, x_test| {

            let x_train: Array<u8> = x_train.cast_into().unwrap();
            let x_test: Array<u8> = x_test.cast_into().unwrap();

            let mut train: Vec<_> = (0..x_train.n_rows())
                .map(|i| Sample {
                    x: x_train.row(i),
                    y: Digit(*y_train.at(i, 0) as u8)
                })
                .collect();

            println!("Fitting...");
            let forest = DeterministicForestBuilder::new(
                100,
                DeterministicTreeBuilder::new(
                    1000,
                    None,
                    BestRandomSplit::new(10)
                )
            ).fit(&mut train as &mut [_]);

            println!("Predicting...");
            (0..x_test.n_rows())
                .map(|i| {
                    let sample = Sample {
                        x: x_test.row(i),
                        y: Digit(99)
                    };
                    let prediction = forest.predict(&sample);
                    let prediction: Digit = prediction.most_frequent();
                    prediction.as_usize() as f64
                })
                .collect()
        });

        cpuprofiler::PROFILER.lock().unwrap().stop().unwrap();

        println!("{:#?}", measure);
        println!("{:#?}", measure.result());

        println!("Profiling done. Convert the profile with something like");
        println!("  > pprof --callgrind target/release/examples/profile2 task.profile > task.prof");
        println!("Or view it with\n  > pprof --gv target/release/examples/profile2 task.profile");
    }
}
