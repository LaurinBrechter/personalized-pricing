use personalized_pricing::evolution::{evolve_pricing, AlgorithmSettings, Selection};
use personalized_pricing::logging::{init_log_evolution, log_event_history, log_price_matrix};
use personalized_pricing::simulation::ProblemSettings;
use std::fs;
fn main() {
    let group_sizes = vec![20, 10, 30];
    let settings = ProblemSettings {
        n_customers: group_sizes.iter().sum(),
        n_periods: 100,
        n_groups: 3,
        tau: 0.5,
        n_visits: 10,
        scaling: 100.0,
        group_sizes,
        group_means: vec![2.0, 5.0, 1.25],
        max_events: 1000,
        alpha: 0.88,
        lambda: 2.25,
        eta: 0.0,
        clustering_accuracy: 0.2,
        k_neighbors: 6,
        p_intra: 0.1,
        p_inter: 0.3,
    };

    let mut algorithm_settings = AlgorithmSettings {
        num_generations: 20,
        lambda: 10,
        mu: 5,
        p: 2,
        selection: Selection::Comma,
        mutation_probability: 0.5,
        mutation_stddev: 100.0,
    };

    let mut writer = init_log_evolution();

    let best_solution = evolve_pricing(&settings, &algorithm_settings, &mut writer);
    algorithm_settings.selection = Selection::Plus;
    let best_solution_plus = evolve_pricing(&settings, &algorithm_settings, &mut writer);

    // write price matrix to csv

    log_price_matrix(&best_solution);
    log_event_history(&best_solution);
    // Save best solution's prices to CSV
}
