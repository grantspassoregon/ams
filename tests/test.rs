use aid::{error::Bandage, prelude::Clean};
use address::prelude::{GrantsPassSpatialAddresses, SpatialAddresses, Portable};
use ams::prelude::*;
use geo::algorithm::bool_ops::BooleanOps;
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
    city_addresses()?;
    info!("City addresses successfully read.");

    info!("Reading city limits.");
    let _ = city_limits()?;
    info!("City limits successfully read.");

    info!("Creating city limits view.");
    let _ = city_limits_view()?;
    info!("City limits view successfully created.");

    info!("Creating Lexis Nexis boundary.");
    let ln = lexis_nexis_boundary()?;
    info!("Lexis Nexis boundary successfully created.");
    info!("{:#?}", ln);

    Ok(())
}

fn city_addresses() -> Clean<()> {
    let addr = GrantsPassSpatialAddresses::from_csv("c:/users/erose/geojson/addresses_20240506.csv")?;
    let addr = SpatialAddresses::from(&addr.records[..]);
    addr.save("data/addresses.data")?;
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
