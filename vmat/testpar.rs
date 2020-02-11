
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::sync_channel;
use std::thread;

extern crate bus;
use bus::Bus;

struct  IndexSeq {
    n: usize,
    tbl: Vec<HashMap<usize, Vec<usize>>>,
}


impl IndexSeq {
    
    fn new(n: usize) -> Self {
        let mut tbl: Vec<HashMap<usize, Vec<usize>>> = vec![];
        for _k in 0..n {
            let t: HashMap<usize, Vec<usize>> = HashMap::new();
            tbl.push(t);
        }
        IndexSeq {
            n: n,
            tbl: tbl
        }
    }

    
    fn index(&mut self, len: usize) {
        for i in 0..len {
            println!("---- Processing position {}", i);
            for k in 0..self.n {
                let key = i % 100;
                if !self.tbl[k].contains_key(&key) {
                    self.tbl[k].insert(key, vec![]);
                }
                self.tbl[k].get_mut(&key).unwrap().push(i);
                println!("Add position {} to key {} in table #{}", i, key, k)
            }
        }
    }
}


struct  IndexPar {
    n: usize,
    tbl: Arc<Mutex<Vec<HashMap<usize, Vec<usize>>>>>,
}


impl IndexPar {
    
    fn new(n: usize) -> Self {
        let mut tbl: Vec<HashMap<usize, Vec<usize>>> = vec![];
        for _k in 0..n {
            let t: HashMap<usize, Vec<usize>> = HashMap::new();
            tbl.push(t);
        }
        IndexPar {
            n: n,
            tbl: Arc::new(Mutex::new(tbl))
        }
    }

    
    fn index(&mut self, len: usize) {
        for i in 0..len {
            println!("---- Processing position {}", i);
            let mut handles = vec![];
            for k in 0..self.n {
                let tblref = self.tbl.clone();
                handles.push( thread::spawn(move || {
                    let mut tbl = tblref.lock().unwrap();
                    let key = i % 100;
                    if !tbl[k].contains_key(&key) {
                        tbl[k].insert(key, vec![]);
                    }
                    tbl[k].get_mut(&key).unwrap().push(i);
                    println!("Add position {} to key {} in table #{}", i, key, k)
                }));
            }
            for h in handles{
                h.join().unwrap();
            }
        }
    }

}




struct IndexBus {
    n: usize
}


impl IndexBus {

    fn new(n:usize) -> Self {
        IndexBus{
            n
        }
    }


    fn index(&mut self, len: usize) {
        let mut bus:Bus<usize> = Bus::new(100);
        
        let mut receivers = vec![];
        for _k in 0..self.n {
            receivers.push(bus.add_rx());
        }
        
        let tx_handler = thread::spawn(move || {
            for i in 0..len {
                bus.broadcast(i);
            }
        });

        let mut rx_handler = vec![];
        for k in 0..self.n {
            let mut recvr = receivers.pop().unwrap();
            rx_handler.push(
                thread::spawn(move || {
                loop {
                    match recvr.recv() {
                        Ok(i) => {println!("receiver {} received {}",k,i)},
                        _ => break
                    }
                }
            }));
        }

        tx_handler.join().unwrap();
        for h in rx_handler {
            h.join().unwrap();
        }

    }

}


fn main() {
    println!("Hello, world!");
    let n = 5;
    let mut idx: IndexBus = IndexBus::new(10);
    let len = 100000;
    idx.index(len);
}
