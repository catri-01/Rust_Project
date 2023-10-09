use geoutils::Location;
use reqwest::Result;
use serde::Deserialize;
use std::io;
use std::result::Result as StdResult;

/// Define a struct to represent User (customer, valet)
struct User {
    name: String,
    lat: f64,
    lng: f64,
}

/// Define a generic function to get user input 
fn collect_input<T: std::str::FromStr>(prompt: &str) -> T {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim().parse() {
            Ok(value) => return value,
            Err(_) => continue,
        }
    }
}

/// get customers details including their location 
fn get_customers() -> StdResult<Vec<User>, String> {
    let mut location_vec = Vec::<User>::new();

    let customers_count = collect_input("How many customers?");
    for i in 1..=customers_count {
        let name = collect_input(&format!("Customer-{} name please?", i));
        let lat = collect_input(&format!("Hi! {}, your geo lat for food please?", name));
        let lng = collect_input(&format!("and your geo lng for food please?"));
        location_vec.push(User {name, lat, lng});
    }
    Ok(location_vec)
}

#[derive(Deserialize)]
struct ValetLocation {
    latitude: f64,
    longitude: f64,
}

// get location input of valet via Location API
#[tokio::main]
async fn get_valet() -> Result<User> {
    let name = get_input(&format!("Valet, your delivery name please?"));

    dotenv::from_path("./.env").expect("Failed to  load .env file");
    let url = std::env::var("VALET_LOCATION_URL").expect("URL var not found");

    let response_body = reqwest::get(url).await?;
    let valet_location = response_body.json::<ValetLocation>().await?;

    // println!("{:?}", response_body);
    // println!("{:?}", valet_location);

    let valet = User {
        name,
        lat: valet_location.latitude,
        lng: valet_location.longitude,
    };
    Ok(valet)
}

// get the final (sorted) delivery list (with proximity counted)
fn get_sorted_proximited_list(customers: &Vec<User>, valet: &User) -> Option<Vec<(String, f64)>> {
    // create a mutable vector of (name, proximity);
    let mut customers_proxi_from_valet = Vec::<(String, f64)>::new();

    // create a geoutils based `Location` of valet
    let valet_loc = Location::new(valet.lat, valet.lng);

    //loop along the customers and push the location proximity for each customer
    for customer in customers {
        // Create a geoutils based `Location` of customer
        let customer_loc = Location::new(customer.lat, customer.lng);

        // calculate the proximity 
        let distante = customer_loc.distance_to(&valet_loc).unwrap();
        let distante_kms = distante.meters() / 1000f64;

        // push the customer w element 
        customers_proxi_from_valet.push((customer.name.clone(), distante_kms));
    }

    // sort the customers proximity (from valet) list
    let _ = customers_proxi_from_valet.sort_by(|a,b| {
        if let Some(ordering) = a.1.partial_cmp(&b.1) {
            ordering
        } else {
            std::cmp::Ordering::Equal
        }
    });
    Some(customers_proxi_from_valet)
}

// `take_order` function
fn take_order() {
    loop {
        let customers = get_customers().unwrap();
        let valet = get_valet().unwrap();

        let delivery_list = get_sorted_proximited_list(&customers, &valet).unwrap();

        println!(
            "Congratulations! {} would delivery food order in this sequence:",
            valet.name
        );
        for d in delivery_list {
            println!("-{}", d.0)
        }

        let play_again: String = collect_input("Take orders again? (y/n)");

        // if input is anything other than "y", it breaks
        if play_again.to_ascii_lowercase() != "y" {
            break;
        }
    }
}

fn main() {
    take_order();
}
