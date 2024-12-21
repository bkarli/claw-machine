use core::task::{RawWaker, RawWakerVTable, Waker};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicU8;

static NUM_TASKS: AtomicU8 = AtomicU8::new(0);
static TASK_Q: heapless::mpmc::Q16<usize> = heapless::mpmc::Q16::new();
static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

pub trait ExtWaker {
    fn task(&self) -> usize;
}

impl ExtWaker for Waker {
    fn task(&self) -> usize {
        for task_id in 0..NUM_TASKS.load(Ordering::Relaxed) {
            if get_waker(task_id as usize).will_wake(self) {
                return task_id as usize;
            }
        }
        panic!("Task not found!");
    }
}

fn get_waker(task: usize) -> Waker {
    unsafe {
        Waker::from_raw(RawWaker::new(task as *const(), &VTABLE))
    }
}

unsafe fn clone(ptr: *const()) -> RawWaker {
    RawWaker::new(ptr, &VTABLE)
}

unsafe fn drop(_ptr: *const ()) {}

unsafe fn wake(ptr: *const()) {
    wake_task(ptr as usize)
}

unsafe fn wake_by_ref(ptr: *const()) {
    wake_task(ptr as usize)
}

fn wake_task(task: usize) {
    if TASK_Q.enqueue(task).is_err() {
        panic!("Task queue full: {}", task);
    }
}

pub fn run_task(tasks: &mut [Pin<&mut dyn Future<Output = ()>>]) {
    NUM_TASKS.store(tasks.len() as u8, Ordering::Relaxed);
}
