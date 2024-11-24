use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chapter_5_channels::blocking_oneshot_channel::Channel as BloockingChannel;
use chapter_5_channels::naive_channel::Channel as NaiveChannel;
use chapter_5_channels::oneshot_channel::Channel as OneshotChannel;
use chapter_5_channels::send_recv_oneshot_channel::channel;
use chapter_5_channels::send_recv_oneshot_channel_noarc::Channel;

fn main() {
    use_naive_channel();
    use_one_shot_channel();
    use_sender_receiver();
    use_sender_receiver_split();
    use_blocking_channel();
}

fn use_blocking_channel() {
    // We’ve had to pay for this convenience by trading in some flexibility:
    // only the thread that calls split() may call receive()
    let mut channel = BloockingChannel::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        s.spawn(move || {
            sender.send("blocing channel");
        });
        println!("{}", receiver.receive());
    });
}

fn use_sender_receiver_split() {
    // created before scope of sender and receiver
    // to prove to compiler it outlives them
    let mut channel = Channel::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();
        let t = thread::current();
        s.spawn(move || {
            sender.send("sender and receiver no arc");
            t.unpark();
        });
        while !receiver.is_ready() {
            thread::park();
        }
        println!("{}", receiver.receive());
    });
}

fn use_sender_receiver() {
    thread::scope(|s| {
        let (sender, receiver) = channel();
        let t = thread::current();
        s.spawn(move || {
            sender.send("sender and receiver");
            t.unpark();
        });
        while !receiver.is_ready() {
            thread::park();
        }
        println!("{}", receiver.receive());
    });
}

fn use_one_shot_channel() {
    let channel = OneshotChannel::new();
    let t = thread::current();
    thread::scope(|s| {
        s.spawn(|| {
            channel.send("one shot channel");
            t.unpark();
        });
        while !channel.is_ready() {
            thread::park();
        }
        println!("{}", channel.receive());
    });
}

fn use_naive_channel() {
    let channel = Arc::new(NaiveChannel::new());
    let reader = channel.clone();
    thread::spawn(move || {
        for _ in 0..5 {
            channel.send("naive channel");
            thread::sleep(Duration::from_secs(1));
        }
    });
    for _ in 0..5 {
        println!("{}", reader.receive());
    }
}
