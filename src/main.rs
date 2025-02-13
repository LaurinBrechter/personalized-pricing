use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Exp, Normal};
use std::cmp::Reverse;
use std::collections::HashMap;

#[derive(Debug)]
struct ProblemSettings {
    n_visits: i32,
    n_periods: i32,
    n_groups: i32,
    n_customers: i32,
    tau: f64,
}

struct Solution {
    // prices: Vec<Vec<Vec<f64>>>,
    prices: HashMap<i32, HashMap<i32, Vec<f64>>>,
}

impl Solution {
    fn new(n_visits: i32, n_periods: i32, n_groups: i32) -> Self {
        let mut rng = rand::thread_rng();
        let mut prices = HashMap::new();

        for w in 0..n_visits {
            let mut group_map = HashMap::new();
            for g in 0..n_groups {
                let mut period_prices = Vec::new();
                for _ in 0..n_periods {
                    period_prices.push(rng.gen_range(0.0..100.0));
                }
                group_map.insert(g, period_prices);
            }
            prices.insert(w, group_map);
        }

        Self { prices }
    }

    fn get_price(&self, w: i32, g: i32, t: i32) -> f64 {
        self.prices[&w][&g][t as usize]
    }
}

#[derive(Debug)]
struct Customer<'a> {
    group: i32,
    irp: f64,
    erp: f64,
    rp: f64,
    wtp: f64,
    price_hist: Vec<f64>,
    visit_hist: Vec<i32>,
    settings: &'a ProblemSettings,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct SimulationEvent {
    t: OrderedFloat<f32>,
    event: String,
    customer: i32,
}

impl<'a> Customer<'a> {
    fn new(group: i32, wtp_min: f64, wtp_max: f64, settings: &'a ProblemSettings) -> Self {
        let mut rng = rand::thread_rng();
        let wtp: f64 = rng.gen_range(wtp_min..=wtp_max);
        Customer {
            group,
            irp: -1.0,
            erp: -1.0,
            rp: -1.0,
            wtp: wtp,
            price_hist: vec![],
            visit_hist: vec![],
            settings: settings,
        }
    }
    fn update_irp(&mut self, new_price: f64) {
        self.irp = self.irp + self.settings.tau * (new_price - self.irp)
    }
    fn update_erp(&mut self, other_customers: Vec<&Customer>) {
        let mut rng = rand::thread_rng();
        let mut ref_prices: Vec<f64> = vec![];
        for cust in other_customers {
            if (rng.gen::<f64>() < 0.2) {
                if let Some(&price) = cust.price_hist.last() {
                    ref_prices.push(price);
                }
            }
        }
    }
    fn update_rp(&mut self) {}
    fn update_wtp(&mut self) {}
    fn next_visit(&self, rng: &mut ThreadRng) -> f32 {
        let distr = Exp::new(0.1).unwrap();
        return rng.sample::<f32, _>(distr);
    }
}

fn init_simulation(
    settings: &ProblemSettings,
    customers: &Vec<Customer>,
) -> PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> {
    let mut rng = rand::thread_rng();

    let mut event_calendar: PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> =
        PriorityQueue::new();

    for customer in customers {
        let event = SimulationEvent {
            t: OrderedFloat(customer.next_visit(&mut rng)),
            event: "customer_arrival".to_string(),
            customer: customer.group.try_into().unwrap(),
        };
        event_calendar.push(event.clone(), Reverse(event.t));
    }

    return event_calendar;
}

fn main() {
    let rng = rand::thread_rng();

    // normal dist
    let mut rng = rand::thread_rng();
    let scaling = 100.0;
    let group_sizes = vec![2, 1, 3];
    let group_means = vec![2.0, 5.0, 1.25];
    let mut customers: Vec<Customer> = vec![];
    let settings = ProblemSettings {
        n_customers: group_sizes.iter().sum(),
        n_periods: 1,
        n_groups: 3,
        tau: 0.5,
        n_visits: 10,
    };

    let solution = Solution::new(settings.n_visits, settings.n_periods, settings.n_groups);

    println!("{:?}", solution.prices);

    for i in 0..group_sizes.len() {
        let group_mean = group_means[i];
        let group_size = group_sizes[i];

        for _ in 0..group_size {
            let normal_dist = Normal::new(group_mean * scaling, 5.0).unwrap();
            let wtp0: f64 = rng.sample::<f64, _>(normal_dist);
            customers.push(Customer::new(i.try_into().unwrap(), wtp0, wtp0, &settings));
        }
    }

    println!("{:?}\n", customers);

    let mut event_calendar = init_simulation(&settings, &customers);

    let mut revenue = 0.0;
    while !event_calendar.is_empty() {
        let event = event_calendar.pop();
        if event.is_none() {
            break;
        }
        let event = event.unwrap();

        println!("{:?}", event);

        if event.0.t > OrderedFloat(settings.n_periods as f32) {
            continue;
        }

        let customer = &customers[event.0.customer as usize];

        let next_visit = customer.next_visit(&mut rng);

        let price = solution.get_price(
            customer.price_hist.len() as i32, // wth visit
            customer.group, // gth group
            event.0.t.0 as i32, // tth period
        );

        if price < customer.wtp {
            println!("Customer {} bought at price {}", customer.group, price);
            revenue += price;
            
            // remove from simulation
            customers.remove(event.0.customer as usize);
        }
    }

    println!("Total revenue: {}", revenue);

    // println!("{:?}", customers);
}

