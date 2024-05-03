use aid::prelude::Clean;
use ams::prelude::*;
use geo::algorithm::bool_ops::BooleanOps;
use tracing::info;

#[test]
fn integration() -> Clean<()> {
    if tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .try_init()
        .is_ok()
    {};
    info!("Subscriber initialized.");

    info!("Creating Lexis Nexis boundary.");
    let ln = lexis_nexis_boundary();
    info!("Lexis Nexis boundary successfully created.");
    info!("{:#?}", ln);

    Ok(())
}

fn lexis_nexis_boundary() -> Clean<Boundary> {
    info!("Reading city limits.");
    let cl = Boundary::from_shp("c:/users/erose/geojson/city_limits.shp", "City Limits")?;
    info!("City limits successfully read.");
    info!("Reading public safety.");
    let ps = Boundary::from_shp(
        "c:/users/erose/geojson/public_safety_agreement.shp",
        "Public Safety Agreement",
    )?;
    info!("Public safety agreements successfully read.");
    ps.save("data/lexis_nexis_boundary.data")?;
    Ok(Boundary::new(
        "Lexis Nexis Boundary",
        cl.geometry.union(&ps.geometry),
    ))
}
