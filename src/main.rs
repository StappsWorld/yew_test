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
            <div class="h-full w-full flex flex-col justify-center items-center text-center text-white dark:text-gray-50">
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
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}
}

fn main() {
    yew::start_app::<Model>();
}
