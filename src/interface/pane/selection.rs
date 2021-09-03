use super::marker::Marker;

#[derive(Copy, Clone, Debug)]
pub struct Selection {
	pub anchor: Marker,
	pub end: Marker
}
