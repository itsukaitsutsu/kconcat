Compiling i-slint-backend-winit v1.18.1
error[E0599]: no variant or associated item named `Karaoke` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:234:54
    |
234 |     if matches!(mode, MotionMode::None | MotionMode::Karaoke)
    |                                                      ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Karaoke` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:249:40
    |
249 |         MotionMode::None | MotionMode::Karaoke => return None,
    |                                        ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Fade` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:250:21
    |
250 |         MotionMode::Fade | MotionMode::Pop | MotionMode::Bounce | MotionMode::SlideUp => {
    |                     ^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Bounce` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:250:58
    |
250 |         MotionMode::Fade | MotionMode::Pop | MotionMode::Bounce | MotionMode::SlideUp => {
    |                                                          ^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `SlideUp` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:250:79
    |
250 |         MotionMode::Fade | MotionMode::Pop | MotionMode::Bounce | MotionMode::SlideUp => {
    |                                                                               ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Fade` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:257:33
    |
257 |                     MotionMode::Fade => (1.0, 0.0, ease_out),
    |                                 ^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Bounce` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:271:33
    |
271 |                     MotionMode::Bounce => {
    |                                 ^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `SlideUp` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:284:33
    |
284 |                     MotionMode::SlideUp => (1.0, 0.075 * (1.0 - ease_out), ease_out),
    |                                 ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0609]: no field `stagger` on type `CaptionMotion`
   --> crates\concat-host\src\titles.rs:308:32
    |
308 |                 + style.motion.stagger * word_count.saturating_sub(1) as f64;
    |                                ^^^^^^^ unknown field
    |
    = note: available fields are: `mode`, `duration`, `scale_from`

error[E0609]: no field `stagger` on type `CaptionMotion`
   --> crates\concat-host\src\titles.rs:319:40
    |
319 |                         + style.motion.stagger * word_pauses as f64)
    |                                        ^^^^^^^ unknown field
    |
    = note: available fields are: `mode`, `duration`, `scale_from`

error[E0599]: no variant or associated item named `Karaoke` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:607:80
    |
607 |                     if style.motion.mode == concat_project::model::MotionMode::Karaoke
    |                                                                                ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Karaoke` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:611:96
    |
611 |                     let synthetic = if style.motion.mode == concat_project::model::MotionMode::Karaoke
    |                                                                                                ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Karaoke` found for enum `concat_project::model::MotionMode` in the current scope
   --> crates\concat-host\src\titles.rs:624:104
    |
624 |                         let karaoke_style = if style.motion.mode == concat_project::model::MotionMode::Karaoke
    |                                                                                                        ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Fade` found for enum `concat_project::model::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1133:52
     |
1133 |                 concat_project::model::MotionMode::Fade => concat_text::MotionMode::Fade,
     |                                                    ^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Bounce` found for enum `concat_project::model::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1135:52
     |
1135 |                 concat_project::model::MotionMode::Bounce => concat_text::MotionMode::Bounce,
     |                                                    ^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `SlideUp` found for enum `concat_project::model::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1136:52
     |
1136 |                 concat_project::model::MotionMode::SlideUp => concat_text::MotionMode::SlideUp,
     |                                                    ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Karaoke` found for enum `concat_project::model::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1139:52
     |
1139 |                 concat_project::model::MotionMode::Karaoke => concat_text::MotionMode::Karaoke,
     |                                                    ^^^^^^^ variant or associated item not found in `concat_project::model::MotionMode`

error[E0599]: no variant or associated item named `Fade` found for enum `concat_text::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1133:85
     |
1133 |                 concat_project::model::MotionMode::Fade => concat_text::MotionMode::Fade,
     |                                                                                     ^^^^ variant or associated item not found in `concat_text::MotionMode`

error[E0599]: no variant or associated item named `Bounce` found for enum `concat_text::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1135:87
     |
1135 |                 concat_project::model::MotionMode::Bounce => concat_text::MotionMode::Bounce,
     |                                                                                       ^^^^^^ variant or associated item not found in `concat_text::MotionMode`

error[E0599]: no variant or associated item named `SlideUp` found for enum `concat_text::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1136:88
     |
1136 |                 concat_project::model::MotionMode::SlideUp => concat_text::MotionMode::SlideUp,
     |                                                                                        ^^^^^^^ variant or associated item not found in `concat_text::MotionMode`

error[E0599]: no variant or associated item named `Karaoke` found for enum `concat_text::MotionMode` in the current scope
    --> crates\concat-host\src\titles.rs:1139:88
     |
1139 |                 concat_project::model::MotionMode::Karaoke => concat_text::MotionMode::Karaoke,
     |                                                                                        ^^^^^^^ variant or associated item not found in `concat_text::MotionMode`

error[E0560]: struct `Motion` has no field named `stagger`
    --> crates\concat-host\src\titles.rs:1142:13
     |
1142 |             stagger: style.motion.stagger,
     |             ^^^^^^^ `Motion` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `stagger` on type `CaptionMotion`
    --> crates\concat-host\src\titles.rs:1142:35
     |
1142 |             stagger: style.motion.stagger,
     |                                   ^^^^^^^ unknown field
     |
     = note: available fields are: `mode`, `duration`, `scale_from`

Some errors have detailed explanations: E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `concat-host` (lib) due to 23 previous errors
warning: build failed, waiting for other jobs to finish...

C:\dev\kconcat\src>