#![feature(async_await)]

extern crate futures;
extern crate crossbeam_channel;
extern crate twox_hash;
extern crate dashmap;
extern crate r#async;
extern crate rand;

use std::thread;
use std::rc::{Weak, Rc};
use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::cell::{UnsafeCell, RefCell};
use std::task::{Context, Poll, Waker};

use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}};
use crossbeam_channel::Sender;
use twox_hash::RandomXxHashBuilder64;
use dashmap::DashMap;
use rand::prelude::*;

use r#async::{AsyncTask, AsyncExecutorResult, AsyncExecutor, AsyncSpawner, TaskId, AsyncRuntime, AsyncValue,
              single_thread::{SingleTask, SingleTaskRuntime, SingleTaskRunner},
              multi_thread::{MultiTask, MultiTaskRuntime, MultiTaskPool},
              local_queue::{LocalQueueSpawner, LocalQueue}, task::LocalTask};

struct TestFuture(usize, Weak<RefCell<HashMap<usize, Waker>>>);

unsafe impl Send for TestFuture {}
unsafe impl Sync for TestFuture {}

impl Future for TestFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = self.as_ref().0;
        if index % 2 == 0 {
            match self.as_ref().1.upgrade() {
                None => {
                    println!("!!!> future poll failed, index: {}", index);
                },
                Some(handle) => {
                    self.as_mut().0 += 1;
                    handle.borrow_mut().insert(index, cx.waker().clone());
                },
            }
            Poll::Pending
        } else {
            Poll::Ready("future ready".to_string())
        }
    }
}

impl TestFuture {
    pub fn new(handle: Rc<RefCell<HashMap<usize, Waker>>>, index: usize) -> Self {
        TestFuture(index, Rc::downgrade(&handle))
    }
}

#[test]
fn test_async_task() {
    let handle = Rc::new(RefCell::new(HashMap::new()));
    let mut queue = LocalQueue::with_capacity(10);
    let spawner = Rc::new(queue.get_spawner());

    for index in 0..100 {
        let future = TestFuture::new(handle.clone(), index); //handle是Rc，不允许跨线程，需要在外部用TestFuture封装后再move到async block中，否则handle将无法move到async block中
        if let Err(e) = spawner.spawn(LocalTask::new(spawner.clone(), async move {
            println!("!!!!!!async task start, index: {}", index);
            let r = future.await;
            println!("!!!!!!async task finish, index: {}, r: {:?}", index, r);
        })) {
            println!("!!!> spawn task failed, index: {}, reason: {:?}", index, e);
        }

        queue.run_once();
    }

    let keys = &mut handle.borrow().keys().map(|key| {
        key.clone()
    }).collect::<Vec<usize>>()[..];
    keys.sort();
    keys.reverse();
    for key in keys {
        //唤醒中止任务
        if let Some(waker) = handle.borrow_mut().remove(key) {
            waker.wake();
        }

        queue.run_once();
    }
}

#[test]
fn test_dashmap() {
    let map: Arc<DashMap<usize, usize, RandomXxHashBuilder64>> = Arc::new(DashMap::with_hasher(Default::default()));

    let map0 = map.clone();
    let handle0 = thread::spawn(move || {
        let start = Instant::now();
        for key in 0..1000000 {
            map0.insert(key, key);
        }
        println!("!!!!!!handle0, insert time: {:?}", Instant::now() - start);
    });

    let map1 = map.clone();
    let handle1 = thread::spawn(move || {
        let start = Instant::now();
        for key in 1000000..2000000 {
            map1.insert(key, key);
        }
        println!("!!!!!!handle1, insert time: {:?}", Instant::now() - start);
    });

    let map3 = map.clone();
    let handle3 = thread::spawn(move || {
        let start = Instant::now();
        for key in 0..2000000 {
            map3.get(&key);
        }
        println!("!!!!!!handle3, get time: {:?}", Instant::now() - start);
    });

    handle0.join();
    handle1.join();
    handle3.join();

    let mut total = 0;
    let start = Instant::now();
    for key in 0..map.len() {
        map.get(&key);
        total += key;
    }
    println!("!!!!!!finish, total: {:?}, get time: {:?}", total, Instant::now() - start);
}

#[derive(Clone)]
struct SyncUsize(Arc<RefCell<usize>>);

unsafe impl Send for SyncUsize {}
unsafe impl Sync for SyncUsize {}

struct TestFuture0(SyncUsize, TaskId, SingleTaskRuntime<()>);

unsafe impl Send for TestFuture0 {}
unsafe impl Sync for TestFuture0 {}

impl Future for TestFuture0 {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = *(self.as_ref().0).0.borrow();
        if index % 2 == 0 {
            self.2.pending(self.1.clone(), cx.waker().clone())
        } else {
            Poll::Ready("future ready".to_string())
        }
    }
}

impl TestFuture0 {
    pub fn new(rt: SingleTaskRuntime<()>, index: SyncUsize, uid: TaskId) -> Self {
        TestFuture0(index, uid, rt)
    }
}

#[test]
fn test_single_task() {
    let mut runner = SingleTaskRunner::new();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run_once() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let mut ids = Vec::with_capacity(50);
    for index in 0..100 {
        let uid = rt.alloc();
        let value = SyncUsize(Arc::new(RefCell::new(index)));
        let future = TestFuture0::new(rt.clone(), value.clone(), uid);
        if let Err(e) = rt.spawn(uid, async move {
            println!("!!!!!!async task start, uid: {:?}", uid);
            let r = future.await;
            println!("!!!!!!async task finish, uid: {:?}, r: {:?}", uid, r);
        }) {
            println!("!!!> spawn task failed, uid: {:?}, reason: {:?}", uid, e);
        }
        ids.push((uid, value));
    }

    thread::sleep(Duration::from_millis(3000));

    for (id, value) in ids {
        let uid = rt.alloc();
        let rt_copy = rt.clone();
        if let Err(e) = rt.spawn(uid, async move {
            //修改值，并继续中止的任务
            *value.0.borrow_mut() += 1;
            rt_copy.wakeup(id);
        }) {
            println!("!!!> spawn waker failed, id: {:?}, uid: {:?}, reason: {:?}", id, uid, e);
        }
    }

    thread::sleep(Duration::from_millis(100000000));
}

struct TestFuture1(SyncUsize, TaskId, MultiTaskRuntime<()>);

unsafe impl Send for TestFuture1 {}
unsafe impl Sync for TestFuture1 {}

impl Future for TestFuture1 {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let index = *(self.as_ref().0).0.borrow();
        if index % 2 == 0 {
            self.2.pending(self.1.clone(), cx.waker().clone())
        } else {
            Poll::Ready("future ready".to_string())
        }
    }
}

impl TestFuture1 {
    pub fn new(rt: MultiTaskRuntime<()>, index: SyncUsize, uid: TaskId) -> Self {
        TestFuture1(index, uid, rt)
    }
}

#[test]
fn test_multil_task() {
    let pool = MultiTaskPool::new("AsyncWorker".to_string(), 8, 1024 * 1024, 10);
    let rt = pool.startup();

    let mut ids = Vec::with_capacity(50);
    for index in 0..100 {
        let uid = rt.alloc();
        let value = SyncUsize(Arc::new(RefCell::new(index)));
        let future = TestFuture1::new(rt.clone(), value.clone(), uid);
        if let Err(e) = rt.spawn(uid, async move {
            println!("!!!!!!async task start, uid: {:?}", uid);
            let r = future.await;
            println!("!!!!!!async task finish, uid: {:?}, r: {:?}", uid, r);
        }) {
            println!("!!!> spawn task failed, uid: {:?}, reason: {:?}", uid, e);
        }
        ids.push((uid, value));
    }

    thread::sleep(Duration::from_millis(3000));

    for (id, value) in ids {
        let uid = rt.alloc();
        let rt_copy = rt.clone();
        if let Err(e) = rt.spawn(uid, async move {
            //修改值，并继续中止的任务
            *value.0.borrow_mut() += 1;
            rt_copy.wakeup(id);
        }) {
            println!("!!!> spawn waker failed, id: {:?}, uid: {:?}, reason: {:?}", id, uid, e);
        }
    }

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_async_value() {
    let mut runner = SingleTaskRunner::new();
    let rt0 = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run_once() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskPool::<()>::new("AsyncRuntime0".to_string(), 2, 1024 * 1024, 10);
    let rt1 = pool.startup();

    let rt0_copy = rt0.clone();
    let future = async move {
        let value = AsyncValue::new(AsyncRuntime::Single(rt0_copy));
        let value_copy = value.clone();

        rt1.spawn(rt1.alloc(), async move {
            value_copy.set(true);
        });

        println!("!!!!!!async value: {:?}", value.await);
    };
    rt0.spawn(rt0.alloc(), future);

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_async_wait() {
    let mut runner = SingleTaskRunner::new();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run_once() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskPool::<()>::new("AsyncRuntime0".to_string(), 2, 1024 * 1024, 10);
    let rt0 = pool.startup();

    let pool = MultiTaskPool::<()>::new("AsyncRuntime1".to_string(), 2, 1024 * 1024, 10);
    let rt1 = pool.startup();

    let rt_copy = rt.clone();
    let future = async move {
        let rt0_copy = rt0.clone();
        let r = rt_copy.clone().wait(AsyncRuntime::Multi(rt0), async move {
            let rt1_copy = rt1.clone();
            rt0_copy.wait(AsyncRuntime::Multi(rt1), async move {
                rt1_copy.wait(AsyncRuntime::Single(rt_copy), async move {
                    Ok(true)
                }).await
            }).await
        }).await;

        match r {
            Err(e) => {
                println!("!!!!!!wait failed, reason: {:?}", e);
            },
            Ok(result) => {
                println!("!!!!!!wait ok, result: {:?}", result);
            },
        }
    };
    rt.spawn(rt.alloc(), future);

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_async_wait_any() {
    let mut runner = SingleTaskRunner::new();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run_once() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskPool::<()>::new("AsyncRuntime0".to_string(), 2, 1024 * 1024, 10);
    let rt0 = pool.startup();

    let pool = MultiTaskPool::<()>::new("AsyncRuntime1".to_string(), 2, 1024 * 1024, 10);
    let rt1 = pool.startup();

    let rt_copy = rt.clone();
    let future = async move {
        let f0 = Box::new(async move {
            let mut rng = rand::thread_rng();
            let timeout: u64 = rng.gen_range(0, 10000);
            thread::sleep(Duration::from_millis(timeout));
            Ok("rt0-".to_string() + timeout.to_string().as_str())
        }).boxed();
        let f1 = Box::new(async move {
            let mut rng = rand::thread_rng();
            let timeout: u64 = rng.gen_range(0, 10000);
            thread::sleep(Duration::from_millis(timeout));
            Ok("rt1-".to_string() + timeout.to_string().as_str())
        }).boxed();

        match rt_copy.wait_any(vec![(AsyncRuntime::Multi(rt0), f0), (AsyncRuntime::Multi(rt1), f1)]).await {
            Err(e) => {
                println!("!!!!!!wait any failed, reason: {:?}", e);
            },
            Ok(result) => {
                println!("!!!!!!wait any ok, result: {:?}", result);
            },
        }
    };
    rt.spawn(rt.alloc(), future);

    thread::sleep(Duration::from_millis(100000000));
}

#[test]
fn test_async_wait_all() {
    let mut runner = SingleTaskRunner::new();
    let rt = runner.startup().unwrap();

    thread::spawn(move || {
        loop {
            if let Err(e) = runner.run_once() {
                println!("!!!!!!run failed, reason: {:?}", e);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    let pool = MultiTaskPool::<()>::new("AsyncRuntime0".to_string(), 2, 1024 * 1024, 10);
    let rt0 = pool.startup();

    let pool = MultiTaskPool::<()>::new("AsyncRuntime1".to_string(), 2, 1024 * 1024, 10);
    let rt1 = pool.startup();

    let rt_copy = rt.clone();
    let future = async move {
        let mut map = rt_copy.map();
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(0)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(1)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(2)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(3)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(4)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(5)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(6)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(7)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(8)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(9)
        });

        println!("!!!!!!map result: {:?}", map.map(AsyncRuntime::Single(rt_copy.clone()), false).await);

        let mut map = rt_copy.map();
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(0)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(1)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(2)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(3)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(4)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(5)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(6)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(7)
        });
        map.join(AsyncRuntime::Multi(rt0.clone()), async move {
            Ok(8)
        });
        map.join(AsyncRuntime::Multi(rt1.clone()), async move {
            Ok(9)
        });

        println!("!!!!!!map result by order: {:?}", map.map(AsyncRuntime::Single(rt_copy), true).await);
    };
    rt.spawn(rt.alloc(), future);

    thread::sleep(Duration::from_millis(100000000));
}