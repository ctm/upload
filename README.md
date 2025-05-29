# Image Uploader

This is a [Yew](https://yew.rs/) app that allows a user to upload an
image (like a poker button) to the user's browser, without it going to
a server.

It's just a proof-of-concept that exercises the Browser's IndexDB
facilities via the
[indexed-db](https://docs.rs/indexed-db/latest/indexed_db/) crate.

The easiest way to use this app is to install
[trunk](https://trunkrs.dev/) and then run it from trunk:

```
cargo install trunk
trunk serve --open
```

You'll then see a circular image with a red B in it. If you click on
the image, it'll change into a narcoleptic dinosaur.  Both of those
images are built-in.  If you shift-click the button, you'll get a
file picker which will allow you to choose any image and have that be
the image that is displayed until it's clicked again.
