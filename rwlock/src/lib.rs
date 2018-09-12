use std::cell::UnsafeCell;
use std::sync::{Mutex, Condvar};
use std::ops::{Deref,DerefMut};
use std::collections::VecDeque;
pub enum Preference {
    Reader,
    Writer,
}
pub enum Order {
    Fifo,
    Lifo,
}
pub struct RwLock<T> {
	data: UnsafeCell<T>,
	preference : Preference,
	ord : Order,
	reader_go : Condvar,
	writer_go : Condvar,
	lock: Mutex<ReaderWriter>, 	
}
pub struct ReaderWriter {
	reader: Operation,
	writer: Operation,
}
pub struct Operation {
	active : u32,
	waiting : u32,
	order_list: VecDeque<i32>,
	tot : i32,
	cur : i32,
}
impl Operation {
	pub fn new() -> Operation {
	    Operation {
    		active: 0,
	    	waiting: 0,
	    	order_list : VecDeque::new(),
	    	tot : 0, 
	    	cur : 0,
	    }
	}
}
impl ReaderWriter {
	pub fn new()-> ReaderWriter {
		ReaderWriter {
			reader : Operation::new(),
			writer : Operation::new(),
		}
	}
}
impl<T> RwLock<T> {
    pub fn new(data: T, pref: Preference, order: Order) -> RwLock<T> {
    	RwLock {
	    	data : UnsafeCell::new(data),
	    	preference : pref,
	    	ord : order,
	    	reader_go : Condvar::new(),
	    	writer_go : Condvar::new(),
	    	lock : Mutex::new(ReaderWriter::new()),
	    }
    }
    pub fn read(&self) -> Result<RwLockReadGuard<T>, ()> {

    	//println!("read start");
        let mut guard = self.lock.lock().unwrap();
        //println!("read: {} {} {} {}", guard.reader.active, guard.reader.waiting, guard.writer.active, guard.writer.waiting);
        guard.reader.waiting += 1;
        //println!("read waiting +1");
        match self.preference {
        	Preference::Reader => {
        		while guard.writer.active > 0 {
        			guard = self.reader_go.wait(guard).unwrap();
        		}	
        	},
        	Preference::Writer => {
        		while guard.writer.active > 0 || guard.writer.waiting > 0 {
        	 		guard = self.reader_go.wait(guard).unwrap();
        		}	
        	}
        }
        guard.reader.active += 1;
        guard.reader.waiting -= 1;
        Ok(RwLockReadGuard{r_guard : self,})
    }
    pub fn done_read(&self) {
    	let mut guard = self.lock.lock().unwrap();
       	guard.reader.active -= 1;
    	match self.preference {
    		Preference::Reader => {
    			if guard.reader.active == 0 && guard.writer.waiting > 0 {
    				self.writer_go.notify_all();
    			}
    		},
    		Preference::Writer => {
    			if guard.writer.waiting > 0 {
    				self.writer_go.notify_all();
    			}
    		},
    	}	
    }
    pub fn write(&self) -> Result<RwLockWriteGuard<T>, ()> {
 		let mut guard = self.lock.lock().unwrap();
 		guard.writer.waiting += 1;
 		let id = guard.writer.tot;
 		guard.writer.tot += 1;
 		guard.writer.order_list.push_back(id);
 		let is_lifo = match self.ord {
 			Order::Lifo =>{ 
 				guard.writer.cur = match guard.writer.order_list.back() {
 					Some(x) => *x,
 					None => -1,
 				};
 				true
 			},
 			Order::Fifo => {
 				guard.writer.cur = match guard.writer.order_list.get(0){
 					Some(x) => *x,
 					None => -1,
 				};
 				false
 			},
 		};
 		match self.preference {
 		   	Preference::Reader => {
 		   		while (guard.reader.active > 0 || guard.reader.waiting > 0 || guard.writer.active > 0)
 		   				|| (!(is_lifo && id == guard.writer.cur) && !(!is_lifo && id == guard.writer.cur)) {
 		   				guard = self.writer_go.wait(guard).unwrap();
 		   		}
 		   	},
 		   	Preference::Writer => {
  		   		while (guard.reader.active > 0 || guard.writer.active > 0) 
 		   			|| (!(is_lifo && id == guard.writer.cur) && !(!is_lifo &&id == guard.writer.cur)) {
	 		   			guard = self.writer_go.wait(guard).unwrap();
 		   		}
 		   	},
 		}
 		guard.writer.active += 1;
 		guard.writer.waiting -= 1;
 		match self.ord {
 			Order::Lifo => {
 				guard.writer.order_list.pop_back();
 				guard.writer.cur = match guard.writer.order_list.back() {
 					Some(x) => *x ,
 					None => -1,
 				};
 			},
 			Order::Fifo => {
 				guard.writer.order_list.pop_front();
 				guard.writer.cur = match guard.writer.order_list.get(0){
 					Some(x) => *x,
 					None => -1,
 				};
 			},
 		}
 		Ok(RwLockWriteGuard{w_guard: self,})
    }
    pub fn done_write(&self) {
    	let mut guard = self.lock.lock().unwrap();
    	guard.writer.active -= 1;
    	match self.preference {
    		Preference::Reader => {
    			if guard.reader.waiting > 0 {
    				self.reader_go.notify_all();
    			}
    			else {
    				self.writer_go.notify_all();
    			}
    		},
    		Preference::Writer => {
    			if guard.writer.waiting > 0 {
    				self.writer_go.notify_all();
    			}
    			else {
    				self.reader_go.notify_all();
    			}
    		} ,
    	}
    	
    }
}
unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}
pub struct RwLockReadGuard<'a, T: 'a> {
    r_guard : &'a RwLock<T>,
}
impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe{ &*self.r_guard.data.get()}
    }
}
impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        self.r_guard.done_read();
    }
}
pub struct RwLockWriteGuard<'a, T: 'a> {
    w_guard : &'a RwLock<T>,
}
impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe{& *self.w_guard.data.get()}
    }
}
impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe{ &mut *self.w_guard.data.get()}
    }
}
impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.w_guard.done_write();
    }
}

