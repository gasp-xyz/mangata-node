#![cfg(test)]

#[cfg(any(feature = "with-kusama-runtime",))]
mod setup;

#[cfg(any(feature = "with-kusama-runtime",))]
mod xyk;

#[cfg(any(feature = "with-kusama-runtime",))]
mod bootstrap;

#[cfg(any(feature = "with-kusama-runtime",))]
mod proxy;
