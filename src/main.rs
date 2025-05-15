use personalized_pricing::custom::simulate_custom;
use personalized_pricing::evolution::Adaptation;
use personalized_pricing::evolution::{evolve_pricing, ESSettings, Selection};
use personalized_pricing::logging::{
    init_log, init_log_es, init_log_mab, init_log_pso, log_event_history, log_individual
};
use personalized_pricing::mab::MAB;
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
        n_visits: 10,
        scaling: 100.0,
        group_sizes,
        group_means: vec![2.0, 5.0, 1.25],
        max_events: 1000,
        alpha: 0.88,
        lambda: 2.25,
        eta: 0.0,
        clustering_accuracy: 1.0,
        k_neighbors: 6,
        p_intra: 0.1,
        p_inter: 0.3,
        global_wom_prob: 0.0,
        max_price: 700.0,
        num_predicted_groups: 3,
        sigmoid_scale: 1000.0
    };

    let mut es_default_settings = ESSettings {
        num_generations: 100,
        lambda: 10,
        mu: 10,
        p: 2,
        selection: Selection::Plus,
        mutation_probability: 0.5,
        mutation_strength: 100.0,
        adaptation: Adaptation::None,
        rechenberg_window: 10,
        // num_period_prices: 10,
        // num_visits: 10,
    };
    let mut es_steady_state_settings = ESSettings {
        num_generations: 1000,
        lambda: 1,
        mu: 1,
        p: 1,
        selection: Selection::Plus,
        mutation_probability: 1.0,
        mutation_strength: 50.0,
        rechenberg_window: 20,
        adaptation: Adaptation::RechenbergRule,
        // num_period_prices: 10,
        // num_visits: 10,
    };
    let mut pso_settings = PSOSettings {
        num_iterations: 100,
        swarm_size: 10,
        inertia_weight: 0.7,
        cognitive_coefficient: 1.5,
        social_coefficient: 1.5,
    };

    let mut writer = init_log();
    // let result = simulate_custom(&settings);
    let (mut arms_writer, mut mab_log_writer) = init_log_mab();
    
    
    let config_id = 0;
    let mut mab = MAB::new(
        &settings,
        0.0,
        settings.max_price,
        30,
        0.05,
        &mut mab_log_writer,
        0,
        0
    );

        
    for _ in 0..1000 {
        let result = simulate_revenue(&mut mab, &settings);
        mab.run_id += 1;
    }
    mab.log(&mut arms_writer);
    let mut mab = MAB::new(
        &settings,
        0.0,
        settings.max_price,
        30,
        0.05,
        &mut mab_log_writer,
        0,
        1
    );
    settings.clustering_accuracy = 0.7;
    for _ in 0..1000 {
        let result = simulate_revenue(&mut mab, &settings);
        mab.run_id += 1;
    }
    mab.log(&mut arms_writer);
    


    // let n_iterations = 1000;
    // let best_solution = random_search(&settings, n_iterations, );

    // // // // write price matrix to csv
    
    // let mut es_writer = init_log_es();
    // let n_runs = 1;
    // let mut run_id = 0;
    // for _ in 0..n_runs {
    //     let best_solution =
    //         evolve_pricing(run_id, &settings, &es_steady_state_settings, &mut es_writer);
    //     log_individual(run_id, &best_solution);
    //     log_event_history(run_id, &best_solution.simulation_result, &settings);
    //     println!("Best solution: {:?}", best_solution.fitness_score);
    //     run_id += 1;
    // }

    // settings.num_predicted_groups = 3;

    // for _ in 0..n_runs {
    //     let best_solution =
    //         evolve_pricing(run_id, &settings, &es_steady_state_settings, &mut writer);
    //     log_individual(&best_solution);
    //     log_event_history(run_id, &best_solution, &settings);
    //     run_id += 1;
    // }


    // let mut writer = init_log_pso();
    // let best_solution = optimize_pricing(run_id, &settings, &pso_settings, &mut writer);

    // pso_settings.inertia_weight = 2.0;

    // for run_id in 0..n_runs {
    //     let best_solution = optimize_pricing(run_id, &settings, &pso_settings, &mut writer);
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
