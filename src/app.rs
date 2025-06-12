use chrono::{DateTime, Utc};
use fast_qr::convert::{Builder, Shape, svg::SvgBuilder};
use fast_qr::qr::QRBuilder;
use futures::StreamExt;
use ipc_layer as api;
use ipc_layer::events;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use tauri_sys::event::listen;

/// Delineate incoming vs outgoing messages in the chat so they can render differently.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LabeledMessage {
    Incoming(api::Message),
    Outgoing(api::Message),
}

impl LabeledMessage {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            LabeledMessage::Incoming(m) => m.timestamp(),
            LabeledMessage::Outgoing(m) => m.timestamp(),
        }
    }
}

#[component]
pub fn Message(msg: LabeledMessage) -> impl IntoView {
    match msg {
        LabeledMessage::Incoming(m) => {
            let (msg, timestamp) = m.unpack_for_html_integration();
            view! {
                <div class="flex justify-start animate-slide-up">
                    <div class="max-w-xs lg:max-w-md">
                        <div class="bg-white dark:bg-gray-700 rounded-lg px-4 py-2 shadow-sm border border-gray-200 dark:border-gray-600">
                            <p class="text-gray-900 dark:text-white">{msg}</p>
                        </div>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-2">
                            {timestamp}
                        </p>
                    </div>
                </div>
            }
        }
        LabeledMessage::Outgoing(m) => {
            let (msg, timestamp) = m.unpack_for_html_integration();
            view! {
                <div class="flex justify-end animate-slide-up">
                    <div class="max-w-xs lg:max-w-md">
                        <div class="bg-blue-500 rounded-lg px-4 py-2 shadow-sm">
                            <p class="text-white">{msg}</p>
                        </div>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 mr-2 text-right">
                            {timestamp}
                        </p>
                    </div>
                </div>
            }
        }
    }
}

#[component]
pub fn Chat(connection_type: ReadSignal<String>) -> impl IntoView {
    // signal to handle a vector of all messages sent and received in this chat session
    let (messages, set_messages) = signal(vec![]);
    // signal to handle the input of messages to the text area by the user.
    let (send_message, set_send_message) = signal(String::new());

    // listen for incoming messages and add them to the messages vector
    spawn_local(async move {
        let mut incoming_messages = events::ui::conversation::listen()
            .await
            .expect("there should be a valid message incoming");
        while let Some(msg) = incoming_messages.next().await {
            log!("Received message: {:?}", msg);
            let labeled_msg = LabeledMessage::Incoming(msg.payload);
            set_messages.update(|messages| {
                messages.push(labeled_msg);
            });
        }
    });

    // adds new messages created by the user and sends them out
    // todo: handle message send failures
    // todo: allow sending on keyboard "enter" key press
    let send_out = move |_ev| {
        let msg = send_message.get();
        if !msg.is_empty() {
            let msg = api::Message::new(msg);
            let labeled_msg = LabeledMessage::Outgoing(msg.clone());
            set_messages.update(|messages| messages.push(labeled_msg));
            // todo: we have no check to make sure the message broadcasts here, we should add this
            spawn_local(async move {
                api::ui::broadcast_message(msg)
                    .await
                    .expect("should send a valid message");
            });
        }
    };

    view! {
        <div class="fixed inset-0 flex flex-col bg-gray-50 dark:bg-gray-900">

            <header class="flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3 shadow-sm">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-3">
                        <button class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-200 md:hidden">
                            <svg
                                class="w-5 h-5 text-gray-600 dark:text-gray-400"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M15 19l-7-7 7-7"
                                ></path>
                            </svg>
                        </button>
                        <div class="flex items-center space-x-3">
                            <div class="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                                <span class="text-white font-semibold text-sm">U</span>
                            </div>
                            <div>
                                <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
                                    Connection Type
                                </h2>
                                <p class="text-sm text-green-500">
                                    {move || connection_type.get()}
                                </p>
                            </div>
                        </div>
                    </div>
                    <button class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-200">
                        <svg
                            class="w-5 h-5 text-gray-600 dark:text-gray-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
                            ></path>
                        </svg>
                    </button>
                </div>
            </header>

            <div class="flex-1 min-h-0 overflow-hidden">
                <div id="messages-container" class="h-full overflow-y-auto px-4 py-4">
                    <div class="space-y-4">
                        // todo: is this the most efficient way to render messages?  this will likely result in poor performance for large chats.
                        <For
                            each=move || messages.get()
                            key=|message| message.timestamp().to_string()
                            children=move |message| {
                                view! { <Message msg=message /> }
                            }
                        />

                    </div>
                </div>
            </div>

            <div class="flex-shrink-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-4 py-3">
                <div class="flex items-center space-x-3">
                    <div class="flex-1">
                        <textarea
                            rows="1"
                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white resize-none max-h-32"
                            placeholder="Type your message..."
                            prop:value=send_message
                            on:input=move |ev| {
                                set_send_message.set(event_target_value(&ev));
                            }
                        ></textarea>
                    </div>
                    <button
                        on:click=send_out
                        class="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg font-medium transition-colors duration-200 flex items-center justify-center min-w-[60px]"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                            ></path>
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // todo: implement proper error handling across the app.
    // todo: adjust animations per accessibility preferences in system
    // signal to present the node ticket when it has been created
    let (this_nodes_ticket, set_this_nodes_ticket) = signal(String::new());
    // signal to build and present the qr code for the node ticket
    let (this_nodes_ticket_qr, set_this_nodes_ticket_qr) = signal(String::new());
    // signal to set a connection message indicated what Document we just connected to (used mostly for debugging at this time)
    let (connection_msg, set_connection_msg) = signal(String::new());
    // signal to manage the input of a connection event into a text area
    let (connection_ticket, set_connection_ticket) = signal(String::new());
    // signal to indicate we have connected to a chat session and will cause a switch to the chat screen
    let (is_connected, set_is_connected) = signal(false);
    // signal to set the connection type on the chat screen (direct, mixed, etc.)
    let (connection_type, set_connection_type) = signal(String::new());

    spawn_local(async move {
        // todo: migrate to once listen once implemented in the macro for ipc layer
        // once we receive the connection event, we can set the connected state to true and stop listening further
        let connection_events = events::ui::connection::listen()
            .await
            .expect("there should be a valid connection event");
        let (mut events, abort_handle) = futures::stream::abortable(connection_events);
        while let Some(msg) = events.next().await {
            if msg.payload == "connected" {
                set_is_connected.set(true);
                break;
            }
        }
        abort_handle.abort();
    });

    // spawned task to listen for and set the connection type for all send and receives of messages as they stream in and out.
    spawn_local(async move {
        let mut connection_updates = events::ui::connection_type::listen()
            .await
            .expect("there should be a valid connection update incoming");
        while let Some(msg) = connection_updates.next().await {
            log!("Received message: {:?}", msg);
            let connection_type = msg.payload;
            set_connection_type.set(connection_type);
        }
    });

    let display_ticket = move |_ev| {
        spawn_local(async move {
            let ticket = api::ui::get_serialized_ticket()
                .await
                .expect("should produce a valid ticket");
            set_this_nodes_ticket.set(ticket.clone());

            // QRBuilder::new can fail if content is too big for version,
            // please check before unwrapping.
            // todo: implement safety checks.
            // todo: there was an odd wasm out of bound memory access error form fast_qr that only happened once during testing.  Will need to investigate further.
            let qrcode = QRBuilder::new(ticket).build().unwrap();

            let svg = SvgBuilder::default()
                .shape(Shape::RoundedSquare)
                .to_str(&qrcode);
            set_this_nodes_ticket_qr.set(svg);
        });
    };

    let connect = move |_ev| {
        let ticket_value = connection_ticket.get();
        spawn_local(async move {
            let new_msg = api::ui::connect_via_serialized_ticket(ticket_value)
                .await
                .expect("should consume a valid ticket");
            set_connection_msg.set(new_msg);
            set_is_connected.set(true); // Set connected state
        });
    };

    let scan_qr_code = view! {
        {move || {
            #[cfg(feature = "mobile")]
            {
                let scan = move |_ev| {
                    spawn_local(async move {
                        let new_msg = api::barcode_scanner::scan_barcode(
                                api::barcode_scanner::Format::QRCode,
                                false,
                                api::barcode_scanner::CameraDirection::Back,
                            )
                            .await;
                        set_connection_ticket.set(new_msg.content)
                    });
                };

                view! {
                    <button
                        on:click=scan
                        class="w-full flex justify-center items-center px-4 py-2 border border-transparent text-base font-medium rounded-lg text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 dark:focus:ring-offset-gray-900 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed shadow-lg"
                    >
                        <svg
                            class="w-5 h-5 mr-2"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
                            ></path>
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"
                            ></path>
                        </svg>
                        "Scan QR Code"
                    </button>
                }
            }
            #[cfg(not(feature = "mobile"))]
            {

                view! { <div></div> }
            }
        }}
    };

    view! {
        <button
            id="theme-toggle"
            class="fixed top-4 right-4 z-50 p-2 rounded-lg bg-white dark:bg-gray-800 shadow-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-200"
        >

            <svg
                class="w-5 h-5 text-yellow-500 hidden dark:block"
                fill="currentColor"
                viewBox="0 0 20 20"
            >
                <path
                    fill-rule="evenodd"
                    d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"
                    clip-rule="evenodd"
                ></path>
            </svg>

            <svg
                class="w-5 h-5 text-gray-700 block dark:hidden"
                fill="currentColor"
                viewBox="0 0 20 20"
            >
                <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"></path>
            </svg>
        </button>

        {move || {
            if is_connected.get() {
                view! { <Chat connection_type=connection_type /> }.into_any()
            } else {
                view! {
                    <div class="h-full flex items-center justify-center p-6">
                        <div class="w-full max-w-md space-y-8 animate-fade-in">

                            <div class="text-center">
                                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                                    Chat Connect
                                </h1>
                                <p class="text-gray-600 dark:text-gray-400">
                                    Connect to start chatting
                                </p>
                            </div>

                            <div class="text-center">
                                <button
                                    on:click=display_ticket
                                    class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-lg text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:focus:ring-offset-gray-900 transition-all duration-200 transform hover:scale-105 shadow-lg"
                                >
                                    <svg
                                        class="w-5 h-5 mr-2"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z"
                                        ></path>
                                    </svg>
                                    Share QR Code
                                </button>
                                <div inner_html=move || this_nodes_ticket_qr.get()></div>
                                <p
                                    class="text-gray-600 dark:text-gray-400 text-sm font-mono"
                                    style="word-break: break-word; overflow-wrap: break-word; hyphens: auto; max-width: 100%; white-space: normal;"
                                >
                                    {move || this_nodes_ticket.get()}
                                </p>
                            </div>

                            <div class="relative">
                                <div class="absolute inset-0 flex items-center">
                                    <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                                </div>
                                <div class="relative flex justify-center text-sm">
                                    <span class="px-2 bg-gray-50 dark:bg-gray-900 text-gray-500 dark:text-gray-400">
                                        or
                                    </span>
                                </div>
                            </div>

                            <div class="space-y-4">
                                {scan_qr_code} <div>
                                    <label
                                        for="connection-ticket"
                                        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                                    >
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
                                <button
                                    on:click=connect
                                    class="w-full flex justify-center items-center px-4 py-2 border border-transparent text-base font-medium rounded-lg text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 dark:focus:ring-offset-gray-900 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed shadow-lg"
                                >
                                    <svg
                                        class="w-5 h-5 mr-2"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M13 10V3L4 14h7v7l9-11h-7z"
                                        ></path>
                                    </svg>
                                    Connect & Start Chat
                                </button>
                                <p
                                    class="text-gray-600 dark:text-gray-400 text-sm font-mono"
                                    style="word-break: break-word; overflow-wrap: break-word; hyphens: auto; max-width: 100%; white-space: normal;"
                                >
                                    {move || connection_msg.get()}
                                </p>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            }
        }}
        // todo: Replace this with proper rust implementation through leptos, should also respect platform preferences.
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
