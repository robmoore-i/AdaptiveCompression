// Nodes @ social_network/person_0_0.csv
// Edges @ social_network/person_knows_person_0_0.csv

fn social_network_sf1() -> &'static str {
    return "/home/rob/S3G2/ldbc_snb_datagen/social_network_sf1"
}

fn social_network_sf10() -> &'static str {
    return "/home/rob/S3G2/ldbc_snb_datagen/social_network_sf10"
}

fn nodes(social_network: &str) -> String {
    return social_network.to_string() + "/person_0_0.csv"
}

fn edges(social_network: &str) -> String {
    return social_network.to_string() + "/person_knows_person_0_0.csv"
}

pub fn print_sf10_dirs() {
    println!("nodes @ {}", nodes(social_network_sf1()).to_string());
    println!("edges @ {}", edges(social_network_sf1()).to_string());
}