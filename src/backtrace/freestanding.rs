
extern {
    #[link_name = "llvm.returnaddress"]
    fn return_address(a: i32) -> *const u8;

    #[link_name = "llvm.frameaddress"]
    fn frame_address(a: i32) -> *const u8;
}

#[derive(Debug, Clone)]
pub struct Frame {
    rbp: u64,
    rsp: u64,
    rip: u64,
}

impl Frame {
    pub fn new(rbp: u64, rsp: u64, rip: u64) -> Frame {
        Frame {
            rbp: rbp,
            rsp: rsp,
            rip: rip,
        }
    }

    pub fn ip(&self) -> *mut u8 {
        if self.rip == 0 {
            0 as *mut u8
        } else {
            (self.rip - 1) as *mut u8
        }

    }

    pub fn symbol_address(&self) -> *mut u8 {
        0 as *mut u8
    }
}

#[inline(always)]
pub fn trace_from(mut curframe: Frame, cb: &mut dyn FnMut(&super::Frame) -> bool) {
    loop {
        let mut bomb = ::Bomb { enabled: true };
        let ctxt = super::Frame {
            inner: curframe.clone(),
        };

        let keep_going = cb(&ctxt);
        bomb.enabled = false;

        if keep_going {
            unsafe {
                curframe.rip = *((curframe.rbp + 8) as *mut u64);
                curframe.rsp = curframe.rbp;
                curframe.rbp = *(curframe.rbp as *mut u64);

                if curframe.rip == 0 || curframe.rbp <= 0xfff {
                    break;
                }
            }
        } else {
            break;
        }
    }
}


#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub fn trace(cb: &mut dyn FnMut(&super::Frame) -> bool) {
    use x86::current::registers;

    let pc = unsafe { return_address(0) as u64 };
    let fp = unsafe {frame_address(1) as u64 };

    let curframe = Frame::new(fp, registers::rsp(), pc);
    trace_from(curframe.clone(), cb);
}


#[cfg(target_arch = "aarch64")]
pub fn trace(cb: &mut dyn FnMut(&super::Frame) -> bool) {
    use armv8::aarch64::cpu;

    let pc = unsafe { return_address(0) as u64 };
    let fp = unsafe {frame_address(1) as u64 };

    let curframe = Frame::new(fp, cpu::sp(), pc);
    trace_from(curframe.clone(), cb);
}

