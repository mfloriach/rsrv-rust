use crate::types::{EventId, SeatId};
use std::marker::PhantomData;
use uuid::Uuid;

pub struct Available;
pub struct Blocked;
pub struct Reserved;

pub struct Seat<State> {
    pub id: SeatId,
    pub event_id: EventId,
    state: PhantomData<State>,
}

impl Seat<Available> {
    pub fn block(self) -> Seat<Blocked> {
        Seat { id: self.id, event_id: self.event_id, state: PhantomData }
    }
}

impl Seat<Blocked> {
    pub fn reserve(self) -> Seat<Reserved> {
        Seat { id: self.id, event_id: self.event_id, state: PhantomData }
    }
}

impl Seat<Blocked> {
    pub fn release(self) -> Seat<Available> {
        Seat { id: self.id, event_id: self.event_id, state: PhantomData }
    }
}
