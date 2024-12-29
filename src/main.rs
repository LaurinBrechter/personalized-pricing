use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Normal, Exp};
use priority_queue::PriorityQueue;
use std::{
    collections::{HashMap, VecDeque}, process::exit, vec
};
use std::cmp::Reverse;

#[derive(Debug)]
struct ProblemSettings {
    n_periods: i32,
    n_groups: i32,
    n_customers: i32,
    tau: f64,
}

struct Solution {
    prices: Vec<Vec<Vec<f64>>>,
    prices_h: HashMap<i32, HashMap<i32, Vec<f64>>>,
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

#[derive(Debug)]
struct SimulationEvent {
    t: f32,
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
        return rng.sample::<f32, _>(distr)
    }
}

fn init_simulation(
    settings: &ProblemSettings,
    customers: &Vec<Customer>,
) -> VecDeque<SimulationEvent> {
    let mut rng = rand::thread_rng();

    
    let mut event_calendar: VecDeque<SimulationEvent> = VecDeque::new();

    for customer in customers {
        let event = SimulationEvent {
            t: customer.next_visit(&mut rng),
            event: "customer_arrival".to_string(),
            customer: customer.group.try_into().unwrap(),
        };
        event_calendar.push_back(event);
    }

    return event_calendar;
}

fn main() {
    let rng = rand::thread_rng();

    // normal dist
    let mut rng = rand::thread_rng();
    let scaling = 100.0;
    let group_sizes = vec![20, 10, 70];
    let group_means = vec![2.0, 5.0, 1.25];
    let mut customers: Vec<Customer> = vec![];
    let settings = ProblemSettings {
        n_customers: 100,
        n_periods: 100,
        n_groups: 3,
        tau: 0.5,
    };

    for i in 0..group_sizes.len() {
        let group_mean = group_means[i];
        let group_size = group_sizes[i];

        for _ in 0..group_size {
            let normal_dist = Normal::new(group_mean * scaling, 5.0).unwrap();
            let wtp0: f64 = rng.sample::<f64, _>(normal_dist);
            customers.push(Customer::new(i.try_into().unwrap(), wtp0, wtp0, &settings));
        }
    }

    let mut event_calendar = init_simulation(&settings, &customers);


    let mut pq = PriorityQueue::new();

    pq.push("Strawberries", Reverse(2.0));
    pq.push("Apples", Reverse(5.0));
    pq.push("Bananas", Reverse(8.0));

    println!("{:?}", pq.pop());
    println!("{:?}", pq.pop());
    println!("{:?}", pq.pop());

    exit(0);

    while !event_calendar.is_empty() {
        let event = event_calendar.pop_front();
        if event.is_none() {
            break;
        }
        let event = event.unwrap();

        if event.t > settings.n_periods as f32 {
            continue;
        }

        let customer = &customers[event.customer as usize];

        let next_visit = customer.next_visit(&mut rng);



        println!("{:?}", event);
    }

    // println!("{:?}", customers);
}
