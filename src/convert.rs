// use crate::prelude::*;
use galileo_types::cartesian::{CartesianPoint2d, Point2d};
use galileo_types::impls::ClosedContour;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::geometry::Rect;
use geo_types::{Coord, LineString, MultiPolygon, Point, Polygon};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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
    pub fn multipolygon(self) -> galileo_types::impls::MultiPolygon<Point2d> {
        let conv = self
            .0
            .iter()
            .map(|v| Convert::new(v.clone()))
            .collect::<Vec<Convert<Polygon>>>();
        let parts = conv
            .par_iter()
            .cloned()
            .map(|v| v.polygon())
            .collect::<Vec<galileo_types::impls::Polygon<Point2d>>>();
        galileo_types::impls::MultiPolygon { parts }
    }

    pub fn bounded_multipolygon(
        self,
    ) -> (
        galileo_types::impls::MultiPolygon<Point2d>,
        galileo_types::cartesian::Rect<f64>,
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
            .collect::<Vec<galileo_types::impls::Polygon<Point2d>>>();

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

        let bounds = galileo_types::cartesian::Rect::new(xmin, ymin, xmax, ymax);

        (galileo_types::impls::MultiPolygon { parts }, bounds)
    }
}

impl Convert<Polygon> {
    pub fn polygon(self) -> galileo_types::impls::Polygon<Point2d> {
        let (e, i) = self.0.into_inner();
        let ext = Convert::new(e).contour();
        let mut poly: galileo_types::impls::Polygon<Point2d> = ext.into();
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
        galileo_types::impls::Polygon<Point2d>,
        Option<galileo_types::cartesian::Rect<f64>>,
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
            let bounds = galileo_types::cartesian::Rect::new(xmin, ymin, xmax, ymax);
            (self.polygon(), Some(bounds))
        } else {
            (self.polygon(), None)
        }
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
