#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerState {
    Stopped,
    Starting,
    Ready,
    Busy,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerTransitionError {
    InvalidTransition {
        from: WorkerState,
        operation: &'static str,
    },
    RestartBudgetExhausted {
        attempts: u8,
        max_attempts: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerLifecycle {
    state: WorkerState,
    restart_attempts: u8,
    max_restart_attempts: u8,
}

impl WorkerLifecycle {
    pub fn new(max_restart_attempts: u8) -> Self {
        Self {
            state: WorkerState::Stopped,
            restart_attempts: 0,
            max_restart_attempts,
        }
    }

    pub fn state(&self) -> WorkerState {
        self.state
    }

    pub fn restart_attempts(&self) -> u8 {
        self.restart_attempts
    }

    pub fn max_restart_attempts(&self) -> u8 {
        self.max_restart_attempts
    }

    pub fn begin_start(&mut self) -> Result<(), WorkerTransitionError> {
        self.require_state(WorkerState::Stopped, "begin_start")?;
        self.state = WorkerState::Starting;
        Ok(())
    }

    pub fn mark_ready(&mut self) -> Result<(), WorkerTransitionError> {
        self.require_state(WorkerState::Starting, "mark_ready")?;
        self.state = WorkerState::Ready;
        Ok(())
    }

    pub fn begin_request(&mut self) -> Result<(), WorkerTransitionError> {
        self.require_state(WorkerState::Ready, "begin_request")?;
        self.state = WorkerState::Busy;
        Ok(())
    }

    pub fn finish_request(&mut self) -> Result<(), WorkerTransitionError> {
        self.require_state(WorkerState::Busy, "finish_request")?;
        self.state = WorkerState::Ready;
        Ok(())
    }

    pub fn fail(&mut self) {
        self.state = WorkerState::Failed;
    }

    pub fn begin_restart(&mut self) -> Result<(), WorkerTransitionError> {
        self.require_state(WorkerState::Failed, "begin_restart")?;
        if self.restart_attempts >= self.max_restart_attempts {
            return Err(WorkerTransitionError::RestartBudgetExhausted {
                attempts: self.restart_attempts,
                max_attempts: self.max_restart_attempts,
            });
        }

        self.restart_attempts += 1;
        self.state = WorkerState::Starting;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.state = WorkerState::Stopped;
        self.restart_attempts = 0;
    }

    fn require_state(
        &self,
        expected: WorkerState,
        operation: &'static str,
    ) -> Result<(), WorkerTransitionError> {
        if self.state == expected {
            return Ok(());
        }

        Err(WorkerTransitionError::InvalidTransition {
            from: self.state,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_follows_normal_start_and_request_path() {
        let mut lifecycle = WorkerLifecycle::new(2);

        assert_eq!(lifecycle.state(), WorkerState::Stopped);
        lifecycle.begin_start().unwrap();
        assert_eq!(lifecycle.state(), WorkerState::Starting);
        lifecycle.mark_ready().unwrap();
        assert_eq!(lifecycle.state(), WorkerState::Ready);
        lifecycle.begin_request().unwrap();
        assert_eq!(lifecycle.state(), WorkerState::Busy);
        lifecycle.finish_request().unwrap();
        assert_eq!(lifecycle.state(), WorkerState::Ready);
    }

    #[test]
    fn failure_can_restart_only_within_budget() {
        let mut lifecycle = WorkerLifecycle::new(2);
        lifecycle.begin_start().unwrap();
        lifecycle.fail();

        lifecycle.begin_restart().unwrap();
        assert_eq!(lifecycle.restart_attempts(), 1);
        lifecycle.fail();
        lifecycle.begin_restart().unwrap();
        assert_eq!(lifecycle.restart_attempts(), 2);
        lifecycle.fail();

        assert_eq!(
            lifecycle.begin_restart(),
            Err(WorkerTransitionError::RestartBudgetExhausted {
                attempts: 2,
                max_attempts: 2,
            })
        );
        assert_eq!(lifecycle.state(), WorkerState::Failed);
    }

    #[test]
    fn becoming_ready_does_not_silently_reset_restart_budget() {
        let mut lifecycle = WorkerLifecycle::new(1);
        lifecycle.begin_start().unwrap();
        lifecycle.fail();
        lifecycle.begin_restart().unwrap();
        lifecycle.mark_ready().unwrap();
        lifecycle.fail();

        assert_eq!(
            lifecycle.begin_restart(),
            Err(WorkerTransitionError::RestartBudgetExhausted {
                attempts: 1,
                max_attempts: 1,
            })
        );
    }

    #[test]
    fn intentional_stop_resets_restart_budget() {
        let mut lifecycle = WorkerLifecycle::new(1);
        lifecycle.begin_start().unwrap();
        lifecycle.fail();
        lifecycle.begin_restart().unwrap();
        lifecycle.stop();

        assert_eq!(lifecycle.state(), WorkerState::Stopped);
        assert_eq!(lifecycle.restart_attempts(), 0);
        lifecycle.begin_start().unwrap();
    }

    #[test]
    fn invalid_transitions_do_not_mutate_state() {
        let mut lifecycle = WorkerLifecycle::new(2);

        assert_eq!(
            lifecycle.begin_request(),
            Err(WorkerTransitionError::InvalidTransition {
                from: WorkerState::Stopped,
                operation: "begin_request",
            })
        );
        assert_eq!(lifecycle.state(), WorkerState::Stopped);
    }

    #[test]
    fn zero_restart_budget_disallows_automatic_restart() {
        let mut lifecycle = WorkerLifecycle::new(0);
        lifecycle.begin_start().unwrap();
        lifecycle.fail();

        assert_eq!(
            lifecycle.begin_restart(),
            Err(WorkerTransitionError::RestartBudgetExhausted {
                attempts: 0,
                max_attempts: 0,
            })
        );
    }
}
