use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

enum Phase {
    Running,
    Suspend,
    Paused,
    Stop,
}

pub(super) enum Step {
    Run,
    Skip,
    Stop,
}

pub(super) struct KeyGate {
    phase: Mutex<Phase>,
    cv: Condvar,
}

impl KeyGate {
    pub(super) const fn new() -> Self {
        Self {
            phase: Mutex::new(Phase::Running),
            cv: Condvar::new(),
        }
    }

    pub(super) fn suspend_and_wait(&self) {
        let mut phase = lock(&self.phase);
        if !matches!(*phase, Phase::Running) {
            return;
        }
        *phase = Phase::Suspend;
        self.cv.notify_all();
        while matches!(*phase, Phase::Suspend) {
            phase = wait(&self.cv, phase);
        }
    }

    pub(super) fn resume(&self) {
        let mut phase = lock(&self.phase);
        if matches!(*phase, Phase::Paused | Phase::Suspend) {
            *phase = Phase::Running;
            self.cv.notify_all();
        }
    }

    #[cfg(test)]
    pub(super) fn is_paused(&self) -> bool {
        matches!(*lock(&self.phase), Phase::Paused)
    }

    #[cfg(test)]
    pub(super) fn suspend_requested(&self) -> bool {
        matches!(*lock(&self.phase), Phase::Suspend | Phase::Paused)
    }

    pub(super) fn checkpoint(&self) -> Step {
        let mut phase = lock(&self.phase);
        if matches!(*phase, Phase::Stop) {
            return Step::Stop;
        }
        if !matches!(*phase, Phase::Suspend) {
            return Step::Run;
        }
        *phase = Phase::Paused;
        self.cv.notify_all();
        while matches!(*phase, Phase::Paused) {
            phase = wait(&self.cv, phase);
        }
        if matches!(*phase, Phase::Stop) {
            Step::Stop
        } else {
            Step::Skip
        }
    }

    pub(super) fn stop(&self) {
        *lock(&self.phase) = Phase::Stop;
        self.cv.notify_all();
    }
}

fn lock(mutex: &Mutex<Phase>) -> MutexGuard<'_, Phase> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn wait<'a>(cv: &Condvar, guard: MutexGuard<'a, Phase>) -> MutexGuard<'a, Phase> {
    cv.wait(guard).unwrap_or_else(PoisonError::into_inner)
}
