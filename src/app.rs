use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // invoke without arguments
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke_without_args(cmd: &str) -> JsValue;

    // invoke with arguments (default)
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn App() -> impl IntoView {

    // let (name, set_name) = signal(String::new());
    let (greet_msg, set_greet_msg) = signal(String::new());
    let (connection_msg, set_connection_msg) = signal(String::new());
    let (connection_ticket, set_connection_ticket) = signal(String::new());

    // let update_name = move |ev| {
    //     let v = event_target_value(&ev);
    //     let new_msg = invoke_without_args("get_serialized_ticket").await.as_string().unwrap();
    //     set_name.set(v);
    // };

    let greet = move |_ev| {
        // ev.prevent_default();
        spawn_local(async move {

            // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
            let new_msg = invoke_without_args("get_serialized_ticket").await.as_string().unwrap();
            set_greet_msg.set(new_msg);
        });
    };

    let connect = move |_ev| {
        // ev.prevent_default();
        let ticket_value = connection_ticket.get();
        spawn_local(async move {
            // Create args with the ticket value
            let args = serde_json::json!({ "ticket": ticket_value });
            let args_js = serde_wasm_bindgen::to_value(&args).unwrap();

            // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
            let new_msg = invoke("connect_via_serialized_ticket", args_js).await.as_string().unwrap();
            set_connection_msg.set(new_msg);
        });
    };


    view! {
    <button id="theme-toggle" class="fixed top-4 right-4 z-50 p-2 rounded-lg bg-white dark:bg-gray-800 shadow-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200">

        <svg class="w-5 h-5 text-yellow-500 hidden dark:block" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z" clip-rule="evenodd"></path>
        </svg>

        <svg class="w-5 h-5 text-gray-700 block dark:hidden" fill="currentColor" viewBox="0 0 20 20">
            <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"></path>
        </svg>
    </button>


    <div class="connection-screen h-full flex items-center justify-center p-6">
        <div class="w-full max-w-md space-y-8 animate-fade-in">

            <div class="text-center">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">Chat Connect</h1>
                <p class="text-gray-600 dark:text-gray-400">Connect to start chatting</p>
            </div>

            <div class="text-center">
                <button on:click=greet class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-lg text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:focus:ring-offset-gray-900 transition-all duration-200 transform hover:scale-105 shadow-lg">
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z"></path>
                    </svg>
                    Share QR Code
                </button>
        <p class="text-gray-600 dark:text-gray-400 text-sm font-mono" style="word-break: break-word; overflow-wrap: break-word; hyphens: auto; max-width: 100%; white-space: normal;">{ move || greet_msg.get() }</p>
            </div>

            <div class="relative">
                <div class="absolute inset-0 flex items-center">
                    <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                </div>
                <div class="relative flex justify-center text-sm">
                    <span class="px-2 bg-gray-50 dark:bg-gray-900 text-gray-500 dark:text-gray-400">or</span>
                </div>
            </div>

            <div class="space-y-4">
                <div>
                    <label for="connection-ticket" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Connection Ticket
                    </label>
                    <textarea
                            id="connection-ticket"
                            rows="3"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-800 text-gray-900 dark:text-white resize-none transition-all duration-200"
                            placeholder="Paste your connection ticket here..."
                            prop:value=connection_ticket
                            on:input=move |ev| {
                                set_connection_ticket.set(event_target_value(&ev));
                            }

                    ></textarea>
                </div>

                <button on:click=connect class="w-full flex justify-center items-center px-4 py-2 border border-transparent text-base font-medium rounded-lg text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 dark:focus:ring-offset-gray-900 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed shadow-lg">
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                    </svg>
                    Connect & Start Chat
                </button>
            </div>
        </div>
    </div>

    <div class="chat-screen h-full flex flex-col">

        <header class="sticky top-0 z-40 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3 shadow-sm">
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-3">
                    <button class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-200 md:hidden">
                        <svg class="w-5 h-5 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
                        </svg>
                    </button>
                    <div class="flex items-center space-x-3">
                        <div class="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                            <span class="text-white font-semibold text-sm">U</span>
                        </div>
                        <div>
                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Chat Partner</h2>
                            <p class="text-sm text-green-500">Online</p>
                        </div>
                    </div>
                </div>
                <button class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-200">
                    <svg class="w-5 h-5 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"></path>
                    </svg>
                </button>
            </div>
        </header>

        <div class="flex-1 overflow-hidden">
            <div id="messages-container" class="messages-container h-full overflow-y-auto custom-scrollbar px-4 py-4">
                <div class="space-y-4">

                    <div class="flex justify-start animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                                <p class="text-gray-900 dark:text-white">Hey there! How are you doing?</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">2:30 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-end animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-blue-500 rounded-lg px-4 py-2 shadow-sm">
                                <p class="text-white">Im doing great! Thanks for asking. How about you?</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mr-2 text-right">2:31 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-start animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                                <p class="text-gray-900 dark:text-white">Pretty good! Just working on some projects. This chat interface looks really nice by the way.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">2:32 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-end animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-blue-500 rounded-lg px-4 py-2 shadow-sm">
                                <p class="text-white">Thanks! Im really happy with how it turned out. The dark mode looks especially clean.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mr-2 text-right">2:33 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-start animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                                <p class="text-gray-900 dark:text-white">Absolutely! The responsiveness is great too. Works well on both mobile and desktop.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">2:34 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-end animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-blue-500 rounded-lg px-4 py-2 shadow-sm">
                                <p class="text-white">That was definitely a priority. Mobile-first design is so important these days.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mr-2 text-right">2:35 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-start animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                                <p class="text-gray-900 dark:text-white">For sure! Well done on this project. Looking forward to using it more.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">2:36 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-end animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-blue-500 rounded-lg px-4 py-2 shadow-sm">
                                <p class="text-white">Thank you so much! Im excited to see how it evolves. </p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mr-2 text-right">2:37 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-start animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                                <p class="text-gray-900 dark:text-white">I have some ideas for additional features too. Maybe we can discuss them sometime.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">2:38 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-end animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-blue-500 rounded-lg px-4 py-2 shadow-sm">
                                <p class="text-white">That sounds great! Im always open to new ideas and improvements.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mr-2 text-right">2:39 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-start animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                                <p class="text-gray-900 dark:text-white">Perfect! Ill prepare a list and we can go through them together.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">2:40 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-end animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-blue-500 rounded-lg px-4 py-2 shadow-sm">
                                <p class="text-white">Looking forward to it! This collaboration is going to be awesome.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mr-2 text-right">2:41 PM</p>
                        </div>
                    </div>

                    <div class="flex justify-start animate-slide-up">
                        <div class="max-w-xs lg:max-w-md">
                            <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                                <p class="text-gray-900 dark:text-white">Definitely! By the way, have you tested the scroll functionality? It should work smoothly.</p>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">2:42 PM</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-4 py-3">
            <div class="flex items-center space-x-3">
                <div class="flex-1">
                    <textarea
                            rows="1"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white resize-none max-h-32"
                            placeholder="Type your message..."
                    ></textarea>
                </div>
                <button class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg font-medium transition-colors duration-200 flex items-center justify-center min-w-[60px]">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
                    </svg>
                </button>
            </div>
        </div>
    </div>
    <script>
        r#"
        // Theme toggle functionality
        const themeToggle = document.getElementById('theme-toggle');
        const html = document.documentElement;
    
        themeToggle.addEventListener('click', () => {
          html.classList.toggle('dark');
          localStorage.setItem('theme', html.classList.contains('dark') ? 'dark' : 'light');
        });
    
        // Load saved theme or default to light
        const savedTheme = localStorage.getItem('theme');
        if (savedTheme === 'dark' || (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
          html.classList.add('dark');
        }
        "#
        </script>
        }
}
