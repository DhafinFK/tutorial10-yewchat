use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
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
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
    
        html! {
            <div class="flex w-screen">
                // Users section
                <div class="flex-none w-56 h-screen bg-blue-900 text-white shadow-lg">
                    <div class="text-xl p-4 font-semibold">{"Users"}</div>
                    {
                        for self.users.iter().map(|u| {
                            html! {
                                <div class="flex m-4 bg-blue-100 rounded-xl shadow-md p-3">
                                    <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    <div class="flex-grow ml-4">
                                        <div class="text-sm font-medium">{&u.name}</div>
                                        <div class="text-xs text-blue-900">{"Hi there!"}</div>
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>
                // Chat section
                <div class="grow h-screen flex flex-col bg-blue-50">
                    <div class="w-full h-16 border-b-2 border-blue-300 flex items-center pl-4 bg-blue-200">
                        <div class="text-xl font-semibold text-gray-800">{"💬 Chat"}</div>
                    </div>
                    <div class="flex-grow overflow-auto">
                        {
                            for self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html! {
                                    <div class={format!("flex items-end m-8 rounded-lg {}", if m.from == "You" { "bg-red-100" } else { "bg-green-100" })}>
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="flex flex-col p-3">
                                            <div class="text-sm font-medium">{&m.from}</div>
                                            <div class="text-xs text-gray-800 mt-1">
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html! { <img class="mt-1" src={m.message.clone()} /> }
                                                    } else {
                                                        html! { {&m.message} }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            })
                        }
                    </div>
                    <div class="w-full h-16 flex px-4 items-center bg-white">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Type a message..." class="block w-full py-2 pl-4 mx-3 bg-gray-200 rounded-full outline-none focus:bg-white" name="message" required=true />
                        <button onclick={submit} class="ml-3 p-2 bg-blue-600 w-12 h-12 rounded-full flex justify-center items-center text-white">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-current">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}