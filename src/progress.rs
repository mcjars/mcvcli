//! modified code from [linya]: https://lib.rs/crates/linya

#![warn(missing_docs)]

use human_bytes::human_bytes;
use std::io::{BufWriter, Stderr, Write};

/// A progress bar "coordinator" to share between threads.
#[derive(Debug)]
pub struct Progress {
    /// The drawable bars themselves.
    bars: Vec<SubBar>,
    /// A shared handle to `Stderr`.
    ///
    /// Buffered so that the cursor doesn't jump around unpleasantly.
    out: BufWriter<Stderr>,
    /// Terminal width and height.
    size: Option<(usize, usize)>,
}

impl Default for Progress {
    fn default() -> Progress {
        Progress::new()
    }
}

// You will notice in a number of the methods below that `Result` values from
// calling `write!` are being ignored via a `let _ = ...` pattern, as opposed to
// unwrapping. This avoids a rare panic that can occur under very specific shell
// piping scenarios.
impl Progress {
    /// Initialize a new progress bar coordinator.
    pub fn new() -> Progress {
        let out = BufWriter::new(std::io::stderr());
        let bars = vec![];
        let size = term_size::dimensions();
        Progress { bars, out, size }
    }

    /// Like [`Progress::new`] but accepts a size hint to avoid reallocation as bar count grows.
    pub fn with_capacity(capacity: usize) -> Progress {
        let out = BufWriter::new(std::io::stderr());
        let bars = Vec::with_capacity(capacity);
        let size = term_size::dimensions();
        Progress { bars, out, size }
    }

    /// Create a new progress bar with default styling and receive an owned
    /// handle to it.
    ///
    /// # Panics
    ///
    /// Passing `0` to this function will cause a panic the first time a draw is
    /// attempted.
    pub fn bar<S: Into<String>>(&mut self, total: usize, label: S) -> Bar {
        let twidth = self.size.map(|(w, _)| w).unwrap_or(100);
        let w = (twidth / 2) - 7;
        let label: String = label.into();

        // An initial "empty" rendering of the new bar.
        let _ = writeln!(
            self.out,
            "{:<l$}             {:░>f$}  0%",
            label,
            "",
            l = twidth - w - 8,
            f = w
        );
        let _ = self.out.flush();

        let bar = SubBar {
            curr: 0,
            prev_percent: 0,
            total,
            label,
            cancelled: false,
        };
        self.bars.push(bar);
        Bar(self.bars.len() - 1)
    }

    /// Set a particular [`Bar`]'s progress value, but don't draw it.
    pub fn set(&mut self, bar: &Bar, value: usize) {
        self.bars[bar.0].curr = value;
    }

    /// Force the drawing of a particular [`Bar`].
    ///
    /// **Note 1:** Drawing will only occur if there is something meaningful to
    /// show. Namely, if the progress has advanced at least 1% since the last
    /// draw.
    ///
    /// **Note 2:** If your program is not being run in a terminal, an initial
    /// empty bar will be printed but never refreshed.
    pub fn draw(&mut self, bar: &Bar) {
        self.draw_impl(bar, false);

        // Very important, or the output won't appear fluid.
        let _ = self.out.flush();
    }

    /// Actually draw a particular [`Bar`].
    ///
    /// When `force` is true draw the bar at the current cursor position and
    /// advance the cursor one line.
    ///
    /// This function does not flush the output stream.
    fn draw_impl(&mut self, bar: &Bar, force: bool) {
        // If there is no legal width value present, that means we aren't
        // running in a terminal, and no rerendering can be done.
        if let Some((term_width, term_height)) = self.size {
            let pos = self.bars.len() - bar.0;
            let b = &mut self.bars[bar.0];
            let cur_percent = (100 * b.curr as u64) / (b.total as u64);
            // For a newly cancelled bar `diff` is equal to 100.
            let diff = cur_percent - b.prev_percent as u64;

            // For now, if the progress for a particular bar is slow and drifts
            // past the top of the terminal, redrawing is paused.
            if (pos < term_height && diff >= 1) || force {
                let w = (term_width / 2) - 7;
                let data = human_bytes(b.curr as f64);
                b.prev_percent = cur_percent as usize;

                if !force {
                    // Save cursor position and then move up `pos` lines.
                    let _ = write!(self.out, "\x1B[s\x1B[{pos}A\r");
                }

                let _ = write!(
                    self.out,
                    "{:<l$} {:10}  ",
                    b.label,
                    data,
                    l = term_width - w - 8,
                );
                if b.cancelled {
                    let _ = write!(self.out, "{:░>f$} ??? ", "", f = w);
                } else if b.curr >= b.total {
                    let _ = write!(self.out, "{:█>f$} 100%", "", f = w);
                } else {
                    let f = (((w as u64) * (b.curr as u64) / (b.total as u64)) as usize).min(w - 1);
                    let e = (w - 1) - f;

                    let _ = write!(
                        self.out,
                        "{:█>f$}▒{:░>e$} {:3}%",
                        "",
                        "",
                        (100 * (b.curr as u64)) / (b.total as u64),
                        f = f,
                        e = e
                    );
                }

                if !force {
                    // Return to previously saved cursor position.
                    let _ = write!(self.out, "\x1B[u\r");
                } else {
                    let _ = writeln!(self.out);
                }
            }
        }
    }

    /// Increment a given [`Bar`]'s progress, but don't draw it.
    pub fn inc(&mut self, bar: &Bar, value: usize) {
        self.set(bar, self.bars[bar.0].curr + value)
    }

    /// Increment a given [`Bar`]'s progress and immediately try to draw it.
    pub fn inc_and_draw(&mut self, bar: &Bar, value: usize) {
        self.inc(bar, value);
        self.draw(bar);
    }
}

/// An internal structure that stores individual bar state.
#[derive(Debug)]
struct SubBar {
    /// Progress as of the previous draw in percent.
    prev_percent: usize,
    /// Current progress.
    curr: usize,
    /// The progress target.
    total: usize,
    /// A user-supplied label for the left side of the bar line.
    label: String,
    /// Did the user force this bar to stop?
    cancelled: bool,
}

/// A progress bar index for use with [`Progress`].
///
/// This type has no meaningful methods of its own. Individual bars are advanced
/// by method calls on `Progress`:
///
/// ```
/// use linya::Progress;
///
/// let mut progress = Progress::new();
/// let bar = progress.bar(100, "Downloading");
/// progress.inc_and_draw(&bar, 1);
/// ```
///
/// As shown above, this type can only be constructed via [`Progress::bar`].
#[derive(Debug)]
pub struct Bar(usize);
