use std::{collections::VecDeque, time::Duration};
use uuid::Uuid;

struct Timer {
    reservation_id: Uuid,
    rounds: u32,
}

pub struct TimingWheel {
    size: usize,
    current_slot: usize,
    slots: Vec<VecDeque<Timer>>,
}

impl TimingWheel {
    pub fn new(size: usize) -> Self {
        Self { current_slot: 0, size, slots: (0..size).map(|_| VecDeque::new()).collect() }
    }

    pub fn add(&mut self, reservation_id: Uuid, delay: Duration) {
        let seconds = delay.as_secs() as usize;

        let slot = (self.current_slot + seconds) % self.size;
        let rounds = seconds / self.size;

        self.slots[slot].push_back(Timer { reservation_id, rounds: rounds as u32 });
    }

    pub fn tick(&mut self) -> Vec<Uuid> {
        let mut expired = Vec::new();

        let bucket = &mut self.slots[self.current_slot];

        let len = bucket.len();

        for _ in 0..len {
            let mut timer = bucket.pop_front().unwrap();

            if timer.rounds == 0 {
                expired.push(timer.reservation_id);
            } else {
                timer.rounds -= 1;
                bucket.push_back(timer);
            }
        }

        self.current_slot = (self.current_slot + 1) % self.size;

        expired
    }
}
