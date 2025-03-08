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
        lambda: 10,
        mu: 5,
        p: 2,
    };

    let best_solution = evolve_pricing(&settings, &algorithm_settings);

    // Save best solution's prices to CSV
    let mut writer = csv::Writer::from_path("event_history.csv").unwrap();

    // Write header row
    let header = vec!["t", "event", "customer"];
    writer.write_record(&header).unwrap();

    // Write price data
    for event in best_solution.event_history {
        writer
            .write_record(&[
                event.t.to_string(),
                event.event.to_string(),
                event.customer.to_string(),
            ])
            .unwrap();
    }
    writer.flush().unwrap();
}
