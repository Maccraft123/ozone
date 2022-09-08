# Have you ever wanted to run your Rust program as a pseudo-UEFI program using Linux? No? Well I got the solution just for you!

# Presenting: Ozone!
Think of ozone like an init system but statically built into your executable.
Best served with [cargo-mkinitrd](https://github.com/Maccraft123/cargo-mkinitrd)

## How to use this?

1. Add a `ozone` feature to your program
2. Make a new struct, `ozone::basic::Config`, `new` associated function has sane defaults, TODO: document it here
3. Run `ozone::basic::init()` with borrow of that `Config` struct, if feature `ozone` is enabled
