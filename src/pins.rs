use core::convert::Infallible;
use embedded_hal::digital as hal;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum PinState {
	High = 1,
	Low,
	Floating,
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

#[derive(Debug)]
pub struct AtomicPinState {
	state: AtomicUsize,
}

impl AtomicPinState {
	pub fn new() -> Self {
		Self::new_with_state(PinState::Floating)
	}

	pub fn new_with_state(state: PinState) -> Self {
		AtomicPinState {
			state: AtomicUsize::new(state.to_usize().unwrap()),
		}
	}

	pub fn load(&self, order: Ordering) -> PinState {
		PinState::from_usize(self.state.load(order)).unwrap()
	}

	pub fn store(&self, state: PinState, order: Ordering) {
		self.state.store(state.to_usize().unwrap(), order);
	}
}

#[derive(Clone, Debug)]
pub struct InputPin {
	state: Arc<AtomicPinState>,
}

impl InputPin {
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

impl hal::InputPin for PushPullPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}

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

impl hal::InputPin for OpenDrainPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
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
		use PinState::*;
		let state = Arc::new(AtomicPinState::new());
		let mut pin = PushPullPin::new(state.clone());
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		assert_eq!(Ok(()), pin.try_set_high());
		assert_eq!(High, state.load(Ordering::SeqCst));
		assert_eq!(Ok(true), pin.try_is_high());
		assert_eq!(Ok(false), pin.try_is_low());
		assert_eq!(Ok(()), pin.try_set_low());
		assert_eq!(Low, state.load(Ordering::SeqCst));
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(true), pin.try_is_low());
	}

	#[test]
	fn hal_open_drain_pin() {
		use hal::InputPin as HalInputPin;
		use hal::OutputPin as HalOutputPin;
		use PinState::*;
		let state = Arc::new(AtomicPinState::new());
		let mut pin = OpenDrainPin::new(state.clone());
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		assert_eq!(Ok(()), pin.try_set_high());
		assert_eq!(Low, state.load(Ordering::SeqCst));
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(true), pin.try_is_low());
		assert_eq!(Ok(()), pin.try_set_low());
		assert_eq!(Floating, state.load(Ordering::SeqCst));
		assert_eq!(Ok(false), pin.try_is_high());
		assert_eq!(Ok(false), pin.try_is_low());
	}
}
