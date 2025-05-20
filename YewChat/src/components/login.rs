use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).clone())
    };

    html! {
        <div class="bg-gradient-to-r from-indigo-600 to-purple-600 min-h-screen flex items-center">
            <div class="container mx-auto px-4">
                <div class="max-w-md mx-auto bg-white rounded-xl shadow-lg p-6">
                    <h1 class="text-2xl font-bold text-center text-gray-800 mb-6">{"Welcome to YewChat"}</h1>
                    
                    <div class="flex flex-col">
                        <div class="mb-4">
                            <input 
                                oninput={oninput} 
                                class="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent" 
                                placeholder="Username"
                            />
                        </div>
                        
                        <div>
                            <Link<Route> to={Route::Chat} classes="block w-full">
                                <button 
                                    onclick={onclick} 
                                    disabled={username.len() < 1} 
                                    class="w-full rounded-lg bg-purple-600 hover:bg-purple-700 text-white font-medium py-3 px-4 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {"Go Chatting!"}
                                </button>
                            </Link<Route>>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}