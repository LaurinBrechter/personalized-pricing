use crate::{evolution::Individual, simulation::ProblemSettings};
use std::fs::{self, File};

pub fn log_individual(best_solution: &Individual) {
    let mut writer = csv::Writer::from_path("./results/price_matrix.csv").unwrap();

    for (group, prices) in best_solution.prices.0.iter() {
        for (visit, prices) in prices {
            for (t, price) in prices.iter().enumerate() {
                println!("{}", price);
                writer
                    .write_record(&[
                        group.to_string(),
                        visit.to_string(),
                        t.to_string(),
                        price.to_string(),
                    ])
                    .unwrap();
            }
        }
    }
    writer.flush().unwrap();
}

pub fn log_event_history(best_solution: &Individual, settings: &ProblemSettings) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/event_history.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);
    for event in best_solution.simulation_result.event_history.iter() {
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
                settings.lambda.to_string(),
            ])
            .unwrap();
    }
    writer.flush().unwrap();
}

pub fn init_log() -> csv::Writer<File> {
    fs::remove_file("./results/evolution_log.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });
    fs::remove_file("./results/event_history.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });
    fs::remove_file("./results/price_matrix.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });

    let mut writer = csv::Writer::from_path("./results/price_matrix.csv").unwrap();
    let header = vec!["group", "visit", "t", "price"];
    writer.write_record(&header).unwrap();

    let mut writer = csv::Writer::from_path("./results/event_history.csv").unwrap();
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
        "loss_aversion",
    ];
    writer.write_record(&header).unwrap();

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/evolution_log.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record(&[
            "run_id",
            "generation",
            "n_evals",
            "type",
            "individual",
            "score",
            "avg_regret",
            "regret",
            "lambda",
            "mu",
            "p",
            "selection",
            "mutation_probability",
            "mutation_strength",
            "mutation_strat",
        ])
        .unwrap();
    return writer;
}

pub fn init_log_pso() -> csv::Writer<File> {
    fs::remove_file("./results/pso_log.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/pso_log.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record(&[
            "run_id",
            "particle_id",
            "current_fitness",
            "best_fitness",
            "swarm_size",
            "inertia_weight",
            "cognitive_coefficient",
            "social_coefficient",
        ])
        .unwrap();
    return writer;
}

pub fn init_log_mab() -> csv::Writer<File> {
    fs::remove_file("./results/mab_log.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/mab_log.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record(&["t", "group", "visit", "price", "reward", "last_action"])
        .unwrap();
    return writer;
}
