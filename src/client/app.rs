use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use super::{error_template::ErrorTemplate, errors::AppError, routes::home::RouteHome};
use crate::i18n;

fn router_fallback() -> View {
  let mut outside_errors = Errors::default();
  outside_errors.insert_with_default_key(AppError::NotFound);

  view! { <ErrorTemplate outside_errors/> }
}

#[component]
fn AppBase() -> impl IntoView {
  view! {
    <ErrorBoundary fallback=move |errs| {
        view! { <ErrorTemplate errors=errs/> }
    }>
      <Title text="Admin views"/>

      <Meta name="description" content="Admin views"/>

      <Meta http_equiv="X-UA-Compatible" content="IE=edge"/>

      <Meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no"/>

      <Stylesheet id="leptos" href="/pkg/this-app.css"/>

      <Link href="/favicon/apple-touch-icon.png" rel="apple-touch-icon" sizes="180x180"/>

      <Link href="/favicon/favicon-16x16.png" rel="icon" sizes="16x16" type_="image/png"/>

      <Link href="/favicon/favicon-32x32.png" rel="icon" sizes="32x32" type_="image/png"/>

      <Link href="/favicon/site.webmanifest" rel="manifest"/>

      <Router fallback=router_fallback>
        <Routes>
          <Route path="" ssr=SsrMode::OutOfOrder view=|| view! { <RouteHome/> }/>
        </Routes>
      </Router>
    </ErrorBoundary>
  }
}

#[component]
pub fn App() -> impl IntoView {
  provide_meta_context();
  i18n::provide_i18n_context();

  view! { <AppBase/> }
}
