extern crate forester;
extern crate image;
extern crate rand;

mod common;

use std::f64::consts::PI;
use std::fs::File;

use rand::{thread_rng, Rng};

use forester::array_ops::Partition;
use forester::categorical::CatCount;
use forester::data::{SampleDescription, TrainingData};
use forester::dforest::DeterministicForestBuilder;
use forester::dtree::DeterministicTreeBuilder;
use forester::split::{BestRandomSplit, Split};

use common::rgb_classes::{ClassCounts, Classes};

struct Sample<Y> {
    x: [f64; 2],
    y: Y,
}

impl<Y> SampleDescription for Sample<Y> {
    type ThetaSplit = usize;
    type ThetaLeaf = ClassCounts;
    type Feature = f64;
    type Prediction = ClassCounts;

    fn sample_as_split_feature(&self, theta: &Self::ThetaSplit) -> Self::Feature {
        self.x[*theta]
    }

    fn sample_predict(&self, w: &Self::ThetaLeaf) -> Self::Prediction {
        w.clone()
    }
}

impl TrainingData<Sample<Classes>> for [Sample<Classes>] {
    fn n_samples(&self) -> usize {
        self.len()
    }

    fn gen_split_feature(&self) -> usize {
        thread_rng().gen_range(0, 2)
    }

    fn train_leaf_predictor(&self) -> ClassCounts {
        self.iter().map(|sample| sample.y).sum()
    }

    fn partition_data(&mut self, split: &Split<usize, f64>) -> (&mut Self, &mut Self) {
        let i = self.partition(|sample| sample.sample_as_split_feature(&split.theta) <= split.threshold);
        self.split_at_mut(i)
    }

    fn split_criterion(&self) -> f64 {
        let counts: ClassCounts = self.iter().map(|sample| sample.y).sum();
        let p_red = counts.probability(Classes::Red);
        let p_green = counts.probability(Classes::Green);
        let p_blue = counts.probability(Classes::Blue);
        let gini = p_red * (1.0 - p_red) + p_green * (1.0 - p_green) + p_blue * (1.0 - p_blue);
        gini
    }

    fn feature_bounds(&self, theta: &usize) -> (f64, f64) {
        self.iter()
            .map(|sample| sample.sample_as_split_feature(theta))
            .fold((std::f64::INFINITY, std::f64::NEG_INFINITY),
                         |(min, max), x| {
                             (if x < min {x} else {min},
                              if x > max {x} else {max})
        })
    }
}

const N_SAMPLES: usize = 1000;

const N_ROWS: u32 = 300;
const N_COLS: u32 = 300;

/// function used to generate training data
fn spiral(r: f64, c: u8) -> f64 {
    let phi = r + PI * 2.0 * c as f64 / 3.0;
    phi
}

/// generate a Vec<f64> of linearly spaced values
fn linspace(l: f64, h: f64, n: usize) -> Vec<f64> {
    let di = (h - l) / (n - 1) as f64;
    (0..n).map(|i| l + di * i as f64).collect()
}

/// generate a Vec<f64> of uniformly distributed random values
fn randspace(l: f64, h: f64, n: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    (0..n).map(|_| rng.gen_range(l, h)).collect()
}

fn main() {
    // generate data points
    let y0: Vec<Classes> = vec![Classes::Red, Classes::Green, Classes::Blue].into_iter().cycle().take(N_SAMPLES).collect();
    let r = randspace(0.0, 6.0, N_SAMPLES);
    let phi: Vec<_> = r.iter().map(|&r| r).zip(y0.iter())
        .map(|(r, &c)| spiral(r, c as u8) + thread_rng().gen::<f64>() * PI * 2.0 / 3.0)
        .collect();

    let x0: Vec<_> = r.into_iter()
        .zip(phi.into_iter())
        .map(|(r, phi)| [phi.sin() * r, phi.cos() * r]).collect();

    // convert data to data set for fitting
    let mut data: Vec<_> = x0.iter().zip(y0.iter()).map(|(&x, &y)| Sample{x, y}).collect();

    // configure and fit random forest
    println!("Fitting...");
    let forest = DeterministicForestBuilder::new(
        100,  // 100 trees
        DeterministicTreeBuilder::new(
            10,  // don't split less than 10 samples
            None,
            BestRandomSplit::new(1)
        )
    ).fit(&mut data as &mut [_]);

    // generate test data
    let x_grid = linspace(-4.0, 4.0, N_ROWS as usize);
    let y_grid = linspace(-4.0, 4.0, N_COLS as usize);

    // predict
    println!("Predicting...");
    let mut z = Vec::with_capacity(3 * (N_ROWS * N_COLS) as usize);
    for &y in y_grid.iter() {
        for &x in x_grid.iter() {
            let sx = [x, y];
            let c = forest.predict(&Sample{x: sx, y: ()});
            z.push(c.probability(Classes::Red));
            z.push(c.probability(Classes::Green));
            z.push(c.probability(Classes::Blue));
        }
    }

    // plot original samples
    for xy in x0 {
        let (x, y) = (xy[0], xy[1]);
        let x = (N_COLS as f64 * (x + 4.0) / 8.0) as usize;
        let y = (N_ROWS as f64 * (y + 4.0) / 8.0) as usize;

        if x <= 0 || y <= 0 || x >= N_COLS as usize - 1 || y >= N_ROWS as usize - 1 {
            continue
        }

        z[(x + y * N_COLS as usize) * 3 + 0] *= 0.5;
        z[(x + y * N_COLS as usize) * 3 + 1] *= 0.5;
        z[(x + y * N_COLS as usize) * 3 + 2] *= 0.5;

        z[(x + 1 + y * N_COLS as usize) * 3 + 0] *= 0.5;
        z[(x + 1 + y * N_COLS as usize) * 3 + 1] *= 0.5;
        z[(x + 1 + y * N_COLS as usize) * 3 + 2] *= 0.5;

        z[(x - 1 + y * N_COLS as usize) * 3 + 0] *= 0.5;
        z[(x - 1 + y * N_COLS as usize) * 3 + 1] *= 0.5;
        z[(x - 1 + y * N_COLS as usize) * 3 + 2] *= 0.5;

        z[(x + (y + 1) * N_COLS as usize) * 3 + 0] *= 0.5;
        z[(x + (y + 1) * N_COLS as usize) * 3 + 1] *= 0.5;
        z[(x + (y + 1) * N_COLS as usize) * 3 + 2] *= 0.5;

        z[(x + (y - 1) * N_COLS as usize) * 3 + 0] *= 0.5;
        z[(x + (y - 1) * N_COLS as usize) * 3 + 1] *= 0.5;
        z[(x + (y - 1) * N_COLS as usize) * 3 + 2] *= 0.5;
    }

    // store result
    let z: Vec<u8> = z.into_iter().map(|i| (i * 255.0) as u8).collect();
    let encoder = image::png::PNGEncoder::new(File::create("extra_trees_classifier.png").unwrap());
    encoder.encode(&z, N_COLS, N_ROWS, image::ColorType::RGB(8)).unwrap();
}
