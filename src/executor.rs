use avr_device::asm::sleep;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering;
use core::task::{Context, RawWaker, RawWakerVTable, Waker};
static NUM_TASKS: AtomicU8 = AtomicU8::new(0);
static TASK_Q: heapless::mpmc::Q16<usize> = heapless::mpmc::Q16::new();
static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

pub trait ExtWaker {
    fn task_id(&self) -> usize;
}

impl ExtWaker for Waker {
    fn task_id(&self) -> usize {
        for task_id in 0..NUM_TASKS.load(Ordering::Relaxed) {
            if get_waker(task_id as usize).will_wake(self) {
                return task_id as usize;
            }
        }
        panic!("Task not found!");
    }
}

fn get_waker(task: usize) -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(task as *const (), &VTABLE)) }
}

unsafe fn clone(ptr: *const ()) -> RawWaker {
    RawWaker::new(ptr, &VTABLE)
}

unsafe fn drop(_ptr: *const ()) {}

unsafe fn wake(ptr: *const ()) {
    wake_task(ptr as usize)
}

unsafe fn wake_by_ref(ptr: *const ()) {
    wake_task(ptr as usize)
}

/**
If wake task is called if invalid INVALID_TASK_ID loop will break
otherwise the task registered with that index will make progress
*/
pub fn wake_task(task: usize) {
    if TASK_Q.enqueue(task).is_err() {
        panic!("Task queue full: {}", task);
    }
}

/// tasks marked with this task id are break tasks
/// the loop will break and the game will advance to the next state
const INVALID_TASK_ID: usize = 0xFFFF;

/**
this function will run all registered tasks
once the loop breaks the game advances to the next state
*/
pub fn run_task(tasks: &mut [Pin<&mut dyn Future<Output = ()>>]) -> ! {
    NUM_TASKS.store(tasks.len() as u8, Ordering::Relaxed);
    for task in 0..tasks.len() {
        TASK_Q.enqueue(task).ok();
    }
    loop {
        // while there is a task in queue
        while let Some(task) = TASK_Q.dequeue() {
            // check if the task is a breaker task and exit loop

            // get task from array and make progress at that task
            let _ = tasks[task]
                .as_mut()
                .poll(&mut Context::from_waker(&get_waker(task)));
        }
        // else sleep
        sleep();
    }
}
