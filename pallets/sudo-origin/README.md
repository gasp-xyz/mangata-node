# SudoOrigin Module

## Overview

The SudoOrigin module allows for a origin
to execute dispatchable functions that require a `Root` call.

## Interface

### Dispatchable Functions

Only the configured origin can call the dispatchable functions from the Sudo module.

* `sudo` - Make a `Root` call to a dispatchable function.

## Usage

### Executing Privileged Functions

The SudoOrigin module itself is not intended to be used within other modules.
Instead, you can build "privileged functions" (i.e. functions that require `Root` origin) in other modules.
You can execute these privileged functions by calling `sudo` from the configured origin.
Privileged functions cannot be directly executed via an extrinsic.

Learn more about privileged functions and `Root` origin in the [`Origin`] type documentation.

### Simple Code Snippet

This is an example of a module that exposes a privileged function:

```rust
use frame_support::{decl_module, dispatch};
use frame_system::ensure_root;

pub trait Trait: frame_system::Trait {}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		#[weight = 0]
        pub fn privileged_function(origin) -> dispatch::DispatchResult {
            ensure_root(origin)?;

            // do something...

            Ok(())
        }
    }
}
```

## Runtime Config

The SudoOrigin module depends on the Runtime for its accepted Origin configuration.

## Related Modules

* [Democracy](https://docs.rs/pallet-democracy/latest/pallet_democracy/)

[`Call`]: ./enum.Call.html
[`Trait`]: ./trait.Trait.html
[`Origin`]: https://docs.substrate.dev/docs/substrate-types
