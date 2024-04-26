pub mod address;
pub mod controls;
pub mod convert;
pub mod data;
pub mod ops;
pub mod parcels;
pub mod run;
pub mod run_ui;
pub mod state;
pub mod table;
pub mod utils;

pub mod prelude {
    pub use crate::address::{
        AddressPoint, AddressPoints, AddressSymbol, MatchPoint, MatchPoints, MatchSymbol,
    };
    pub use crate::controls::{Action, Binding, KEY_BINDINGS, MOUSE_BINDINGS};
    pub use crate::convert::Convert;
    pub use crate::data::{AddressSource, Data};
    pub use crate::ops::{Compare, Operations};
    pub use crate::parcels::{Parcel, Parcels};
    pub use crate::run::run;
    pub use crate::run_ui::{Card, SearchConfig, UiState};
    pub use crate::state::{App, EguiState, GalileoState, WgpuFrame};
    pub use crate::table::{Columnar, Filtration, TableConfig, TableView, Tabular};
    pub use crate::utils::{from_csv, point_bounds, toggle_select};
}
