type Thunk<A> = Box<Fn() -> A>;

pub struct IO<A> {
    thunk: Thunk<A>
}

impl<'a, A: 'a + 'static> IO<A> {
    pub fn new(f: Box<Fn() -> A>) -> IO<A> {
        IO {
            thunk: f
        }
    }

    pub fn unsafe_run(self) -> A {
        (self.thunk)()
    }

    pub fn map<'b: 'a + 'static, B: 'b>(self, f: Box<Fn(A) -> B>) -> IO<B> {
        IO {
            thunk: Box::new(move || f((self.thunk)()))
        }
    }

    pub fn and_then<'b: 'a + 'static, B: 'b>(self, f: Box<Fn(A) -> IO<B>>) -> IO<B> {
        IO {
            thunk: Box::new(move || {
                let b: IO<B> = f((self.thunk)());
                b.unsafe_run()
            })
        }
    }
}

pub fn foo() -> Result<String, i8> {
    let external_io = Box::new(|| Ok(String::from("coo coo coo")));
    let io: IO<i32> = IO::new(Box::new(|| 42));

    let io2: IO<Result<String, i8>> =
        io
            .map(Box::new(|i: i32| i * 2))
            .and_then(Box::new(move |i: i32| IO::new(external_io.clone())));

    io2.unsafe_run()
}