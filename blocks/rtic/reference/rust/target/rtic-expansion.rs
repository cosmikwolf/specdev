#[doc = r" The RTIC application module"] pub mod app
{
    #[doc =
    r" Always include the device crate which contains the vector table"] use
    lm3s6965 as you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml;
    #[doc =
    r" Holds the maximum priority level for use by async HAL drivers."]
    #[no_mangle] static RTIC_ASYNC_MAX_LOGICAL_PRIO : u8 = 2u8; use rtic_sync
    :: { channel :: { Receiver, Sender }, make_channel };
    #[doc = r" User code end"] #[doc = r"Shared resources"] struct Shared
    { counter : u32, } #[doc = r"Local resources"] struct Local
    {
        toggle : bool, tx : Sender < 'static, u32, 4 > , rx : Receiver <
        'static, u32, 4 > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_init_Context <
    'a >
    {
        #[doc(hidden)] __rtic_internal_p : :: core :: marker :: PhantomData <
        & 'a () > ,
        #[doc = r" The space used to allocate async executors in bytes."] pub
        executors_size : usize, #[doc = r" Core peripherals"] pub core : rtic
        :: export :: Peripherals, #[doc = r" Device peripherals (PAC)"] pub
        device : lm3s6965 :: Peripherals,
        #[doc = r" Critical section token for init"] pub cs : rtic :: export
        :: CriticalSection < 'a > ,
    } impl < 'a > __rtic_internal_init_Context < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn
        new(core : rtic :: export :: Peripherals, executors_size : usize) ->
        Self
        {
            __rtic_internal_init_Context
            {
                __rtic_internal_p : :: core :: marker :: PhantomData, core :
                core, device : lm3s6965 :: Peripherals :: steal(), cs : rtic
                :: export :: CriticalSection :: new(), executors_size,
            }
        }
    } #[allow(non_snake_case)] #[doc = "Initialization function"] pub mod init
    {
        #[doc(inline)] pub use super :: __rtic_internal_init_Context as
        Context;
    } #[inline(always)] #[allow(non_snake_case)] fn
    init(_cx : init :: Context) -> (Shared, Local)
    {
        let (tx, rx) = make_channel! (u32, 4); drain :: spawn().ok();
        (Shared { counter : 0 }, Local { toggle : false, tx, rx })
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_idle_Context <
    'a >
    {
        #[doc(hidden)] __rtic_internal_p : :: core :: marker :: PhantomData <
        & 'a () > ,
    } impl < 'a > __rtic_internal_idle_Context < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_idle_Context
            { __rtic_internal_p : :: core :: marker :: PhantomData, }
        }
    } #[allow(non_snake_case)] #[doc = "Idle loop"] pub mod idle
    {
        #[doc(inline)] pub use super :: __rtic_internal_idle_Context as
        Context;
    }
    #[doc =
    " `kind = \"idle\"`. Runs when nothing else is runnable, never returns,"]
    #[doc = " and takes no priority — it is always 0."]
    #[allow(non_snake_case)] fn idle(_cx : idle :: Context) -> !
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ; loop
        { cortex_m :: asm :: wfi(); }
    } #[allow(non_snake_case)] #[no_mangle] unsafe fn GPIOA()
    {
        const PRIORITY : u8 = 3u8; rtic :: export ::
        run(PRIORITY, || { sample(sample :: Context :: new()) });
    } impl < 'a > __rtic_internal_sampleLocalResources < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_sampleLocalResources
            {
                toggle : & mut *
                (& mut *
                __rtic_internal_local_resource_toggle.get_mut()).as_mut_ptr(),
                __rtic_internal_marker : :: core :: marker :: PhantomData,
            }
        }
    } impl < 'a > __rtic_internal_sampleSharedResources < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_sampleSharedResources
            {
                counter : shared_resources :: counter_that_needs_to_be_locked
                :: new(), __rtic_internal_marker : core :: marker ::
                PhantomData,
            }
        }
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = "Local resources `sample` has access to"] pub struct
    __rtic_internal_sampleLocalResources < 'a >
    {
        #[allow(missing_docs)] pub toggle : & 'a mut bool, #[doc(hidden)] pub
        __rtic_internal_marker : :: core :: marker :: PhantomData < & 'a () >
        ,
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = "Shared resources `sample` has access to"] pub struct
    __rtic_internal_sampleSharedResources < 'a >
    {
        #[allow(missing_docs)] pub counter : shared_resources ::
        counter_that_needs_to_be_locked < 'a > , #[doc(hidden)] pub
        __rtic_internal_marker : core :: marker :: PhantomData < & 'a () > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_sample_Context <
    'a >
    {
        #[doc(hidden)] __rtic_internal_p : :: core :: marker :: PhantomData <
        & 'a () > , #[doc = r" Local Resources this task has access to"] pub
        local : sample :: LocalResources < 'a > ,
        #[doc = r" Shared Resources this task has access to"] pub shared :
        sample :: SharedResources < 'a > ,
    } impl < 'a > __rtic_internal_sample_Context < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_sample_Context
            {
                __rtic_internal_p : :: core :: marker :: PhantomData, local :
                sample :: LocalResources :: new(), shared : sample ::
                SharedResources :: new(),
            }
        }
    } #[allow(non_snake_case)] #[doc = "Hardware task"] pub mod sample
    {
        #[doc(inline)] pub use super :: __rtic_internal_sampleLocalResources
        as LocalResources; #[doc(inline)] pub use super ::
        __rtic_internal_sampleSharedResources as SharedResources;
        #[doc(inline)] pub use super :: __rtic_internal_sample_Context as
        Context;
    } #[allow(non_snake_case)] fn sample(mut cx : sample :: Context)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ;
        cx.shared.counter.lock(| counter : & mut u32 | * counter += 1); *
        cx.local.toggle = ! * cx.local.toggle;
    } impl < 'a > __rtic_internal_drainSharedResources < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_drainSharedResources
            {
                counter : shared_resources :: counter_that_needs_to_be_locked
                :: new(), __rtic_internal_marker : core :: marker ::
                PhantomData,
            }
        }
    } impl < 'a > __rtic_internal_pumpLocalResources < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_pumpLocalResources
            {
                tx : & mut *
                (& mut *
                __rtic_internal_local_resource_tx.get_mut()).as_mut_ptr(), rx
                : & mut *
                (& mut *
                __rtic_internal_local_resource_rx.get_mut()).as_mut_ptr(),
                __rtic_internal_marker : :: core :: marker :: PhantomData,
            }
        }
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = "Shared resources `drain` has access to"] pub struct
    __rtic_internal_drainSharedResources < 'a >
    {
        #[allow(missing_docs)] pub counter : shared_resources ::
        counter_that_needs_to_be_locked < 'a > , #[doc(hidden)] pub
        __rtic_internal_marker : core :: marker :: PhantomData < & 'a () > ,
    } #[doc = r" Spawns the task directly"] #[allow(non_snake_case)]
    #[doc(hidden)] pub fn __rtic_internal_drain_spawn() -> :: core :: result
    :: Result < (), () >
    {
        unsafe
        {
            let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
            from_ptr_1_args(drain, & __rtic_internal_drain_EXEC); if
            exec.try_allocate()
            {
                exec.spawn(drain(unsafe { drain :: Context :: new() })); rtic
                :: export :: pend(lm3s6965 :: interrupt :: UART0); Ok(())
            } else { Err(()) }
        }
    } #[doc = r" Gives waker to the task"] #[allow(non_snake_case)]
    #[doc(hidden)] pub fn __rtic_internal_drain_waker() -> :: core :: task ::
    Waker
    {
        unsafe
        {
            let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
            from_ptr_1_args(drain, & __rtic_internal_drain_EXEC);
            exec.waker(||
            {
                let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
                from_ptr_1_args(drain, & __rtic_internal_drain_EXEC);
                exec.set_pending(); rtic :: export ::
                pend(lm3s6965 :: interrupt :: UART0);
            })
        }
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_drain_Context <
    'a >
    {
        #[doc(hidden)] __rtic_internal_p : :: core :: marker :: PhantomData <
        & 'a () > , #[doc = r" Shared Resources this task has access to"] pub
        shared : drain :: SharedResources < 'a > ,
    } impl < 'a > __rtic_internal_drain_Context < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_drain_Context
            {
                __rtic_internal_p : :: core :: marker :: PhantomData, shared :
                drain :: SharedResources :: new(),
            }
        }
    } #[allow(non_snake_case)] #[doc = "Software task"] pub mod drain
    {
        #[doc(inline)] pub use super :: __rtic_internal_drainSharedResources
        as SharedResources; #[doc(inline)] pub use super ::
        __rtic_internal_drain_spawn as spawn; #[doc(inline)] pub use super ::
        __rtic_internal_drain_waker as waker; #[doc(inline)] pub use super ::
        __rtic_internal_drain_Context as Context;
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = "Local resources `pump` has access to"] pub struct
    __rtic_internal_pumpLocalResources < 'a >
    {
        #[allow(missing_docs)] pub tx : & 'a mut Sender < 'static, u32, 4 > ,
        #[allow(missing_docs)] pub rx : & 'a mut Receiver < 'static, u32, 4 >
        , #[doc(hidden)] pub __rtic_internal_marker : :: core :: marker ::
        PhantomData < & 'a () > ,
    } #[doc = r" Spawns the task directly"] #[allow(non_snake_case)]
    #[doc(hidden)] pub fn __rtic_internal_pump_spawn() -> :: core :: result ::
    Result < (), () >
    {
        unsafe
        {
            let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
            from_ptr_1_args(pump, & __rtic_internal_pump_EXEC); if
            exec.try_allocate()
            {
                exec.spawn(pump(unsafe { pump :: Context :: new() })); rtic ::
                export :: pend(lm3s6965 :: interrupt :: UART1); Ok(())
            } else { Err(()) }
        }
    } #[doc = r" Gives waker to the task"] #[allow(non_snake_case)]
    #[doc(hidden)] pub fn __rtic_internal_pump_waker() -> :: core :: task ::
    Waker
    {
        unsafe
        {
            let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
            from_ptr_1_args(pump, & __rtic_internal_pump_EXEC);
            exec.waker(||
            {
                let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
                from_ptr_1_args(pump, & __rtic_internal_pump_EXEC);
                exec.set_pending(); rtic :: export ::
                pend(lm3s6965 :: interrupt :: UART1);
            })
        }
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_pump_Context <
    'a >
    {
        #[doc(hidden)] __rtic_internal_p : :: core :: marker :: PhantomData <
        & 'a () > , #[doc = r" Local Resources this task has access to"] pub
        local : pump :: LocalResources < 'a > ,
    } impl < 'a > __rtic_internal_pump_Context < 'a >
    {
        #[inline(always)] #[allow(missing_docs)] pub unsafe fn new() -> Self
        {
            __rtic_internal_pump_Context
            {
                __rtic_internal_p : :: core :: marker :: PhantomData, local :
                pump :: LocalResources :: new(),
            }
        }
    } #[allow(non_snake_case)] #[doc = "Software task"] pub mod pump
    {
        #[doc(inline)] pub use super :: __rtic_internal_pumpLocalResources as
        LocalResources; #[doc(inline)] pub use super ::
        __rtic_internal_pump_spawn as spawn; #[doc(inline)] pub use super ::
        __rtic_internal_pump_waker as waker; #[doc(inline)] pub use super ::
        __rtic_internal_pump_Context as Context;
    } #[allow(non_snake_case)] async fn drain < 'a >
    (mut cx : drain :: Context < 'a >)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ; let value
        = cx.shared.counter.lock(| counter : & mut u32 | * counter); let _ =
        value;
    } #[allow(non_snake_case)] async fn pump < 'a >
    (cx : pump :: Context < 'a >)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ; let tx : &
        mut Sender < 'static, u32, 4 > = cx.local.tx; let rx : & mut Receiver
        < 'static, u32, 4 > = cx.local.rx; tx.send(1).await.ok(); if let
        Ok(value) = rx.recv().await { let _ = value; }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic0"] static
    __rtic_internal_shared_resource_counter : rtic :: RacyCell < core :: mem
    :: MaybeUninit < u32 >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: counter_that_needs_to_be_locked < 'a >
    {
        type T = u32; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut u32) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 3u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_counter.get_mut() as *
                mut _, CEILING, lm3s6965 :: NVIC_PRIO_BITS, f,)
            }
        }
    } mod shared_resources
    {
        #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        counter_that_needs_to_be_locked < 'a >
        {
            __rtic_internal_p : :: core :: marker :: PhantomData <
            (& 'a (), * const u8) > ,
        } unsafe impl < 'a > Sync for counter_that_needs_to_be_locked < 'a >
        {} impl < 'a > counter_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new() -> Self
            {
                counter_that_needs_to_be_locked
                { __rtic_internal_p : :: core :: marker :: PhantomData }
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic1"] static
    __rtic_internal_local_resource_toggle : rtic :: RacyCell < core :: mem ::
    MaybeUninit < bool >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic2"] static
    __rtic_internal_local_resource_tx : rtic :: RacyCell < core :: mem ::
    MaybeUninit < Sender < 'static, u32, 4 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic3"] static
    __rtic_internal_local_resource_rx : rtic :: RacyCell < core :: mem ::
    MaybeUninit < Receiver < 'static, u32, 4 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_upper_case_globals)] static __rtic_internal_drain_EXEC : rtic
    :: export :: executor :: AsyncTaskExecutorPtr = rtic :: export :: executor
    :: AsyncTaskExecutorPtr :: new(); #[allow(non_upper_case_globals)] static
    __rtic_internal_pump_EXEC : rtic :: export :: executor ::
    AsyncTaskExecutorPtr = rtic :: export :: executor :: AsyncTaskExecutorPtr
    :: new(); #[allow(non_snake_case)]
    #[doc = "Interrupt handler to dispatch async tasks at priority 1"]
    #[no_mangle] unsafe fn UART0()
    {
        #[doc = r" The priority of this interrupt handler"] const PRIORITY :
        u8 = 1u8; rtic :: export ::
        run(PRIORITY, ||
        {
            let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
            from_ptr_1_args(drain, & __rtic_internal_drain_EXEC);
            exec.poll(||
            {
                let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
                from_ptr_1_args(drain, & __rtic_internal_drain_EXEC);
                exec.set_pending(); rtic :: export ::
                pend(lm3s6965 :: interrupt :: UART0);
            });
        });
    } #[allow(non_snake_case)]
    #[doc = "Interrupt handler to dispatch async tasks at priority 2"]
    #[no_mangle] unsafe fn UART1()
    {
        #[doc = r" The priority of this interrupt handler"] const PRIORITY :
        u8 = 2u8; rtic :: export ::
        run(PRIORITY, ||
        {
            let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
            from_ptr_1_args(pump, & __rtic_internal_pump_EXEC);
            exec.poll(||
            {
                let exec = rtic :: export :: executor :: AsyncTaskExecutor ::
                from_ptr_1_args(pump, & __rtic_internal_pump_EXEC);
                exec.set_pending(); rtic :: export ::
                pend(lm3s6965 :: interrupt :: UART1);
            });
        });
    } #[doc(hidden)] #[no_mangle] unsafe extern "C" fn main() -> !
    {
        rtic :: export :: assert_send :: < u32 > (); rtic :: export ::
        assert_send :: < bool > (); rtic :: export :: assert_send :: < Sender
        < 'static, u32, 4 > > (); rtic :: export :: assert_send :: < Receiver
        < 'static, u32, 4 > > (); rtic :: export :: interrupt :: disable();
        let mut core : rtic :: export :: Peripherals = rtic :: export ::
        Peripherals :: steal().into(); let _ =
        you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml ::
        interrupt :: UART0; let _ =
        you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml ::
        interrupt :: UART1; const _ : () = if
        (1 << lm3s6965 :: NVIC_PRIO_BITS) < 1u8 as usize
        {
            :: core :: panic!
            ("Maximum priority used by interrupt vector 'UART0' is more than supported by hardware");
        };
        core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
        :: interrupt :: UART0, rtic :: export ::
        cortex_logical2hw(1u8, lm3s6965 :: NVIC_PRIO_BITS),); rtic :: export
        :: NVIC ::
        unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
        :: interrupt :: UART0); const _ : () = if
        (1 << lm3s6965 :: NVIC_PRIO_BITS) < 2u8 as usize
        {
            :: core :: panic!
            ("Maximum priority used by interrupt vector 'UART1' is more than supported by hardware");
        };
        core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
        :: interrupt :: UART1, rtic :: export ::
        cortex_logical2hw(2u8, lm3s6965 :: NVIC_PRIO_BITS),); rtic :: export
        :: NVIC ::
        unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
        :: interrupt :: UART1); const _ : () = if
        (1 << lm3s6965 :: NVIC_PRIO_BITS) < 3u8 as usize
        {
            :: core :: panic!
            ("Maximum priority used by interrupt vector 'GPIOA' is more than supported by hardware");
        };
        core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
        :: interrupt :: GPIOA, rtic :: export ::
        cortex_logical2hw(3u8, lm3s6965 :: NVIC_PRIO_BITS),); rtic :: export
        :: NVIC ::
        unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
        :: interrupt :: GPIOA); #[inline(never)] fn __rtic_init_resources < F
        > (f : F) where F : FnOnce() { f(); } let mut executors_size = 0; let
        executor = :: core :: mem :: ManuallyDrop ::
        new(rtic :: export :: executor :: AsyncTaskExecutor ::
        new_1_args(drain));
        {
            executors_size += :: core :: mem :: size_of_val(& executor);
            __rtic_internal_drain_EXEC.set_in_main(& executor);
        } let executor = :: core :: mem :: ManuallyDrop ::
        new(rtic :: export :: executor :: AsyncTaskExecutor ::
        new_1_args(pump));
        {
            executors_size += :: core :: mem :: size_of_val(& executor);
            __rtic_internal_pump_EXEC.set_in_main(& executor);
        } extern "C"
        { pub static _stack_start : u32; pub static __ebss : u32; } let
        stack_start = & _stack_start as * const _ as u32; let ebss = & __ebss
        as * const _ as u32; if stack_start > ebss
        {
            if rtic :: export :: msp :: read() <= ebss
            {
                :: core :: panic!
                ("Stack overflow after allocating executors");
            }
        }
        __rtic_init_resources(||
        {
            let (shared_resources, local_resources) =
            init(init :: Context :: new(core.into(), executors_size));
            __rtic_internal_shared_resource_counter.get_mut().write(core ::
            mem :: MaybeUninit :: new(shared_resources.counter));
            __rtic_internal_local_resource_toggle.get_mut().write(core :: mem
            :: MaybeUninit :: new(local_resources.toggle));
            __rtic_internal_local_resource_tx.get_mut().write(core :: mem ::
            MaybeUninit :: new(local_resources.tx));
            __rtic_internal_local_resource_rx.get_mut().write(core :: mem ::
            MaybeUninit :: new(local_resources.rx)); rtic :: export ::
            interrupt :: enable();
        }); idle(idle :: Context :: new())
    }
}