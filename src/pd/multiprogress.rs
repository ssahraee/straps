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

use hytra::TrAdder;
use indicatif::{ProgressBar, ProgressStyle};
use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

#[derive(Debug, Clone)]
pub struct MultiProgressConfig {
    n_bars: usize,
    style: ProgressStyle,
}
#[derive(Debug)]
pub struct MultiProgress {
    sub_progress: Vec<SubProgress>,
}
#[derive(Debug)]
pub struct SubProgress {
    end_is_finish: AtomicBool,
    count: TrAdder<i64>,
    length: TrAdder<i64>,
}

impl MultiProgressConfig {
    pub fn new(n_bars: usize, style: ProgressStyle) -> Self {
        Self { n_bars, style }
    }
    fn run_progress_reader(
        &self,
        mp: &MultiProgress,
        f_finished: &AtomicBool,
        imp: &indicatif::MultiProgress,
        started_pb: ProgressBar,
        finished_pb: ProgressBar,
    ) {
        let mut sub_bars: Vec<Option<ProgressBar>> = vec![None; self.n_bars];
        while !f_finished.load(Ordering::Acquire) {
            thread::sleep(std::time::Duration::from_millis(100));
            for (i, (s_b, s_p)) in sub_bars.iter_mut().zip(mp.sub_progress.iter()).enumerate() {
                if let Some(bar) = s_b {
                    let end_is_finish = s_p.end_is_finish.load(Ordering::Acquire);
                    let length: u64 = s_p.length.get().try_into().expect("negative length");
                    let count: u64 = s_p.count.get().try_into().expect("negative count");
                    if count > length {
                        println!("wrong count len: {}/{}", count, length);
                    }
                    if bar.length() != length {
                        bar.set_length(length);
                    }
                    if bar.position() != count {
                        bar.set_position(count);
                        if count == length && end_is_finish {
                            bar.finish_and_clear();
                            finished_pb.inc(1);
                            if finished_pb.position() == self.n_bars as u64 {
                                started_pb.finish_and_clear();
                                finished_pb.finish_and_clear();
                            }
                        }
                    }
                } else {
                    let length = s_p.length.get();
                    if length != 0 {
                        let pb = imp.add(ProgressBar::new(length as u64));
                        pb.set_style(self.style.clone());
                        pb.set_message(&format!("{:>3}", i));
                        *s_b = Some(pb);
                        started_pb.inc(1);
                    }
                }
            }
        }
        // Finish all bars that are not yet finished (to end the indicatif::MultiProgress join
        // thread).
        for sp in mp.sub_progress.iter() {
            assert_eq!(sp.length(), sp.position());
        }

        for bar in [Some(started_pb), Some(finished_pb)]
            .iter()
            .chain(sub_bars.iter())
        {
            if let Some(bar) = bar {
                if !bar.is_finished() {
                    bar.finish();
                }
            }
        }
    }

    pub fn run<T, F: FnOnce(&MultiProgress) -> T>(&self, f: F) -> T {
        // Initialize the progress bar container and the star/end bars.
        let imp = indicatif::MultiProgress::new();
        let started_pb = imp.add(ProgressBar::new(self.n_bars as u64));
        started_pb.set_style(self.style.clone());
        started_pb.set_message("started ");
        let finished_pb = imp.add(ProgressBar::new(self.n_bars as u64));
        finished_pb.set_style(self.style.clone());
        finished_pb.set_message("finished");

        let mp = MultiProgress::new(self.n_bars);
        // Kill switch for the counters reader.
        let f_finished = AtomicBool::new(false);
        crossbeam_utils::thread::scope(|s| {
            // Make the actual progress bars run their display. This will finish once all the
            // progress bars are finished.
            s.spawn(|_| imp.join());

            // Run the reader of the counter, and update the progress bars accordingly.
            s.spawn(|_| self.run_progress_reader(&mp, &f_finished, &imp, started_pb, finished_pb));

            // Run the main function (in the same thread, so we don't need F: Send.
            let res = f(&mp);
            // Stop the reader thread.
            f_finished.store(true, Ordering::Release);

            res
        })
        .unwrap()
    }
}
impl MultiProgress {
    fn new(n_bars: usize) -> Self {
        Self {
            sub_progress: (0..n_bars)
                .map(|_| SubProgress {
                    end_is_finish: AtomicBool::new(false),
                    count: TrAdder::new(),
                    length: TrAdder::new(),
                })
                .collect(),
        }
    }
    pub fn sub(&self, i: usize) -> &SubProgress {
        &self.sub_progress[i]
    }
}
impl SubProgress {
    pub fn inc(&self, delta: i64) {
        self.count.inc(delta);
        assert!(self.length() >= self.position());
    }
    pub fn inc_length(&self, delta: i64) {
        self.length.inc(delta);
        assert!(self.length() >= self.position());
    }
    pub fn finishing(&self, end_is_finish: bool) {
        self.end_is_finish.store(end_is_finish, Ordering::Release);
    }
    pub fn position(&self) -> i64 {
        self.count.get()
    }
    pub fn length(&self) -> i64 {
        self.length.get()
    }
}
