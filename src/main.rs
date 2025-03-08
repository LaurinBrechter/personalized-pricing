use personalized_pricing::evolution::{evolve_pricing, AlgorithmSettings};
use personalized_pricing::simulation::ProblemSettings;
fn main() {
    let scaling = 100.0;
    let group_sizes = vec![20, 10, 30];
    let group_means = vec![2.0, 5.0, 1.25];
    let settings = ProblemSettings {
        n_customers: group_sizes.iter().sum(),
        n_periods: 100,
        n_groups: 3,
        tau: 0.5,
        n_visits: 10,
        scaling,
        group_sizes,
        group_means,
        alpha: 0.88,
        lambda: 2.25,
        eta: 0.0,
    };

    let algorithm_settings = AlgorithmSettings {
        num_generations: 10,
        population_size: 10,
    };

    let best_solution = evolve_pricing(&settings, &algorithm_settings);
    // println!("Best solution: {:?}", best_solution);
}
