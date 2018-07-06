use game::PlayerId;

pub struct Client {
    player: PlayerId,
    state: State,
}

enum State {
    Playing { next_input_frame: u64 },
}

impl Client {
    pub fn new(player: PlayerId, current_frame: u64) -> Client {
        Client {
            player,
            state: State::Playing {
                next_input_frame: current_frame + 1,
            },
        }
    }

    pub fn player_id(&self) -> &PlayerId {
        &self.player
    }

    pub fn received_input(&mut self, frame: u64) -> Result<(), InvalidOp> {
        match self.state {
            State::Playing {
                ref mut next_input_frame,
            } => {
                if frame != *next_input_frame {
                    trace!(
                        "wrong input frame: expected {}, got {}",
                        *next_input_frame,
                        frame,
                    );
                    return Err(InvalidOp);
                }
                *next_input_frame += 1;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InvalidOp;
