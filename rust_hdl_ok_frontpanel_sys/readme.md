# Quickstart

Set the `OK_FRONTPANEL_DIR` environment variable to point to the `FrontPanel/API` directory.  This
should be the location of the `okFrontPanel.h` header file.

## Mac OS X

Set the `DYLD_FALLBACK_LIBRARY_PATH` environment variable to point to the same directory.  You can do

```shell
export DYLD_FALLBACK_LIBRARY_PATH=$OK_FRONTPANEL_DIR
```

Then the usual `cargo build` and `cargo test` should work.
