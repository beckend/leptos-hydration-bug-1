# Leptos with Axum + TailwindCSS Tempate

This is a template demonstrating how to integrate [TailwindCSS](https://tailwindcss.com/) with the [Leptos](https://github.com/leptos-rs/leptos) web framework, Axum server, and the [cargo-leptos](https://github.com/akesson/cargo-leptos) tool.

## Getting Started

See the [Examples README](../README.md) for setup and run instructions.

# Reproduce hydration bug:
edit `./src/client/routes/home.rs`
On lines `14-24` there are elements ready to be used, uncomment any element you want, and see the SSR bug in browser console. 
