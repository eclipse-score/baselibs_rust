// *******************************************************************************
// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

use crate::parameters::{SchedulerPolicy, ThreadParameters};
use core::marker::PhantomData;
use core::mem::MaybeUninit;

/// Expected [`libc::cpu_set_t`] size in bits.
const CPU_SET_SIZE: usize = 1024;

const _U8_SIZE: usize = u8::BITS as usize;
const _DATA_SIZE: usize = CPU_SET_SIZE / _U8_SIZE;

/// Wrapper for [`libc::cpu_set_t`].
struct CpuSet {
    data: [u8; CPU_SET_SIZE / _U8_SIZE],
}

impl CpuSet {
    /// Create a new [`CpuSet`] based on provided `affinity`.
    fn new(affinity: &[usize]) -> Self {
        let mut data = [0; CPU_SET_SIZE / _U8_SIZE];

        // Set affinity as bits.
        for cpu_id in affinity.iter().copied() {
            const MAX_ID: usize = CPU_SET_SIZE - 1;
            if cpu_id > MAX_ID {
                panic!("CPU ID provided to affinity exceeds max supported size, provided: {cpu_id}, max: {MAX_ID}");
            }

            let index = cpu_id / _U8_SIZE;
            let offset = cpu_id % _U8_SIZE;
            data[index] |= 1 << offset;
        }

        Self { data }
    }

    /// Get inner representation.
    fn get(self) -> libc::cpu_set_t {
        unsafe { core::mem::transmute(self.data) }
    }
}

/// `pthread` attributes object.
struct Attributes {
    attr_handle: libc::pthread_attr_t,
}

impl Attributes {
    /// Create `pthread` attributes object.
    fn new() -> Self {
        let mut attr = MaybeUninit::uninit();
        let rc = unsafe { libc::pthread_attr_init(attr.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_init failed, rc: {rc}, errno: {errno}");
        }

        let attr_handle = unsafe { attr.assume_init() };
        Self { attr_handle }
    }

    /// Pointer to mutable internal handle.
    fn ptr(&mut self) -> *mut libc::pthread_attr_t {
        &mut self.attr_handle as *mut _
    }

    /// Set inherit scheduling attributes.
    fn inherit_scheduling_attributes(&mut self, inherit: bool) {
        let inherit_native = if inherit {
            libc::PTHREAD_INHERIT_SCHED
        } else {
            libc::PTHREAD_EXPLICIT_SCHED
        };
        let rc = unsafe { libc::pthread_attr_setinheritsched(self.ptr(), inherit_native) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_setinheritsched failed, rc: {rc}, errno: {errno}");
        }
    }

    /// Set thread priority.
    fn priority(&mut self, priority: i32) {
        let params = libc::sched_param {
            sched_priority: priority,
        };
        let rc = unsafe { libc::pthread_attr_setschedparam(self.ptr(), &params as *const _) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_setschedparam failed, rc: {rc}, errno: {errno}");
        }
    }

    /// Set scheduler policy.
    fn scheduler_policy(&mut self, scheduler_policy: SchedulerPolicy) {
        let policy = scheduler_policy as i32;
        let rc = unsafe { libc::pthread_attr_setschedpolicy(self.ptr(), policy) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_setschedpolicy failed, rc: {rc}, errno: {errno}");
        }
    }

    /// Set thread affinity - array of CPU core IDs that the thread can run on.
    fn affinity(&mut self, affinity: &[usize]) {
        let cpu_set = CpuSet::new(affinity).get();
        let cpu_set_size = size_of::<libc::cpu_set_t>();

        let rc = unsafe { libc::pthread_attr_setaffinity_np(self.ptr(), cpu_set_size, &cpu_set as *const _) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_setaffinity_np failed, rc: {rc}, errno: {errno}");
        }
    }

    /// Set stack size.
    fn stack_size(&mut self, stack_size: usize) {
        let rc = unsafe { libc::pthread_attr_setstacksize(self.ptr(), stack_size) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_setstacksize failed, rc: {rc}, errno: {errno}");
        }
    }

    /// Get reference to inner handle.
    fn get(&self) -> &libc::pthread_attr_t {
        &self.attr_handle
    }
}

impl Drop for Attributes {
    fn drop(&mut self) {
        let rc = unsafe { libc::pthread_attr_destroy(self.ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_destroy failed, rc: {rc}, errno: {errno}");
        }
    }
}

struct ThreadData<T, F: FnOnce() -> T> {
    f: F,
}

/// `pthread` thread object.
struct Thread {
    thread_handle: libc::pthread_t,
}

impl Thread {
    fn new<T, F>(attributes: Attributes, f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let mut thread_handle = MaybeUninit::uninit();

        extern "C" fn start_routine<T, F: FnOnce() -> T>(data: *mut libc::c_void) -> *mut libc::c_void {
            let data = unsafe { core::ptr::read(data as *const ThreadData<T, F>) };
            let result = (data.f)();
            Box::into_raw(Box::new(result)).cast()
        }

        let data = Box::into_raw(Box::new(ThreadData { f }));
        let rc = unsafe {
            libc::pthread_create(
                thread_handle.as_mut_ptr(),
                attributes.get() as *const _,
                start_routine::<T, F>,
                data as *mut _,
            )
        };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_create failed, rc: {rc}, errno: {errno}");
        }

        Self {
            thread_handle: unsafe { thread_handle.assume_init() },
        }
    }
}

/// An owned permission to join on a thread (block on its termination).
pub struct JoinHandle<T> {
    thread: Thread,
    _marker: PhantomData<T>,
}

impl<T> JoinHandle<T> {
    fn new(thread: Thread) -> Self {
        Self {
            thread,
            _marker: PhantomData,
        }
    }

    /// Wait for the associated thread to finish.
    ///
    /// This function will return immediately if the associated thread has already finished.
    pub fn join(self) -> T {
        let mut result = MaybeUninit::<*mut libc::c_void>::uninit();
        let thread_handle = self.thread.thread_handle;
        let rc = unsafe { libc::pthread_join(thread_handle, result.as_mut_ptr().cast()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_join failed, rc: {rc}, errno: {errno}");
        }

        let result_box = unsafe { Box::from_raw(result.assume_init().cast()) };
        *result_box
    }
}

/// Spawn a new thread, returning [`JoinHandle`] for it.
pub fn spawn<F, T>(f: F, thread_parameters: ThreadParameters) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    // Construct attributes based on provided parameters.
    let mut attributes = Attributes::new();

    if let Some(scheduler_parameters) = thread_parameters.scheduler_parameters {
        attributes.inherit_scheduling_attributes(false);
        attributes.scheduler_policy(scheduler_parameters.policy());
        attributes.priority(scheduler_parameters.priority());
    }
    if let Some(affinity) = thread_parameters.affinity {
        attributes.affinity(&affinity);
    }
    if let Some(stack_size) = thread_parameters.stack_size {
        attributes.stack_size(stack_size);
    }

    // Create a `Thread` and place it in a `JoinHandle`.
    let thread = Thread::new(attributes, f);
    JoinHandle::new(thread)
}

#[cfg(test)]
mod tests {
    use crate::parameters::{SchedulerParameters, SchedulerPolicy, ThreadParameters};
    use crate::thread::{spawn, Attributes, CpuSet};
    use core::mem::MaybeUninit;
    use std::sync::mpsc::channel;

    #[test]
    fn cpu_set_new_empty() {
        let cpu_set = CpuSet::new(&[]);

        // Check all zeroed.
        assert!(cpu_set.data.iter().all(|x| *x == 0));
    }

    #[test]
    fn cpu_set_new_some() {
        let cpu_set = CpuSet::new(&[0, 123, 1023]);
        let mut data_vec = cpu_set.data.to_vec();
        // Test removes from `Vec`, start from the end.
        assert_eq!(data_vec.remove(127), 0x80);
        assert_eq!(data_vec.remove(15), 0x08);
        assert_eq!(data_vec.remove(0), 0x01);
    }

    #[test]
    fn cpu_set_new_all() {
        let all_ids: Vec<_> = (0..1024).collect();
        let cpu_set = CpuSet::new(&all_ids);

        // Check all maxed.
        assert!(cpu_set.data.iter().all(|x| *x == 0xFF));
    }

    #[test]
    #[should_panic(expected = "CPU ID provided to affinity exceeds max supported size, provided: 1024, max: 1023")]
    fn cpu_set_new_out_of_range() {
        let _ = CpuSet::new(&[0, 123, 1023, 1024]);
    }

    fn attr_inherit_scheduling_attributes(attrs: &Attributes) -> bool {
        let mut native = MaybeUninit::uninit();
        let rc = unsafe { libc::pthread_attr_getinheritsched(attrs.get(), native.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_getinheritsched failed, rc: {rc}, errno: {errno}");
        }

        match unsafe { native.assume_init() } {
            libc::PTHREAD_INHERIT_SCHED => true,
            libc::PTHREAD_EXPLICIT_SCHED => false,
            _ => panic!("unknown inherit scheduling attributes value"),
        }
    }

    fn attr_priority(attrs: &Attributes) -> i32 {
        let mut param_native = MaybeUninit::uninit();
        let rc = unsafe { libc::pthread_attr_getschedparam(attrs.get(), param_native.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_getschedparam failed, rc: {rc}, errno: {errno}");
        }

        unsafe { param_native.assume_init().sched_priority }
    }

    fn attr_policy(attrs: &Attributes) -> SchedulerPolicy {
        let mut policy_native = MaybeUninit::uninit();
        let rc = unsafe { libc::pthread_attr_getschedpolicy(attrs.get(), policy_native.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_getschedpolicy failed, rc: {rc}, errno: {errno}");
        }

        SchedulerPolicy::from(unsafe { policy_native.assume_init() })
    }

    fn attr_affinity(attrs: &Attributes) -> Vec<usize> {
        let mut cpu_set = MaybeUninit::uninit();
        let cpu_set_size = size_of::<libc::cpu_set_t>();
        let rc = unsafe { libc::pthread_attr_getaffinity_np(attrs.get(), cpu_set_size, cpu_set.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_getaffinity_np failed, rc: {rc}, errno: {errno}");
        }
        let cpu_set = unsafe { cpu_set.assume_init() };

        let mut affinity = Vec::new();
        for i in 0..libc::CPU_SETSIZE as usize {
            if unsafe { libc::CPU_ISSET(i, &cpu_set) } {
                affinity.push(i);
            }
        }

        affinity
    }

    fn attr_stack_size(attrs: &Attributes) -> usize {
        let mut stack_size = MaybeUninit::uninit();
        let rc = unsafe { libc::pthread_attr_getstacksize(attrs.get(), stack_size.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_attr_getstacksize failed, rc: {rc}, errno: {errno}");
        }

        unsafe { stack_size.assume_init() }
    }

    #[test]
    fn attributes_new_succeeds() {
        // Also checks `Drop` on exit.
        let _ = Attributes::new();
    }

    #[test]
    fn attributes_inherit_scheduling_attributes_succeeds() {
        let mut attrs = Attributes::new();

        attrs.inherit_scheduling_attributes(true);
        assert!(attr_inherit_scheduling_attributes(&attrs));

        attrs.inherit_scheduling_attributes(false);
        assert!(!attr_inherit_scheduling_attributes(&attrs));
    }

    #[test]
    fn attributes_priority_succeeds() {
        let mut attrs = Attributes::new();

        attrs.scheduler_policy(SchedulerPolicy::Fifo);
        attrs.priority(50);
        assert_eq!(attr_priority(&attrs), 50);
    }

    #[test]
    #[should_panic(expected = "libc::pthread_attr_setschedparam failed, rc: 22, errno: 0")]
    fn attributes_priority_wrong_scheduler_policy() {
        let mut attrs = Attributes::new();
        attrs.priority(50);
    }

    #[test]
    fn attributes_scheduler_policy_succeeds() {
        let mut attrs = Attributes::new();

        attrs.scheduler_policy(SchedulerPolicy::Fifo);
        assert_eq!(attr_policy(&attrs), SchedulerPolicy::Fifo);
    }

    #[test]
    fn attributes_affinity_succeeds() {
        let mut attrs = Attributes::new();

        let expected_affinity = vec![0, 123, 1023];
        attrs.affinity(&expected_affinity);
        assert_eq!(attr_affinity(&attrs), expected_affinity);
    }

    #[test]
    #[should_panic(expected = "CPU ID provided to affinity exceeds max supported size, provided: 1024, max: 1023")]
    fn attributes_affinity_out_of_range() {
        let mut attrs = Attributes::new();
        attrs.affinity(&[1024]);
    }

    #[test]
    fn attributes_stack_size_succeeds() {
        let mut attrs = Attributes::new();

        let expected_stack_size = 1024 * 1024;
        attrs.stack_size(expected_stack_size);
        assert_eq!(attr_stack_size(&attrs), expected_stack_size);
    }

    #[test]
    #[should_panic(expected = "libc::pthread_attr_setstacksize failed, rc: 22, errno: 0")]
    fn attributes_stack_size_too_small() {
        let mut attrs = Attributes::new();
        attrs.stack_size(4 * 1024);
    }

    #[test]
    fn spawn_succeeds() {
        let thread_parameters = ThreadParameters::default();
        let (tx, rx) = channel();
        let join_handle = spawn(
            move || {
                tx.send(654321).unwrap();
                123
            },
            thread_parameters,
        );
        let join_result = join_handle.join();

        assert_eq!(join_result, 123);
        assert_eq!(rx.recv().unwrap(), 654321)
    }

    fn current_sched_params() -> (SchedulerPolicy, i32) {
        let thread = unsafe { libc::pthread_self() };
        let mut policy = MaybeUninit::uninit();
        let mut param = MaybeUninit::uninit();
        let rc = unsafe { libc::pthread_getschedparam(thread, policy.as_mut_ptr(), param.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::pthread_getschedparam failed, rc: {rc}, errno: {errno}");
        }

        let policy_native = unsafe { policy.assume_init() };
        let scheduler_policy = match policy_native {
            0 => SchedulerPolicy::Other,
            1 => SchedulerPolicy::Fifo,
            2 => SchedulerPolicy::RoundRobin,
            _ => panic!("Unknown scheduler type"),
        };

        let priority = unsafe { param.assume_init().sched_priority };

        (scheduler_policy, priority)
    }

    #[test]
    #[ignore = "test requires cap_sys_nice=ep"]
    fn spawn_scheduler_params_succeeds() {
        let exp_scheduler_parameters = SchedulerParameters::new(SchedulerPolicy::Fifo, 10);
        let thread_parameters = ThreadParameters::new().scheduler_parameters(exp_scheduler_parameters);
        let (tx, rx) = channel();
        let join_handle = spawn(
            move || {
                let sched_params = current_sched_params();
                tx.send(sched_params).unwrap();
            },
            thread_parameters,
        );
        join_handle.join();

        let (scheduler_policy, priority) = rx.recv().unwrap();
        assert_eq!(scheduler_policy, exp_scheduler_parameters.policy());
        assert_eq!(priority, exp_scheduler_parameters.priority());
    }

    fn current_thread_affinity() -> Vec<usize> {
        let current_thread = 0;
        let mut cpu_set = MaybeUninit::uninit();
        let cpu_set_size = size_of::<libc::cpu_set_t>();
        let rc = unsafe { libc::sched_getaffinity(current_thread, cpu_set_size, cpu_set.as_mut_ptr()) };
        if rc != 0 {
            let errno = unsafe { *libc::__errno_location() };
            panic!("libc::sched_getaffinity failed, rc: {rc}, errno: {errno}");
        }
        let cpu_set = unsafe { cpu_set.assume_init() };

        let mut affinity = Vec::new();
        for i in 0..libc::CPU_SETSIZE as usize {
            if unsafe { libc::CPU_ISSET(i, &cpu_set) } {
                affinity.push(i);
            }
        }

        affinity
    }

    #[test]
    fn spawn_affinity_succeeds() {
        let exp_affinity = vec![1];
        let thread_parameters = ThreadParameters::new().affinity(&exp_affinity);
        let (tx, rx) = channel();
        let join_handle = spawn(
            move || {
                let affinity = current_thread_affinity();
                tx.send(affinity).unwrap();
            },
            thread_parameters,
        );
        join_handle.join();

        assert_eq!(rx.recv().unwrap(), exp_affinity);
    }

    #[test]
    #[should_panic(expected = "CPU ID provided to affinity exceeds max supported size, provided: 1234, max: 1023")]
    fn spawn_affinity_out_of_range() {
        let thread_parameters = ThreadParameters::new().affinity(&[1234]);
        let _ = spawn(|| {}, thread_parameters);
    }

    #[test]
    fn spawn_stack_size_succeeds() {
        // Check that nothing fails - cannot check stack size from within.
        let stack_size = 1024 * 1024;
        let thread_parameters = ThreadParameters::new().stack_size(stack_size);
        let join_handle = spawn(
            || {
                // Do nothing.
            },
            thread_parameters,
        );
        join_handle.join();
    }
}
