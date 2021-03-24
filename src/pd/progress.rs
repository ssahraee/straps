// STRAPS - Statistical Testing of RAndom Probing Security
// Copyright (C) 2021 UCLouvain
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
pub struct SubFinisher {
    started_pb: ProgressBar,
    finished_pb: ProgressBar,
}
impl Finisher for SubFinisher {
    fn finish(&self, pb: &ProgressBar) {
        pb.finish_and_clear();
        self.finished_pb.inc(1);
        if self.finished_pb.position() >= self.finished_pb.length() {
            self.started_pb.finish_and_clear();
            self.finished_pb.finish();
        }
    }
}

pub struct SubProgressSystem {
    mp: Arc<MultiProgress>,
    started_pb: ProgressBar,
    finished_pb: ProgressBar,
    style: ProgressStyle,
    n_issued: u64,
}
impl SubProgressSystem {
    pub fn new(n_to_issue: u64, style: ProgressStyle) -> (Self, thread::JoinHandle<()>) {
        let mp = Arc::new(MultiProgress::new());
        let mpd = Arc::downgrade(&mp);
        let join_handle = thread::spawn(move || {
            while let Some(x) = mpd.upgrade() {
                let _ = x.join();
            }
        });
        let started_pb = mp.add(ProgressBar::new(n_to_issue));
        started_pb.set_style(style.clone());
        started_pb.set_message("started ");
        let finished_pb = mp.add(ProgressBar::new(n_to_issue));
        finished_pb.set_style(style.clone());
        finished_pb.set_message("finished");
        (
            Self {
                mp,
                started_pb,
                finished_pb,
                style,
                n_issued: 0,
            },
            join_handle,
        )
    }
    pub fn issue(&mut self, n: u64) -> FinishingProgress<SubFinisher> {
        let pb = self.mp.add(ProgressBar::new(n));
        pb.set_style(self.style.clone());
        pb.set_message(&format!("{:>3}", self.n_issued));
        self.started_pb.inc(1);
        self.n_issued += 1;
        if self.finished_pb.length() < self.n_issued {
            self.finished_pb.set_length(self.n_issued);
        }
        self.finished_pb.tick();
        FinishingProgress::new(
            pb,
            SubFinisher {
                started_pb: self.started_pb.clone(),
                finished_pb: self.finished_pb.clone(),
            },
        )
    }
}

pub trait Finisher {
    fn finish(&self, pb: &ProgressBar);
}
impl<F: Fn(&ProgressBar)> Finisher for F {
    fn finish(&self, pb: &ProgressBar) {
        self(pb);
    }
}

#[derive(Debug, Clone)]
pub struct FinishingProgress<F: Finisher = SubFinisher> {
    pb: ProgressBar,
    end_is_finish: bool,
    f: F,
    finished: bool,
}
impl<F: Finisher> FinishingProgress<F> {
    fn new(pb: ProgressBar, f: F) -> Self {
        Self {
            pb,
            f: f,
            end_is_finish: false,
            finished: false,
        }
    }
    pub fn finishing(&mut self, end_is_finish: bool) {
        self.end_is_finish = end_is_finish;
    }
    pub fn inc(&mut self, delta: u64) {
        if delta != 0 {
            assert!(!self.finished);
            self.pb.inc(delta);
            if self.end_is_finish && self.pb.position() >= self.pb.length() {
                self.finished = true;
                self.f.finish(&self.pb);
            }
        }
    }
    pub fn inc_length(&mut self, delta: i64) {
        self.pb.set_length((self.pb.length() as i64 + delta) as u64);
    }
    pub fn set_length(&mut self, length: u64) {
        self.pb.set_length(length);
    }
    pub fn position(&self) -> u64 {
        self.pb.position()
    }
}
