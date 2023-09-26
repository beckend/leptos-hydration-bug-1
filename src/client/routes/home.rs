use leptos::*;
use leptos_meta::Title;

use crate::i18n::{t, use_i18n};

#[component]
pub fn RouteHome() -> impl IntoView {
  let (value, set_value) = create_signal(0);
  let i18n = use_i18n();

  view! {
    <Title text=t!(i18n, route_home_title)/>

    <main class="grid w-full h-full">
      // <div class="tabs">
      // <a class="tab tab-lg tab-lifted">"Large"</a>
      // <a class="tab tab-lg tab-lifted tab-active">"Large"</a>
      // <a class="tab tab-lg tab-lifted">"Large"</a>
      // </div>

      // <button
      // class="inline-block cursor-pointer rounded-md bg-gray-800 px-4 py-3 text-center text-sm font-semibold uppercase text-white transition duration-200 ease-in-out hover:bg-gray-900">
      // "Button"
      // </button>

      <div class="bg-gradient-to-tl from-blue-800 to-blue-500 text-white font-mono flex flex-col min-h-screen">
        <div class="flex flex-row-reverse flex-wrap m-auto">
          <button
            on:click=move |_| set_value.update(|value| *value += 1)
            class="rounded px-3 py-2 m-1 border-b-4 border-l-2 shadow-lg bg-blue-700 border-blue-800 text-white"
          >
            "+"
          </button>

          <button class="rounded px-3 py-2 m-1 border-b-4 border-l-2 shadow-lg bg-blue-800 border-blue-900 text-white">
            {value}
          </button>

          <button
            on:click=move |_| set_value.update(|value| *value -= 1)
            class="rounded px-3 py-2 m-1 border-b-4 border-l-2 shadow-lg bg-blue-700 border-blue-800 text-white"
          >
            "-"
          </button>
        </div>
      </div>
    </main>
  }
}
