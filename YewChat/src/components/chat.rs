use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{User, services::websocket::WebsocketService};
use crate::services::event_bus::EventBus;

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleSidebar,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
    sidebar_visible: bool,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            sidebar_visible: true,
        }
    }
    
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
            Msg::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                true
            }
        }
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let on_keypress = ctx.link().batch_callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                Some(Msg::SubmitMessage)
            } else {
                None
            }
        });
        let toggle_sidebar = ctx.link().callback(|_| Msg::ToggleSidebar);

        html! {
            <div class="flex h-screen w-full bg-gray-50">
                // Sidebar with responsive design
                <div class={classes!(
                    "bg-white", "shadow-lg", "transition-all", "duration-300",
                    "md:block", // Always show on medium screens and above
                    if self.sidebar_visible { "w-72" } else { "w-0 md:w-72" },
                    if !self.sidebar_visible { "hidden" } else { "" }
                )}>
                    <div class="py-4 px-5 border-b border-gray-200">
                        <h2 class="text-xl font-semibold text-gray-800 flex items-center">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 mr-2 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                            </svg>
                            {"Online Users"}
                        </h2>
                    </div>
                    <div class="overflow-y-auto" style="max-height: calc(100vh - 68px);">
                        {
                            if self.users.is_empty() {
                                html! {
                                    <div class="py-8 px-5 text-center text-gray-500">
                                        {"No users online at the moment"}
                                    </div>
                                }
                            } else {
                                self.users.clone().iter().map(|u| {
                                    html! {
                                        <div class="flex items-center px-5 py-3 hover:bg-gray-50 transition-colors cursor-pointer">
                                            <div class="relative">
                                                <img class="w-12 h-12 rounded-full object-cover border-2 border-white shadow-sm" src={u.avatar.clone()} alt="avatar"/>
                                                <div class="absolute bottom-0 right-0 h-3 w-3 rounded-full bg-green-400 border-2 border-white"></div>
                                            </div>
                                            <div class="ml-3">
                                                <div class="font-medium text-gray-800">{u.name.clone()}</div>
                                                <div class="text-xs text-gray-500">{"Online"}</div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        }
                    </div>
                </div>

                <div class="flex-1 flex flex-col w-full">
                    <div class="bg-white border-b border-gray-200 px-6 py-4 shadow-sm">
                        <div class="flex items-center justify-between">
                            <div class="flex items-center">
                                // Mobile toggle for sidebar
                                <button 
                                    onclick={toggle_sidebar} 
                                    class="md:hidden mr-4 text-gray-500 hover:text-gray-700 focus:outline-none"
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                                    </svg>
                                </button>
                                <div class="h-10 w-10 rounded-full bg-blue-100 flex items-center justify-center text-blue-500">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                                    </svg>
                                </div>
                                <div class="ml-4">
                                    <h2 class="text-lg font-semibold text-gray-800">{"Group Chat"}</h2>
                                    <p class="text-sm text-gray-500">{format!("{} participants", self.users.len())}</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="flex-1 overflow-y-auto p-6 bg-gray-50" style="scrollbar-width: thin;">
                        {
                            if self.messages.is_empty() {
                                html! {
                                    <div class="flex flex-col items-center justify-center h-full text-gray-500">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mb-4 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                                        </svg>
                                        {"No messages yet. Start the conversation!"}
                                    </div>
                                }
                            } else {
                                self.messages.iter().map(|m| {
                                    let default_profile = UserProfile { 
                                        name: m.from.clone(), 
                                        avatar: format!("https://avatars.dicebear.com/api/adventurer-neutral/{}.svg", m.from)
                                    };
                                    let user = self.users.iter().find(|u| u.name == m.from).unwrap_or(&default_profile);
                                    
                                    html! {
                                        <div class="flex mb-4 items-end">
                                            <div class="flex-shrink-0">
                                                <img class="w-8 h-8 rounded-full" src={user.avatar.clone()} alt="avatar"/>
                                            </div>
                                            <div class="ml-2 max-w-xl lg:max-w-2xl">
                                                <div class="font-medium text-sm text-gray-700">{user.name.clone()}</div>
                                                <div class="bg-white p-3 rounded-lg shadow-sm mt-1">
                                                    if m.message.ends_with(".gif") {
                                                        <img class="rounded-lg max-w-full" src={m.message.clone()}/>
                                                    } else {
                                                        <p class="text-gray-800">{m.message.clone()}</p>
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        }
                    </div>

                    <div class="bg-white border-t border-gray-200 px-6 py-3">
                        <div class="flex items-center">
                            <input 
                                ref={self.chat_input.clone()} 
                                type="text" 
                                placeholder="Type your message here..." 
                                class="block w-full px-4 py-3 bg-gray-100 rounded-full outline-none focus:ring-2 focus:ring-blue-400 focus:bg-white"
                                onkeypress={on_keypress}
                            />
                            <button 
                                onclick={submit} 
                                class="ml-3 px-4 py-3 bg-blue-500 hover:bg-blue-600 rounded-full text-white shadow-sm transition"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}