//! Atomic pin types
//!
//! This module provides implementations of atomic pin types that
//! can be used by any [`embedded_hal`] implementation that use
//! [`Input`-](`embedded_hal::digital::InputPin`) or
//! [`OutputPin`s](`embedded_hal::digital::OutputPin`).
//!
//! As atomic types these pins use primitive [`atomic`](`std::sync::atomic`) types,
//! so that these pins can be shared safely between threads. Especially useful
//! for integration testing.

use core::convert::Infallible;
use embedded_hal::digital as hal;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A digital pin state.
#[derive(Clone, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum PinState {
	/// Logical high
	High = 1,
	/// Logical low
	Low,
	/// Floating potential (not connected / High-Z)
	Floating,
}

/// A digital [pin state](`PinState`) which can be safely shared between threads.
///
/// This type is based on [`AtomicUsize`], so the same limitations and platform
/// support apply.
#[derive(Debug)]
pub struct AtomicPinState {
	state: AtomicUsize,
}

impl AtomicPinState {
	/// Creates a new atomic pin state with a floating state.
	pub fn new() -> Self {
		Self::new_with_state(PinState::Floating)
	}

	/// Creates a new atomic pin state with a given state.
	///
	/// # Examples
	///
	/// ```
	/// use embedded_hal_vcd::pins::{PinState, AtomicPinState};
	///
	/// let high = AtomicPinState::new_with_state(PinState::High);
	/// let low = AtomicPinState::new_with_state(PinState::Low);
	/// ```
	pub fn new_with_state(state: PinState) -> Self {
		AtomicPinState {
			state: AtomicUsize::new(state.to_usize().unwrap()),
		}
	}

	/// Loads a state from the atomic pin state.
	///
	/// `load` taks an [`Ordering`] argument which describes the memory
	/// ordering of this operation. For more information see [`AtomicUsize::load`].
	pub fn load(&self, order: Ordering) -> PinState {
		PinState::from_usize(self.state.load(order)).unwrap()
	}

	/// Stores a state into the atomic pin state.
	///
	/// `store` taks an [`Ordering`] argument which describes the memory
	/// ordering of this operation. For more information see [`AtomicUsize::store`].
	pub fn store(&self, state: PinState, order: Ordering) {
		self.state.store(state.to_usize().unwrap(), order);
	}
}

/// A mutable [input pin](`hal::InputPin`) that can be safely shared between threads.
///
/// This pin implements [`embedded_hal::InputPin`](`hal::InputPin`) and can be used
/// to share an [`AtomicPinState`] with an [`embedded_hal`] implementation.
///
/// # Examples
///
/// ```
/// use embedded_hal_vcd::pins::{AtomicPinState, InputPin, PinState};
/// use embedded_hal::digital::InputPin as HalInputPin;
/// use std::sync::{Arc, atomic::Ordering};
///
/// let state = Arc::new(AtomicPinState::new_with_state(PinState::Low));
/// let pin = InputPin::new(state.clone());
/// assert_eq!(Ok(true), pin.try_is_low());
/// state.store(PinState::High, Ordering::SeqCst);
/// assert_eq!(Ok(true), pin.try_is_high());
/// ```
#[derive(Clone, Debug)]
pub struct InputPin {
	state: Arc<AtomicPinState>,
}

impl InputPin {
	/// Creates a new input pin with a given [`PinState`].
	pub fn new(state: Arc<AtomicPinState>) -> Self {
		InputPin { state }
	}
}

impl hal::InputPin for InputPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

/// A mutable [output pin](`hal::OutputPin`) that can be safely shared between threads.
///
/// This pin implements [`embedded_hal::OutputPin`](`hal::OutputPin`) and can be used
/// to share an [`AtomicPinState`] with an [`embedded_hal`] implementation.
///
/// It also implements [`embedded_hal::InputPin`](`hal::InputPin`), so it is possible
/// to also read the internal state.
///
/// # Examples
///
/// ```
/// use embedded_hal_vcd::pins::{AtomicPinState, PushPullPin, PinState};
/// use embedded_hal::digital::{InputPin as HalInputPin, OutputPin};
/// use std::sync::Arc;
///
/// let state = Arc::new(AtomicPinState::new());
/// let mut pin = PushPullPin::new(state.clone());
/// pin.try_set_low().unwrap();
/// assert_eq!(Ok(true), pin.try_is_low());
/// pin.try_set_high().unwrap();
/// assert_eq!(Ok(true), pin.try_is_high());
/// ```
#[derive(Clone, Debug)]
pub struct PushPullPin {
	state: Arc<AtomicPinState>,
}

impl PushPullPin {
	pub fn new(state: Arc<AtomicPinState>) -> Self {
		PushPullPin { state }
	}
}

impl hal::OutputPin for PushPullPin {
	type Error = Infallible;

	fn try_set_high(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::High, Ordering::SeqCst))
	}

	fn try_set_low(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Low, Ordering::SeqCst))
	}
}

impl hal::StatefulOutputPin for PushPullPin {
	fn try_is_set_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_set_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

impl hal::toggleable::Default for PushPullPin {}

impl hal::InputPin for PushPullPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

/// A mutable [output pin](`hal::OutputPin`) in open drain configuration that can be safely shared between threads.
///
/// This pin implements [`embedded_hal::OutputPin`](`hal::OutputPin`) and can be used
/// to share an [`AtomicPinState`] with an [`embedded_hal`] implementation. In open drain
/// configuration this pin is in a floating state (not connected) if it is set to low and
/// logical low ("pull to GND") if it is set to high.
///
/// It also implements [`embedded_hal::InputPin`](`hal::InputPin`), so it is possible
/// to also read the internal state, which will be either [`Floating`](`PinState::Floating`)
/// or [`Low`](`PinState::Low`).
///
/// # Examples
///
/// ```
/// use embedded_hal_vcd::pins::{AtomicPinState, OpenDrainPin, PinState};
/// use embedded_hal::digital::{InputPin as HalInputPin, OutputPin};
/// use std::sync::{Arc, atomic::Ordering};
///
/// let state = Arc::new(AtomicPinState::new());
/// let mut pin = OpenDrainPin::new(state.clone());
/// pin.try_set_low().unwrap();
/// assert_eq!(Ok(false), pin.try_is_low());
/// assert_eq!(Ok(false), pin.try_is_high());
/// assert_eq!(PinState::Floating, state.load(Ordering::SeqCst));
/// pin.try_set_high().unwrap();
/// assert_eq!(Ok(false), pin.try_is_high());
/// assert_eq!(Ok(true), pin.try_is_low());
/// ```

#[derive(Clone, Debug)]
pub struct OpenDrainPin {
	state: Arc<AtomicPinState>,
}

impl OpenDrainPin {
	pub fn new(state: Arc<AtomicPinState>) -> Self {
		OpenDrainPin { state }
	}
}

impl hal::OutputPin for OpenDrainPin {
	type Error = Infallible;

	fn try_set_high(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Low, Ordering::SeqCst))
	}

	fn try_set_low(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Floating, Ordering::SeqCst))
	}
}

impl hal::StatefulOutputPin for OpenDrainPin {
	fn try_is_set_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}

	fn try_is_set_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Floating)
	}
}

impl hal::toggleable::Default for OpenDrainPin {}

impl hal::InputPin for OpenDrainPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

impl From<vcd::Value> for PinState {
	fn from(val: vcd::Value) -> PinState {
		use vcd::Value::*;
		match val {
			V0 => PinState::Low,
			V1 => PinState::High,
			Z => PinState::Floating,
			X => PinState::Floating,
		}
	}
}

impl From<PinState> for vcd::Value {
	fn from(state: PinState) -> vcd::Value {
		use vcd::Value;
		use PinState::*;
		match state {
			High => Value::V1,
			Low => Value::V0,
			Floating => Value::Z,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn pin_state() {
		use vcd::Value::*;
		use PinState::*;

		assert_eq!(V0, Low.into());
		assert_eq!(V1, High.into());
		assert_eq!(Z, Floating.into());

		assert_eq!(Low, V0.into());
		assert_eq!(High, V1.into());
		assert_eq!(Floating, Z.into());
		assert_eq!(Floating, X.into());
	}

	#[test]
	fn atomic_pin_state() {
		use PinState::*;
		let state = AtomicPinState::new();
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		// loading second time should still contain value
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		state.store(High, Ordering::SeqCst);
		assert_eq!(High, state.load(Ordering::SeqCst));
		let state = AtomicPinState::new_with_state(Low);
		assert_eq!(Low, state.load(Ordering::SeqCst));
	}

	#[test]
	fn hal_input_pin() {
		use hal::InputPin as HalInputPin;
		use PinState::*;
		let state = Arc::new(AtomicPinState::new());
		let pin = InputPin::new(state.clone());
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(false), pin.try_is_low());
		state.store(High, Ordering::SeqCst);
		assert_eq!(Ok(true), pin.try_is_high());
		assert_eq!(Ok(false), pin.try_is_low());
		state.store(Low, Ordering::SeqCst);
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(true), pin.try_is_low());
	}

	#[test]
	fn hal_push_pull_pin() {
		use hal::InputPin as HalInputPin;
		use hal::OutputPin as HalOutputPin;
		use hal::StatefulOutputPin as HalStatefulOutputPin;
		use PinState::*;
		let state = Arc::new(AtomicPinState::new());
		let mut pin = PushPullPin::new(state.clone());
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		assert_eq!(Ok(()), pin.try_set_high());
		assert_eq!(High, state.load(Ordering::SeqCst));
		assert_eq!(Ok(true), pin.try_is_high());
		assert_eq!(Ok(false), pin.try_is_low());
		assert_eq!(Ok(true), pin.try_is_set_high());
		assert_eq!(Ok(false), pin.try_is_set_low());
		assert_eq!(Ok(()), pin.try_set_low());
		assert_eq!(Low, state.load(Ordering::SeqCst));
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(true), pin.try_is_low());
		assert_eq!(Ok(false), pin.try_is_set_high());
		assert_eq!(Ok(true), pin.try_is_set_low());
	}

	#[test]
	fn hal_open_drain_pin() {
		use hal::InputPin as HalInputPin;
		use hal::OutputPin as HalOutputPin;
		use hal::StatefulOutputPin as HalStatefulOutputPin;
		use PinState::*;
		let state = Arc::new(AtomicPinState::new());
		let mut pin = OpenDrainPin::new(state.clone());
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		assert_eq!(Ok(()), pin.try_set_high());
		assert_eq!(Low, state.load(Ordering::SeqCst));
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(true), pin.try_is_low());
		assert_eq!(Ok(true), pin.try_is_set_high());
		assert_eq!(Ok(false), pin.try_is_set_low());
		assert_eq!(Ok(()), pin.try_set_low());
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(false), pin.try_is_low());
		assert_eq!(Ok(false), pin.try_is_set_high());
		assert_eq!(Ok(true), pin.try_is_set_low());
	}
}
