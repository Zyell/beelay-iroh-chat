# Tauri + Leptos + TailwindCSS + Iroh + Keyhive/Beelay

> [!CAUTION]
> This is an experiment, proof of concept, integrating rapidly developing technologies. It should not be relied on for Production!  

This is a simple chat application to demonstrate [Iroh](https://www.iroh.computer/) P2P connections and [Beelay/Keyhive](https://github.com/inkandswitch/keyhive) using an [Iroh Beelay/Keyhive custom protocol](https://github.com/symplasma/custom_beelay_iroh_protocol).  It is a technical exploration and proof of concept of the underlying technologies.  It is not a polished application and many todos, notes, and warnings reside within.  The primary exploratory goals were the following:

* Integrate the Iroh Beelay/Keyhive custom protocol into a "real" application to understand the needs and challenges of doing so.
* Explore cross-platform application development in Rust using Tauri
* Explore how the lack of IPC type safety can be overcome when you have a Rust frontend like Leptos coupled with a Tauri backend, without resorting to external DSL tools.

## Development Environment

*Note: RustRover was used as the development IDE and some configuration for it is contained in this repository, in particular directory exclusion rules*

### TailwindCSS

You can follow the instructions to [setup TailwindCSS here](https://tailwindcss.com/docs/installation/tailwind-cli).  The CLI tool is recommended and was used for this application's development.

Dev:
```console
npx @tailwindcss/cli -i tailwind_input.css -o tailwind_output.css --watch
```

Release:
```console
npx @tailwindcss/cli -i tailwind_input.css -o tailwind_output.css --minify
```

*Note: Integration of tailwind with Leptos is also an option*

### Tauri + Leptos Application

This application was generated using the [Tauri command line tools with cargo](https://tauri.app/start/) using the Leptos template and subsequently updated to Leptos v0.82 without issue.

### IPC Type Safety

Tauri does not by default provide type safety across the IPC boundary.  Given that most Tauri apps target a JS/TS frontend, there are options to generate TS interface code for type safety.  There are very few for Rust frontends.  [Taur-bindgen](https://github.com/tauri-apps/tauri-bindgen) exists, but requires a DSL in the middle. This is not desirable when going from Rust to Rust only.  I found inspiration from [tauri-ipc-macros](https://github.com/jvatic/tauri-ipc-macros) and [tauri-sys](https://github.com/JonasKruckenberg/tauri-sys).  While command invocation was well covered using tauri-ipc-macros, it did not enforce complete type safety around events and required enum setups for events that make integration with event streams in taur-sys difficult.  The workspace for this app contains an ipc_macros for applying type safety on the IPC layer while maintaining good IDE ergonomics using feature flags.  It isn't perfect but manages to find a sweet spot ergonomically while enforcing type safety for this project's needs.  Hopefully with an enough iterations to provide inspiration, we can solve this problem with Tauri elegantly for everyone using Rust frontends. :-) 

*Note: Please see documentation notes in the ipc_macros and ipc_layer crates in the workspace for more details*

### Cross-Platform build Linux & Android

To support identification of a mobile build to exclude/include the barcode scanner logic, it is important to indicate through trunk that leptos should run with the `mobile` feature enabled.  Otherwise, only tauri will know it is targeting a mobile environment.

The `tauri.android.conf.json` was created to pass a feature flag like so:
```json
{
  "build": {
    "beforeDevCommand": "trunk serve --features mobile",
    "beforeBuildCommand": "trunk build --features mobile"
  }
}
```

This will automatically build Leptos properly when running
```console
cargo tauri android dev
```

This application has been built and configured for Linux and Android only thus far. 

For android apk signing, configuration is in place, according to the [Tauri docs](https://v2.tauri.app/distribute/sign/android/); however, you will need to create your own `keystore.properties` to allow for a signed apk to be automatically built when building for release.  (This facilitates sideloading over adb without issues.)
