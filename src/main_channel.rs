use cursive::Cursive;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::*;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread;

pub enum UIMessage {
    RefreshTopics {
        bootstrap: String
    },
    DeleteTopic {
        topic: String
    },
}

pub struct ThreadReceiver {
    pub receiver: Receiver<UIMessage>,
    pub shared_cursive: Arc<Mutex<Cursive>>,
}

unsafe impl Send for ThreadReceiver {}

pub fn of(shared_cursive: Arc<Mutex<Cursive>>) -> Sender<UIMessage> {
    let (sender, receiver): (Sender<UIMessage>, Receiver<UIMessage>) = mpsc::channel();

    let thread_f = |wrapper: ThreadReceiver| {
        move || {
            for msg in wrapper.receiver {
                receive(msg, wrapper.shared_cursive.lock().unwrap())
            };
        }
    };

    thread::spawn(thread_f(ThreadReceiver { receiver, shared_cursive }));

    sender.send(UIMessage::RefreshTopics { bootstrap: String::from("localhost:9092") });
    sender
}

fn receive(msg: UIMessage, mut cursive: MutexGuard<Cursive>) {
    match msg {
        UIMessage::RefreshTopics { bootstrap } => unimplemented!(),
        UIMessage::DeleteTopic { topic } => unimplemented!()
    }
}