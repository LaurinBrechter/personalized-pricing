use crate::simulation::ProblemSettings;
use rand::Rng;

pub fn create_network(settings: &ProblemSettings) -> Vec<Vec<i32>> {
    let mut rng = rand::thread_rng();
    let mut network = vec![vec![]; settings.n_customers as usize];

    // Calculate starting index for each group
    let mut group_starts = vec![0];
    let mut total = 0;
    for &size in &settings.group_sizes {
        total += size;
        group_starts.push(total);
    }

    // Create intra-group connections (similar to Watts-Strogatz)
    for group in 0..settings.n_groups {
        let group_start = group_starts[group as usize];
        let group_end = group_starts[group as usize + 1];
        let group_size = settings.group_sizes[group as usize];

        // Create initial ring lattice
        for i in group_start..group_end {
            for k in 1..=settings.k_neighbors / 2 {
                let forward = (i - group_start + k) % group_size + group_start;
                let backward = (i - group_start + group_size - k) % group_size + group_start;
                network[i as usize].push(forward as i32);
                network[i as usize].push(backward as i32);
            }
        }

        // Rewiring within group
        for i in group_start..group_end {
            for j in 0..network[i as usize].len() {
                if rng.gen::<f64>() < settings.p_intra {
                    let new_target = rng.gen_range(group_start..group_end) as i32;
                    network[i as usize][j as usize] = new_target;
                }
            }
        }
    }

    // Add inter-group connections
    for group in 0..settings.n_groups {
        let group_start = group_starts[group as usize];
        let group_end = group_starts[group as usize + 1];

        for i in group_start..group_end {
            for g in 0..settings.n_groups {

                if g == group {
                    continue;
                }

                if rng.gen::<f64>() < settings.p_inter {
                    let other_start = group_starts[g as usize];
                    let other_end = group_starts[g as usize + 1];
                    let target = rng.gen_range(other_start..other_end) as i32;
                    network[i as usize].push(target);
                    network[target as usize].push(i as i32);
                }
            }
        }
    }

    network
}
