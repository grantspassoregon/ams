use address::{
    geo::SpatialAddresses,
    prelude::{Addresses, Portable},
};
use aid::{error::Bandage, prelude::Clean};
use ams::prelude::*;
use geo::algorithm::bool_ops::BooleanOps;
use geo::algorithm::contains::Contains;
use rayon::prelude::*;
use tracing::info;

#[test]
fn integration() -> Clean<()> {
    if tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .is_ok()
    {};
    info!("Subscriber initialized.");

    info!("Reading city addresses.");
    let addr = city_addresses()?;
    info!("City addresses successfully read.");

    info!("Reading county addresses.");
    county_addresses()?;
    info!("County addresses successfully read.");

    info!("Reading city limits.");
    let _ = city_limits()?;
    info!("City limits successfully read.");

    info!("Creating city limits view.");
    let _ = city_limits_view()?;
    info!("City limits view successfully created.");

    info!("Creating Lexis Nexis boundary.");
    let boundary = lexis_nexis_boundary()?;
    info!("Lexis Nexis boundary successfully created.");
    info!("{:#?}", boundary);

    info!("Creating Lexis Nexis record.");
    let _ = lexis_nexis(&addr, &boundary);

    Ok(())
}

fn city_addresses() -> Clean<SpatialAddresses> {
    let addr = address::import::GrantsPassSpatialAddresses::from_csv(
        "c:/users/erose/repos/address/tests/test_data/city_addresses_20240630.csv",
    )?;
    let addr = SpatialAddresses::from(&addr[..]);
    addr.save("data/addresses.data")?;
    Ok(addr)
}

fn county_addresses() -> Clean<()> {
    let addr = address::import::JosephineCountySpatialAddresses2024::from_csv(
        "c:/users/erose/repos/address/tests/test_data/county_addresses_20240701.csv",
    )?;
    let mut addr = SpatialAddresses::from(&addr[..]);
    addr.standardize();
    addr.save("data/county_addresses.data")?;
    Ok(())
}

fn city_limits() -> Clean<Boundary> {
    let cl = Boundary::from_shp_z("c:/users/erose/geojson/city_limits.shp", "City Limits")?;
    cl.save("data/city_limits.data")?;
    Ok(cl)
}

fn city_limits_view() -> Clean<BoundaryView> {
    let cl = Boundary::from_shp_z("c:/users/erose/geojson/city_limits.shp", "City Limits")?;
    let cl = BoundaryView::from_shp(&cl);
    if let Some(shp) = cl {
        shp.save("data/city_limits_view.data")?;
        Ok(shp)
    } else {
        Err(Bandage::Hint("Failed to create boundary view.".to_string()))
    }
}

fn lexis_nexis_boundary() -> Clean<Boundary> {
    info!("Reading city limits.");
    let cl = Boundary::from_shp_z("c:/users/erose/geojson/city_limits.shp", "City Limits")?;
    info!("City limits successfully read.");
    info!("Reading public safety.");
    let ps = Boundary::from_shp(
        "c:/users/erose/geojson/public_safety_agreement.shp",
        "Public Safety Agreement",
    )?;
    info!("Public safety agreements successfully read.");
    let ln = Boundary::new("Lexis Nexis Boundary", cl.geometry.union(&ps.geometry));
    ln.save("data/lexis_nexis_boundary.data")?;
    Ok(ln)
}

// The Lexis Nexis boundary gets saved in state.data.  Delete state.data after running this test to
// use the new boundary in the program.
fn lexis_nexis(addresses: &SpatialAddresses, boundary: &Boundary) -> Clean<()> {
    tracing::info!("Running LexisNexis.");
    let mut records = Vec::new();
    let mut other = Vec::new();
    let ap = AddressPoints::from(addresses);
    let gp = ap
        .par_iter()
        .map(|v| v.geo_point())
        .collect::<Vec<geo::geometry::Point>>();
    for (i, pt) in gp.iter().enumerate() {
        // info!("Point: {:#?}", pt);
        // info!("Contained: {}", self.boundary.geometry.contains(pt));
        if boundary.geometry.contains(pt) {
            records.push(addresses[i].clone());
        } else {
            other.push(addresses[i].clone());
        }
    }
    let records = SpatialAddresses::from(&records[..]);
    tracing::info!("Inclusion records: {}", records.len());
    let other = SpatialAddresses::from(&other[..]);
    tracing::info!("Exclusion records: {}", other.len());
    let lexis = records.lexis_nexis(&other)?;
    tracing::info!("LexisNexis records: {}", lexis.len());
    Ok(())
}
