use std::fs;

use personalized_pricing::evolution::{evolve_pricing, AlgorithmSettings, Selection};
use personalized_pricing::simulation::ProblemSettings;
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
    };

    let algorithm_settings = AlgorithmSettings {
        num_generations: 20,
        lambda: 10,
        mu: 5,
        p: 2,
        selection: Selection::Comma,
        mutation_probability: 0.5,
        mutation_stddev: 100.0,
    };

    let best_solution = evolve_pricing(&settings, &algorithm_settings);

    // Save best solution's prices to CSV

    fs::remove_file("./results/event_history.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });

    let mut writer = csv::Writer::from_path("./results/event_history.csv").unwrap();

    // Write header row
    let header = vec![
        "t",
        "event",
        "customer",
        "customer_wtp",
        "customer_max_wtp",
        "group",
        "price",
        "irp",
        "erp",
        "rp",
    ];
    writer.write_record(&header).unwrap();

    println!(
        "Best Solution {:?}",
        best_solution.get_price(0 as usize, 0 as usize, 0 as usize)
    );

    // Write price data
    for event in best_solution.event_history {
        writer
            .write_record(&[
                event.t.to_string(),
                event.event.to_string(),
                event.customer.to_string(),
                event.customer_wtp.to_string(),
                event.customer_max_wtp.to_string(),
                event.group.to_string(),
                event.price.to_string(),
                event.irp.to_string(),
                event.erp.to_string(),
                event.rp.to_string(),
            ])
            .unwrap();
    }
    writer.flush().unwrap();
}
