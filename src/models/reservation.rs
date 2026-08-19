use crate::{
    repositories::User,
    types::{EventId, ReservationId, SeatId, UserId},
};
use std::marker::PhantomData;

pub struct Pending;
pub struct Paid;
pub struct Expired;
pub struct Canceled;

pub struct Reservation<State> {
    pub id: ReservationId,
    pub event_id: EventId,
    pub user_id: UserId,
    pub seat_ids: Vec<SeatId>,
    state: PhantomData<State>,
}

impl<State> Reservation<State> {
    pub fn new(
        id: ReservationId,
        event_id: EventId,
        user_id: UserId,
        seat_ids: Vec<SeatId>,
    ) -> Self {
        Self { id, event_id, user_id, seat_ids, state: PhantomData }
    }
}

impl Reservation<Pending> {
    pub fn pay(self) -> Reservation<Paid> {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_id: self.user_id,
            seat_ids: self.seat_ids,
            state: PhantomData,
        }
    }

    pub fn cancel(self) -> Reservation<Canceled> {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_id: self.user_id,
            seat_ids: self.seat_ids,
            state: PhantomData,
        }
    }
}

impl Reservation<Pending> {
    pub fn expire(self) -> Reservation<Expired> {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_id: self.user_id,
            seat_ids: self.seat_ids,
            state: PhantomData,
        }
    }
}
