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
	pub fn new(state: PinState) -> Self {
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
pub struct OpenGainPin {
	state: Arc<AtomicPinState>,
}

impl OpenGainPin {
	pub fn new(state: Arc<AtomicPinState>) -> Self {
		OpenGainPin { state }
	}
}

impl hal::OutputPin for OpenGainPin {
	type Error = Infallible;

	fn try_set_high(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Low, Ordering::SeqCst))
	}

	fn try_set_low(&mut self) -> Result<(), Self::Error> {
		Ok(self.state.store(PinState::Floating, Ordering::SeqCst))
	}
}

impl hal::InputPin for OpenGainPin {
	type Error = Infallible;

	fn try_is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::High)
	}

	fn try_is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.state.load(Ordering::SeqCst) == PinState::Low)
	}
}
