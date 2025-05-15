use crate::{evolution::{Adaptation, ESSettings, Individual, Selection}, simulation::{ProblemSettings, SimulationResult}};
use std::fs::{self, File};

pub fn log_individual(run_id: i32, best_solution: &Individual) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/price_matrix.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);

    for (group, prices) in best_solution.prices.0.iter() {
        for (visit, prices) in prices {
            for (t, price) in prices.iter().enumerate() {
                // println!("{}", price);
                writer
                    .write_record(&[
                        run_id.to_string(),
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

pub fn log_event_history(run_id: i32, simulation_result: &SimulationResult, settings: &ProblemSettings) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/event_history.csv")
        .unwrap();
    let mut writer = csv::Writer::from_writer(file);
    for event in simulation_result.event_history.iter() {
        writer
            .write_record(&[
                run_id.to_string(),
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
        "run_id",
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

    
    
    return writer;
}

pub fn init_log_es() -> csv::Writer<File> {

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
            "loss_aversion"
        ])
        .unwrap();
    return writer
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
            "num_evals",
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

pub fn init_log_mab() -> (csv::Writer<File>, csv::Writer<File>) {
    fs::remove_file("./results/mab_log.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });
    fs::remove_file("./results/mab_arms.csv").unwrap_or_else(|e| {
        println!("Error removing file: {}", e);
    });

    let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("./results/mab_arms.csv")
            .unwrap();
    let mut arms_writer = csv::Writer::from_writer(file);
    arms_writer
        .write_record(&["run_id", "t", "group", "best_price"])
        .unwrap();

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./results/mab_log.csv")
        .unwrap();
    let mut mab_log_writer = csv::Writer::from_writer(file);
    mab_log_writer
        .write_record(&["config_id", "run_id", "t", "group", "visit", "price", "reward", "last_action"])
        .unwrap();
    return (arms_writer, mab_log_writer);
}



pub fn log_population<'a>(
    writer: &mut csv::Writer<std::fs::File>,
    population: &Vec<Individual<'a>>,
    generation: i32,
    type_: &str,
    algorithm_settings: &ESSettings,
    problem_settings: &ProblemSettings,
    n_evals: i32,
    run_id: i32,
) {
    for individual in population.iter() {
        writer
            .write_record(&[
                run_id.to_string(),
                generation.to_string(),
                n_evals.to_string(),
                type_.to_string(),
                individual.ind_id.to_string(),
                individual.fitness_score.to_string(),
                individual.simulation_result.avg_regret.to_string(),
                individual.simulation_result.regret.to_string(),
                algorithm_settings.lambda.to_string(),
                algorithm_settings.mu.to_string(),
                algorithm_settings.p.to_string(),
                (if algorithm_settings.selection == Selection::Comma {
                    "comma"
                } else {
                    "plus"
                })
                .to_string(),
                algorithm_settings.mutation_probability.to_string(),
                algorithm_settings.mutation_strength.to_string(),
                (if algorithm_settings.adaptation == Adaptation::RechenbergRule {
                    "rechenberg"
                } else {
                    "none"
                })
                .to_string(),
                problem_settings.lambda.to_string(),
            ])
            .unwrap();
    }
}