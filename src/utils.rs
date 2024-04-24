use galileo_types::cartesian::{CartesianPoint2d, Point2d, Rect};
use serde::de::DeserializeOwned;
use std::collections::HashSet;

/// Generic function to deserialize data types from a CSV file.  Called by methods to avoid code
/// duplication.
pub fn from_csv<T: DeserializeOwned + Clone, P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Vec<T>, std::io::Error> {
    let mut records = Vec::new();
    let file = std::fs::File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut dropped = 0;
    for result in rdr.deserialize() {
        match result {
            Ok(record) => records.push(record),
            Err(e) => {
                tracing::info!("Dropping: {:#?}.", e.to_string());
                dropped += 1;
            }
        }
    }
    tracing::info!("{} records dropped.", dropped);

    Ok(records)
}

pub fn point_bounds(point: &Point2d, buffer: f64) -> Rect {
    let xmin = point.x() - buffer;
    let xmax = point.x() + buffer;
    let ymin = point.y() - buffer;
    let ymax = point.y() + buffer;
    Rect::new(xmin, ymin, xmax, ymax)
}

pub fn toggle_select(select: &mut HashSet<usize>, row: usize, response: &egui::Response) {
    if response.clicked() {
        if select.contains(&row) {
            select.remove(&row);
        } else {
            select.insert(row);
        }
    }
}
