use crate::types::{EventId, SeatId};
use std::marker::PhantomData;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SeatsError {
    #[error("Not found enought seats")]
    NotEnoughSeats,

    #[error("Event does not exist: {0}")]
    EventNotFound(EventId),
}

pub struct Available;
pub struct Blocked;
pub struct Reserved;

pub struct Seat<State> {
    pub id: SeatId,
    pub event_id: EventId,
    state: PhantomData<State>,
}

impl<State> Seat<State> {
    pub fn new(id: SeatId, event_id: EventId) -> Self {
        Self { id, event_id, state: PhantomData }
    }
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
