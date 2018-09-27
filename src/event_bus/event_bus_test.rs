use event_bus;
use event_bus::*;

#[test]
fn foo() {

    match event_bus::to_event(Message::Quit) {
        Event::Exiting => (),
        _ => panic!()
    }
//    assert_eq!(event_bus::to_event(Message::Quit), Event::Exiting);
}