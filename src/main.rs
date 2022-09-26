use gloo::timers::callback::Interval;
use num_bigint::{BigUint, ToBigUint};
use num_traits::{ops::checked::CheckedMul, One, Zero};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{events::Event, html, Component, Context, Html};
use yew_test::FPSCounter;

pub mod lib;

enum Msg {
    UpdateTime,
    PauseTime,
    ModulusChange(u32),
}

struct Model {
    value: BigUint,
    _clock: Interval,
    pause_time: Arc<AtomicBool>,
    power: BigUint,
    fps_counter: FPSCounter,
    modulus: u32,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let pause_time = Arc::new(AtomicBool::new(false));
        let clock_handle = {
            let link = ctx.link().clone();
            let pause_time = pause_time.clone();
            Interval::new(1, move || {
                if !pause_time.load(Relaxed) {
                    link.send_message(Msg::UpdateTime)
                }
            })
        };
        let fps_counter: FPSCounter = FPSCounter::new();
        Self {
            value: One::one(),
            _clock: clock_handle,
            pause_time,
            power: One::one(),
            fps_counter,
            modulus: 1,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateTime => {
                self.fps_counter.tick();
                self.power += 1_u32;
                self.value = self
                    .value
                    .checked_mul(&2_u32.to_biguint().unwrap())
                    .unwrap_or_else(|| {
                        self.power = One::one();
                        One::one()
                    });
                self.power
                    .modpow(&One::one(), &self.modulus.to_biguint().unwrap())
                    == Zero::zero()
            }
            Msg::PauseTime => {
                self.pause_time
                    .store(!self.pause_time.load(Relaxed), Relaxed);
                true
            }
            Msg::ModulusChange(modulus) => {
                self.modulus = modulus;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let modulus_change = link.callback(|e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .expect("Could not get input element");
            Msg::ModulusChange(input.value().parse::<u32>().expect("Could not parse input"))
        });

        html! {
            <div class="h-full w-full flex flex-col justify-center items-center text-center text-gray-700 dark:text-gray-50">
                <p class="text-2xl w-1/4">{ format!("FPS: {:03}", self.fps_counter.get_tick()) }</p>
                <p class="text-2xl w-1/4">{ format!("2^{}", self.power,) }</p>
                <p class="text-xl">{ format!("{}", self.value) }</p>
                <button
                        onclick={link.callback(|_| Msg::PauseTime)}
                        class="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-50 rounded-md shadow-md hover:bg-gray-300 dark:hover:bg-gray-600"
                    >{ "Pause" }</button>
                <div class="mt-10">
                    <p class="mb-2">{ format!("Displaying on current modulus (lower is smoother, but less performant). Currently: {}", self.modulus) }</p>
                    <input onchange={modulus_change} type="range" min="1" max="500" value={ format!("{}", self.modulus) } />
                </div>

                <a href="https://github.com/StappsWorld/yew_test" target="_blank"
                ><svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="20"
                    height="20"
                    class="fill-current text-gray-700 dark:text-gray-50 hover:text-gray-300 dark:hover:text-gray-400"
                    viewBox="0 0 16 16"
                >
                    <path
                        d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"
                    />
                    </svg>
                </a>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}
}

fn main() {
    yew::start_app::<Model>();
}
