use byteorder::{BigEndian, ReadBytesExt};
use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_serializable::*;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::{Cursor, Read, Write};
use std::net::*;
use util::utils;
use std::iter::FromIterator;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct TcpRequestError { pub error: String }

impl Display for TcpRequestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "TCP Request Error: {}", self.error)
    }
}

impl TcpRequestError {
    pub fn of(error: String) -> TcpRequestError { TcpRequestError { error } }
    pub fn from(error: &str) -> TcpRequestError { TcpRequestError::of(String::from(error)) }
}

pub fn request<A, T, U>(address: A, request: Request<T>) -> Result<Response<U>, TcpRequestError>
    where A: ToSocketAddrs, T: ProtocolSerializable, Vec<u8>: ProtocolDeserializable<Response<U>> {
    let response =
        request.into_protocol_bytes().and_then(|bytes| {
            TcpStream::connect(address).and_then(|mut stream| {
                stream.write(bytes.as_slice()).and_then(|_| {
                    let mut result_size_buf: [u8; 4] = [0; 4];
                    stream.read(&mut result_size_buf).and_then(|_| {
                        Cursor::new(result_size_buf.to_vec()).read_i32::<BigEndian>()
                    }).and_then(|result_size| {
                        let mut message_buf: Vec<u8> = vec![0; result_size as usize];
                        stream.read_exact(&mut message_buf).map(|_| message_buf)
                    })
                })
            })
        });

    response
        .map_err(|e| TcpRequestError::of(format!("{}", e.description())))
        .and_then(|bytes| {
//            println!("bytes: {:?}", utils::to_hex_array(&bytes));
            bytes.into_protocol_type().map_err(|e| TcpRequestError::of(e.error))
        })
}

pub fn multi_endpoint_request<A, T, U>(addresses: Vec<A>, request: Request<T>) -> HashMap<A, Result<Response<U>, TcpRequestError>>
    where A: ToSocketAddrs + Eq + Hash + Clone, T: ProtocolSerializable, Vec<u8>: ProtocolDeserializable<Response<U>> {
    addresses.into_iter().map(|address| {
        (address.clone(), self::request(address, request.clone()))
    }).collect::<HashMap<A, Result<Response<U>, TcpRequestError>>>()
}

//enum LazyResult<'a, T: 'a, E: 'a> {
//    Lazy(Box<FnBox() -> Result<T, E>>),
//    Ok(T),
//    Err(E),
//}
//
//impl<'a, T: 'a, E, TS: FromIterator<T>> FromIterator<LazyResult<'a, T, E>> for LazyResult<'a, TS, E> {
//    fn from_iter<I: IntoIterator<Item=LazyResult<'a, T, E>>>(iter: I) -> LazyResult<'a, TS, E> {
//        struct LazyIter<I, E> {
//            iter: I,
//            err: Option<E>,
//        }
//
//        impl<'a, T: 'a, I: Iterator<Item=LazyResult<'a, T, E>>, E: 'a> Iterator for LazyIter<I, E> {
//            type Item = T;
//
//            fn next(&mut self) -> Option<T> {
//                match self.iter.next() {
//                    None => None,
//                    Some(LazyResult::Lazy(ref fun)) => {
//                        let f: Result<T, E> = fun();
//                        match f {
//                            Ok(t) => Some(t),
//                            Err(e) => {
//                                self.err = Some(e);
//                                None
//                            }
//                        }
//                    }
//                    Some(LazyResult::Ok(t)) => Some(t),
//                    Some(LazyResult::Err(e)) => {
//                        self.err = Some(e);
//                        None
//                    }
//                }
//            }
//        }
//
//        let mut lazy_iter = LazyIter { iter: iter.into_iter(), err: None };
//        let ts: TS = FromIterator::from_iter(lazy_iter.by_ref());
//
//        match lazy_iter.err {
//            Some(err) => LazyResult::Err(err),
//            None => LazyResult::Ok(ts)
//        }
//    }
//}