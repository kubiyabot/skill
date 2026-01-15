//! Root application component
//!
//! Sets up routing, global state, and the main layout structure.

use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::layout::Layout;
use crate::router::{switch, Route};

/// Root application component
#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}
