# Button Uploader

This is a [Yew](https://yew.rs/) app that allows a user to upload a
button to the user's browser, without it going to a server.

It's just a proof-of-concept that exercises the Browser's IndexDB
facilities either via the
[rexie](https://github.com/devashishdxt/rexie) crate (at least on this
branch `indexed-db` uses the
[indexed-db](https://docs.rs/indexed-db/latest/indexed_db/) crate).

The easiest way to use this app is to install
[trunk](https://trunkrs.dev/) and then run it from trunk:

```
cargo install trunk
trunk serve --open
```

You'll then see a circular button with a red B in it. If you click on
the button, it'll change into a narcoleptic dinosaur.  Both of those
buttons are built-in.  If you shift-click the button, you'll get a
file picker which will allow you to choose any image and have that be
your button.

## Ugh!

This code wasn't meant to be made public, per-se, but I have so many
other public horrible Rust repositories that I'm kidding nobody about
the quality of my hack-and-slash code and I want to give people an
opportunity to see this code in advance of the March 26th, 2025
Albuquerque Rust Get-together.  FWIW, we'll be looking at the `indexed-db`
branch.
