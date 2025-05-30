use std::fs;

use personalized_pricing::custom::{self, simulate_custom};
use personalized_pricing::evolution::Adaptation;
use personalized_pricing::evolution::{evolve_pricing, ESSettings, Selection};
use personalized_pricing::logging::{
    init_log, init_log_es, init_log_mab, init_log_pso, log_event_history, log_individual
};
use personalized_pricing::mab::{MABSettings, MABStrategy, MAB};
use personalized_pricing::particle_swarm::{optimize_pricing, PSOSettings};
use personalized_pricing::random_search::random_search;
use personalized_pricing::simulation::{simulate_revenue, ProblemSettings};

fn main() {
    let group_sizes = vec![20, 10, 30];
    let mut settings = ProblemSettings {
        n_customers: group_sizes.iter().sum(),
        n_periods: 100,
        n_groups: 3,
        tau: 0.6,
        n_visits: 3,
        scaling: 100.0,
        group_sizes,
        group_means: vec![2.0, 5.0, 1.25],
        max_events: 1000,
        alpha: 0.88,
        lambda: 2.25,
        eta: 0.5,
        clustering_accuracy: 1.0,
        k_neighbors: 2,
        p_intra: 0.2,
        p_inter: 0.1,
        max_price: 700.0,
        num_predicted_groups: 3,
        sigmoid_scale: 200.0,
        wtp_adjustment_amplitude: 0.7
    };

    let mut es_default_settings = ESSettings {
        num_generations: 24,
        lambda: 40,
        mu: 20,
        p: 2,
        selection: Selection::Plus,
        mutation_strength: 50.0,
        adaptation: Adaptation::None,
        rechenberg_window: 10,
        fn_evals: 3,
        resample: false
        // num_period_prices: 10,
        // num_visits: 10,
    };
    let mut es_steady_state_settings = ESSettings {
        num_generations: 1000,
        lambda: 1,
        mu: 1,
        p: 1,
        selection: Selection::Plus,
        mutation_strength: 50.0,
        rechenberg_window: 20,
        adaptation: Adaptation::RechenbergRule,
        fn_evals: 3,
        resample: false
        // num_period_prices: 10,
        // num_visits: 10,
    };
    let mut pso_settings = PSOSettings {
    num_iterations: 100,
        swarm_size: 10,
        inertia_weight_start: 0.7,
        inertia_weight_end: 0.7,
        cognitive_coefficient: 1.5,
        social_coefficient: 1.5,
        fn_evals: 2,
    };
    let mut mab_settings = MABSettings {
        min_price: 0.0,
        max_price: settings.max_price,
        arms_per_group: 30,
        epsilon: 0.05,
        final_epsilon: 0.01,
        n_runs: 1000,
        ucb_param: 2.0,
        strategy: MABStrategy::UCB, // Change to desired strategy
    };

    let mut writer = init_log();


    // fs::remove_file("./results/custom_log.csv").unwrap_or_else(|e| {
    //     println!("Error removing file: {}", e);
    // });

    let mut custom_writer = csv::Writer::from_path("./results/custom_log.csv").unwrap();

    let header = vec![
        "scenario_id",
        "run_id",
        "revenue",
    ];
    custom_writer.write_record(&header).unwrap();

    let (mut arms_writer, mut mab_log_writer) = init_log_mab();



    // for eps in 1..=10 {
    //     mab_settings.epsilon = eps as f64 / 20.0;
    //     mab_settings.strategy = MABStrategy::EpsilonGreedy;
    //     let mut mab = MAB::new(
    //         &settings,
    //         &mab_settings,
    //         &mut mab_log_writer,
    //         0,
    //         eps as usize
    //     );
    //     for _ in 0..1000 {
    //         let result = simulate_revenue(&mut mab, &settings);
    //         mab.run_id += 1;
    //     }
    //     mab.log(&mut arms_writer);
    //     let result = simulate_revenue(&mut mab, &settings);
    //     log_event_history(eps, &result, &settings);
    // }
    // let mut mab = MAB::new(
    //     &settings,
    //     &mab_settings,
    //     &mut mab_log_writer,
    //     0,
    //     0
    // );
    // for _ in 0..1000 {
    //     let result = simulate_revenue(&mut mab, &settings);
    //     mab.run_id += 1;
    // }
    // mab.log(&mut arms_writer);
    // let result = simulate_revenue(&mut mab, &settings);
    // log_event_history(0, &result, &settings);
    // mab_settings.strategy = MABStrategy::EpsilonGreedy;

    let result = simulate_custom(&settings, 0, &mut custom_writer);

    // let mut mab = MAB::new(
    //     &settings,
    //     &mab_settings,
    //     &mut mab_log_writer,
    //     0,
    //     0
    // );
    // for _ in 0..1000 {
    //     let result = simulate_revenue(&mut mab, &settings);
    //     mab.run_id += 1;
    // }
    // mab.log(&mut arms_writer);
    // let result = simulate_revenue(&mut mab, &settings);
    // log_event_history(0, &result, &settings);


    // settings.wtp_adjustment_amplitude = 0.5;
    // settings.k_neighbors = 1;
    let mut mab = MAB::new(
        &settings,
        &mab_settings,
        &mut mab_log_writer,
        0,
        1
    );
    for _ in 0..1000 {
        let result = simulate_revenue(&mut mab, &settings);
        mab.run_id += 1;

        if (mab.run_id > 970) {
            custom_writer.write_record(&[
                "1",
                "1",
                &result.revenue.to_string(),
            ]).unwrap();
        }

    }
    mab.log(&mut arms_writer);
    let result = simulate_revenue(&mut mab, &settings);
    log_event_history(1, &result, &settings);

    


    // let n_iterations = 1000;
    // let best_solution = random_search(&settings, n_iterations, );

    // // // // // write price matrix to csv
    
    // let mut es_writer = init_log_es();
    // let n_runs = 1;
    // let mut run_id = 1;
    // for _ in 0..n_runs {
    //     let (initial_best, best) =
    //         evolve_pricing(run_id, &settings, &es_default_settings, &mut es_writer);
    //     log_individual("initial", run_id, &initial_best);
    //     log_individual("best", run_id, &best);
    //     log_event_history(run_id, &best.simulation_result, &settings);
    //     println!("Best solution: {:?}", best.fitness_score);
    //     run_id += 1;
    // }
    // es_default_settings.selection = Selection::Comma;
    // for _ in 0..n_runs {
    //     let (initial_best, best) =
    //         evolve_pricing(run_id, &settings, &es_default_settings, &mut es_writer);
    //     log_individual("initial", run_id, &initial_best);
    //     log_individual("best", run_id, &best);
    //     log_event_history(run_id, &best.simulation_result, &settings);
    //     println!("Best solution: {:?}", best.fitness_score);
    //     run_id += 1;
    // }

    


    // es_steady_state_settings.fn_evals = 1;

    // for _ in 0..n_runs {
    //     let best_solution =
    //         evolve_pricing(run_id, &settings, &es_steady_state_settings, &mut es_writer);
    //     log_individual(run_id, &best_solution);
    //     log_event_history(run_id, &best_solution.simulation_result, &settings);
    //     run_id += 1;
    // }
    


    // let mut writer = init_log_pso();
    // let mut run_id = 0;
    // let mut n_runs = 5;
    // for _ in 0..n_runs {
    //     let best_solution = optimize_pricing(run_id, &settings, &pso_settings, &mut writer);
    //     run_id += 1;
    // }
    // pso_settings.inertia_weight_end = 0.3;
    // pso_settings.inertia_weight_start = 0.9;
    // for _ in 0..n_runs {
    //     let best_solution = optimize_pricing(run_id, &settings, &pso_settings, &mut writer);
    //     run_id += 1;
    // }
    // run_id += 1;
    // pso_settings.inertia_weight = 0.3;
    // for _ in 0..n_runs {
    //     let best_solution = optimize_pricing(run_id, &settings, &pso_settings, &mut writer);
    //     run_id += 1;
    // }

    // settings.lambda = 5.0;
    // for _ in 0..n_runs {
    //     let best_solution =
    //         evolve_pricing(run_id, &settings, &es_steady_state_settings, &mut writer);
    //     log_individual(&best_solution);
    //     log_event_history(run_id, &best_solution, &settings);
    //     run_id += 1;
    // }
    // es_steady_state_settings.adaptation = Adaptation::None;
    // for run_id in 0..n_runs {
    //     let best_solution =
    //         evolve_pricing(run_id, &settings, &es_steady_state_settings, &mut writer);
    // }
    // algorithm_settings.selection = Selection::Plus;
    // let best_solution_plus = evolve_pricing(&settings, &es_default_settings, &mut writer);

    // write price matrix to csv
    // Save best solution's prices to CSV
}
