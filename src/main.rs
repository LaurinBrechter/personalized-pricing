use personalized_pricing::evolution::Adaptation;
use personalized_pricing::evolution::{evolve_pricing, AlgorithmSettings, Selection};
use personalized_pricing::logging::{
    init_log, init_log_mab, init_log_pso, log_event_history, log_individual,
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
        global_wom_prob: 0.5,
        max_price: 500.0
    };

    let mut es_default_settings = AlgorithmSettings {
        num_generations: 20,
        lambda: 10,
        mu: 5,
        p: 2,
        selection: Selection::Comma,
        mutation_probability: 0.5,
        mutation_strength: 100.0,
        adaptation: Adaptation::None,
        rechenberg_window: 10,
    };
    let mut es_steady_state_settings = AlgorithmSettings {
        num_generations: 1000,
        lambda: 1,
        mu: 1,
        p: 1,
        selection: Selection::Plus,
        mutation_probability: 1.0,
        mutation_strength: 50.0,
        rechenberg_window: 10,
        adaptation: Adaptation::RechenbergRule,
    };
    let pso_settings = PSOSettings {
        num_iterations: 40,
        swarm_size: 25,
        inertia_weight: 0.7,
        cognitive_coefficient: 1.5,
        social_coefficient: 1.5,
    };

    // let mut writer = init_log_mab();
    // let mut mab = MAB::new(
    //     settings.n_groups as usize,
    //     0.0,
    //     500.0,
    //     20,
    //     0.05,
    //     &mut writer,
    // );

    // for _ in 0..100 {
    //     let result = simulate_revenue(&mut mab, &settings);
    //     println!("{:?}", result.revenue);
    //     println!("{:?}", mab.best_reward);
    // }

    let mut writer = init_log();

    let n_iterations = 1000;
    let best_solution = random_search(&settings, n_iterations);

    // // write price matrix to csv
    let n_runs = 1;

    let mut run_id = 0;
    for _ in 0..n_runs {
        let best_solution =
            evolve_pricing(run_id, &settings, &es_steady_state_settings, &mut writer);
        log_individual(&best_solution);
        log_event_history(run_id, &best_solution, &settings);
        run_id += 1;
    }

    let mut writer = init_log_pso();
    let best_solution = optimize_pricing(&settings, &pso_settings, &mut writer);



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
