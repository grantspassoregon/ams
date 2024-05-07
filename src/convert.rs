// use crate::prelude::*;
use galileo::galileo_types::cartesian::{CartesianPoint2d, Point2d};
use galileo::galileo_types::impls::ClosedContour;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::geometry::Rect;
use geo_types::{Coord, LineString, MultiPolygon, Point, Polygon};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use shapefile::record::traits::HasXY;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Convert<T: Debug + Clone>(pub T);

impl<T: Debug + Clone> Convert<T> {
    pub fn new(from: T) -> Self {
        Convert(from)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl Convert<MultiPolygon> {
    pub fn multipolygon(self) -> galileo::galileo_types::impls::MultiPolygon<Point2d> {
        let conv = self
            .0
            .iter()
            .map(|v| Convert::new(v.clone()))
            .collect::<Vec<Convert<Polygon>>>();
        let parts = conv
            .par_iter()
            .cloned()
            .map(|v| v.polygon())
            .collect::<Vec<galileo::galileo_types::impls::Polygon<Point2d>>>();
        galileo::galileo_types::impls::MultiPolygon { parts }
    }

    pub fn bounded_multipolygon(
        self,
    ) -> (
        galileo::galileo_types::impls::MultiPolygon<Point2d>,
        galileo::galileo_types::cartesian::Rect<f64>,
    ) {
        let mut boundaries = Vec::new();
        let conv = self
            .0
            .iter()
            .map(|v| Convert::new(v.clone()))
            .collect::<Vec<Convert<Polygon>>>();
        let parts = conv
            .iter()
            .cloned()
            .map(|v| {
                let (poly, bounds) = v.bounded_polygon();
                boundaries.push(bounds);
                poly
            })
            .collect::<Vec<galileo::galileo_types::impls::Polygon<Point2d>>>();

        let mut xmin = f64::MAX;
        let mut ymin = f64::MAX;
        let mut xmax = f64::MIN;
        let mut ymax = f64::MIN;

        for bounds in boundaries {
            if let Some(rect) = bounds {
                let x_min = rect.x_min();
                if x_min < xmin {
                    xmin = x_min;
                }
                let y_min = rect.y_min();
                if y_min < ymin {
                    ymin = y_min;
                }

                let x_max = rect.x_max();
                if x_max > xmax {
                    xmax = x_max;
                }
                let y_max = rect.y_max();
                if y_max > ymax {
                    ymax = y_max;
                }
            }
        }

        let bounds = galileo::galileo_types::cartesian::Rect::new(xmin, ymin, xmax, ymax);

        (
            galileo::galileo_types::impls::MultiPolygon { parts },
            bounds,
        )
    }
}

impl Convert<geo::geometry::MultiPolygon> {
    pub fn geo_to_multipolygon(self) -> galileo::galileo_types::impls::MultiPolygon<Point2d> {
        let parts = self.0.iter().map(|v| Convert::new(v.clone()).polygon()).collect::<Vec<galileo::galileo_types::impls::Polygon<Point2d>>>();
        galileo::galileo_types::impls::MultiPolygon { parts }
    }
}

impl Convert<Polygon> {
    pub fn polygon(self) -> galileo::galileo_types::impls::Polygon<Point2d> {
        let (e, i) = self.0.into_inner();
        let ext = Convert::new(e).contour();
        let mut poly: galileo::galileo_types::impls::Polygon<Point2d> = ext.into();
        let mut int = Vec::new();
        if !i.is_empty() {
            for item in i {
                int.push(Convert::new(item).contour());
            }
        }
        poly.inner_contours = int;
        poly
    }

    pub fn bounded_polygon(
        self,
    ) -> (
        galileo::galileo_types::impls::Polygon<Point2d>,
        Option<galileo::galileo_types::cartesian::Rect<f64>>,
    ) {
        let ext = self.0.exterior();
        let conv = Convert::new(ext.clone()).bounds();
        if let Some(rect) = conv {
            let min = rect.min();
            let xmin = min.x();
            let ymin = min.y();
            let max = rect.max();
            let xmax = max.x();
            let ymax = max.y();
            let bounds = galileo::galileo_types::cartesian::Rect::new(xmin, ymin, xmax, ymax);
            (self.polygon(), Some(bounds))
        } else {
            (self.polygon(), None)
        }
    }
}

impl Convert<shapefile::record::polygon::Polygon> {
    pub fn geo_polygons(self) -> Vec<geo::geometry::Polygon> {
        let mut polys = Vec::new();
        let mut outer = None;
        let mut inner = Vec::new();
        for ring in self.0.into_inner() {
            match ring.clone() {
                shapefile::record::polygon::PolygonRing::Outer(_) => match outer {
                    Some(x) => {
                        let poly = geo::geometry::Polygon::new(x, inner);
                        polys.push(poly);
                        outer = None;
                        inner = Vec::new();
                    }
                    None => {
                        let conv = Convert::new(ring);
                        let line = conv.geo_linestring();
                        outer = Some(line);
                    }
                },
                shapefile::record::polygon::PolygonRing::Inner(_) => {
                    let conv = Convert::new(ring);
                    let line = conv.geo_linestring();
                    inner.push(line);
                }
            }
        }
        if polys.is_empty() {
            if let Some(ring) = outer {
                polys.push(geo::geometry::Polygon::new(ring, inner));
            }
        }

        polys
    }
}

impl Convert<shapefile::record::polygon::GenericPolygon<shapefile::record::point::PointZ>> {
    pub fn geo_polygons(self) -> Vec<geo::geometry::Polygon> {
        tracing::info!("Calling convert to multipolygon.");
        let mut polys = Vec::new();
        let mut outer = None;
        let mut inner = Vec::new();
        for ring in self.0.into_inner() {
            match ring.clone() {
                shapefile::record::polygon::PolygonRing::Outer(_) => match outer {
                    Some(x) => {
                        let poly = geo::geometry::Polygon::new(x, inner);
                        polys.push(poly);
                        outer = None;
                        inner = Vec::new();
                    }
                    None => {
                        let conv = Convert::new(ring);
                        let line = conv.geo_linestring();
                        outer = Some(line);
                    }
                },
                shapefile::record::polygon::PolygonRing::Inner(_) => {
                    let conv = Convert::new(ring);
                    let line = conv.geo_linestring();
                    inner.push(line);
                }
            }
        }
        if polys.is_empty() {
            if let Some(ring) = outer {
                polys.push(geo::geometry::Polygon::new(ring, inner));
            }
        }

        polys
    }
}

impl Convert<shapefile::record::polygon::PolygonRing<shapefile::record::point::Point>> {
    pub fn geo_linestring(self) -> geo::geometry::LineString {
        let mut pts = Vec::new();
        for i in self.0.into_inner() {
            let convert = Convert::new(i);
            let pt = convert.geo_coord();
            pts.push(pt);
        }
        geo::geometry::LineString::new(pts)
    }
}

impl Convert<shapefile::record::polygon::PolygonRing<shapefile::record::point::PointZ>> {
    pub fn geo_linestring(self) -> geo::geometry::LineString {
        let mut pts = Vec::new();
        for i in self.0.into_inner() {
            let convert = Convert::new(i);
            let pt = convert.geo_coord();
            pts.push(pt);
        }
        geo::geometry::LineString::new(pts)
    }
}

impl Convert<LineString> {
    pub fn bounds(&self) -> Option<Rect<f64>> {
        self.0.bounding_rect()
    }

    pub fn contour(self) -> ClosedContour<Point2d> {
        let line = self.0.into_inner();
        let points = line
            .iter()
            .map(|v| {
                let p: Coord = v.clone().into();
                Convert::new(p).point()
            })
            .collect::<Vec<Point2d>>();
        ClosedContour::new(points)
    }

    pub fn contour_point(self) -> ClosedContour<Point2d> {
        let line = self.0.into_inner();
        let points = line
            .iter()
            .map(|v| {
                let p: Point = v.clone().into();
                Convert::new(p).point()
            })
            .collect::<Vec<Point2d>>();
        ClosedContour::new(points)
    }
}

impl Convert<geo_types::Rect> {
    pub fn rect(self) -> galileo::galileo_types::cartesian::Rect {
        let min = self.0.min();
        let max = self.0.max();
        galileo::galileo_types::cartesian::Rect::new(min.x, min.y, max.x, max.y)
    }
}

impl CartesianPoint2d for Convert<Point> {
    type Num = f64;
    fn x(&self) -> Self::Num {
        Point::x(self.0)
    }

    fn y(&self) -> Self::Num {
        Point::y(self.0)
    }
}

impl Convert<Point> {
    pub fn point(self) -> Point2d {
        Point2d::new(self.x(), self.y())
    }

    pub fn geo_point(self) -> geo::geometry::Point {
        geo::point!(x: self.x(), y: self.y())
    }

    pub fn geo_coord(self) -> geo::geometry::Coord {
        geo::coord!(x: self.x(), y: self.y())
    }
}

impl CartesianPoint2d for Convert<shapefile::record::point::Point> {
    type Num = f64;
    fn x(&self) -> Self::Num {
        self.clone().into_inner().x
    }

    fn y(&self) -> Self::Num {
        self.clone().into_inner().y
    }
}

impl Convert<shapefile::record::point::Point> {
    pub fn point(self) -> Point2d {
        Point2d::new(self.x(), self.y())
    }

    pub fn geo_point(self) -> geo::geometry::Point {
        geo::point!(x: self.x(), y: self.y())
    }

    pub fn geo_coord(self) -> geo::geometry::Coord {
        geo::coord!(x: self.x(), y: self.y())
    }
}

impl Convert<shapefile::record::point::PointZ> {
    pub fn point(self) -> Point2d {
        Point2d::new(self.0.x(), self.0.y())
    }

    pub fn geo_point(self) -> geo::geometry::Point {
        geo::point!(x: self.0.x(), y: self.0.y())
    }

    pub fn geo_coord(self) -> geo::geometry::Coord {
        geo::coord!(x: self.0.x(), y: self.0.y())
    }
}

impl CartesianPoint2d for Convert<Coord> {
    type Num = f64;
    fn x(&self) -> Self::Num {
        self.0.x
    }

    fn y(&self) -> Self::Num {
        self.0.y
    }
}

impl Convert<Coord> {
    pub fn point(self) -> Point2d {
        Point2d::new(self.x(), self.y())
    }
}
