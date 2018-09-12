extern crate lib;
use lib::RwLock;
use lib::Preference;
use lib::Order;
use std::{thread, time};
use std::sync::Arc;

fn main() {
    let rwlock = Arc::new(RwLock::new(5, Preference::Reader, Order::Lifo));
    {
    	let mut lock = rwlock.clone();
    	let handler0 = thread::spawn(move || {
			let mut r1 = lock.write().unwrap();
			*r1 = 8;
			println!("8");
			assert_eq!(*r1, 8);
            let one_s = time::Duration::from_millis(1000);
            thread::sleep(one_s);
		});	
		lock = rwlock.clone();
		let handler1 = thread::spawn(move || {
			let r1 = lock.read().unwrap();
			assert_eq!(*r1, 8);     
		});	
		let mut one_s = time::Duration::from_millis(50);
        thread::sleep(one_s);
		lock = rwlock.clone();
		let handler2 = thread::spawn(move || {
			let mut r1 = lock.write().unwrap();
			*r1 = 7;
			println!("7");
			assert_eq!(*r1, 7);
            let one_s = time::Duration::from_millis(1000);
            thread::sleep(one_s);
		});
        thread::sleep(one_s);
		lock = rwlock.clone();
		let handler2 = thread::spawn(move || {
			let mut r1 = lock.write().unwrap();
			*r1 = 6;
			println!("6");
			assert_eq!(*r1, 6);
            let one_s = time::Duration::from_millis(1000);
            thread::sleep(one_s);
		});
		
		handler0.join().unwrap();
        handler1.join().unwrap();
        handler2.join().unwrap();
     //    handler3.join().unwrap();
    	// handler4.join().unwrap();
    	// handler5.join().unwrap();
    	// handler6.join().unwrap();
    	// handler7.join().unwrap();

    }
        
}
