#![cfg(test)]

#[cfg(any(feature = "with-kusama-runtime",))]
mod setup;

#[cfg(any(feature = "with-kusama-runtime",))]
mod xyk;

#[cfg(any(feature = "with-kusama-runtime",))]
mod proof_of_stake;

#[cfg(any(feature = "with-kusama-runtime",))]
mod bootstrap;

#[cfg(any(feature = "with-kusama-runtime",))]
mod proxy;

#[cfg(any(feature = "with-kusama-runtime",))]
mod identity;

#[cfg(any(feature = "with-kusama-runtime",))]
mod xcm;
