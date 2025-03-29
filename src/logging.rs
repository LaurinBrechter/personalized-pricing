use crate::evolution::Individual;
use std::fs::{self, File};

pub fn log_price_matrix(best_solution: &Individual) {
    fs::remove_file("./results/price_matrix.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });

    let mut writer = csv::Writer::from_path("./results/price_matrix.csv").unwrap();

    let header = vec!["group", "visit", "t", "price"];
    writer.write_record(&header).unwrap();

    let mut writer = csv::Writer::from_path("./results/price_matrix.csv").unwrap();
    for (group, prices) in best_solution.prices.iter() {
        for (visit, prices) in prices {
            for (t, price) in prices.iter().enumerate() {
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

pub fn log_event_history(best_solution: &Individual) {
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

    for event in best_solution.event_history.iter() {
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

pub fn init_log_evolution() -> csv::Writer<File> {
    fs::remove_file("./results/evolution_log.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/evolution_log.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record(&[
            "generation",
            "type",
            "individual",
            "score",
            "lambda",
            "mu",
            "p",
            "selection",
            "mutation_probability",
            "mutation_stddev",
        ])
        .unwrap();
    return writer;
}
