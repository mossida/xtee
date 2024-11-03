use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use ractor_actors::streams::{IterationResult, Operation};

pub enum Direction {
    Forward,
    Backward,
}

pub struct Move {
    pub duration: f32,
    pub direction: Direction,
}

#[async_trait]
impl Operation for Move {
    type State = ();

    async fn work(&self, _state: &mut Self::State) -> Result<IterationResult, ActorProcessingErr> {
        // TODO: Implement movement
        Ok(IterationResult::Continue)
    }
}

pub struct Operator;

#[async_trait]
impl Actor for Operator {
    type Arguments = ();
    type Msg = Move;
    type State = ();

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(())
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let result = message.work(state).await?;
        matches!(result, IterationResult::End).then(|| myself.stop(None));

        Ok(())
    }
}
