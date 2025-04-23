//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[allow(dead_code)] // 允许未使用代码的编译警告（临时用于开发阶段）
pub struct TaskInfo {
    /// 任务状态生命周期（运行中/就绪/阻塞等）
    /// 使用TaskStatus枚举类型，定义在task模块中
    status: TaskStatus,
    
    /// 系统调用统计数组（按系统调用类型索引）
    /// 数组长度由config::MAX_SYSCALL_NUM配置（通常为256）
    /// 示例：syscall_times[1] 表示SYS_WRITE系统调用的调用次数
    syscall_times: [u32; MAX_SYSCALL_NUM],
    
    /// 任务总运行时间（单位：毫秒）
    /// 通过定时器中断计数实现时间统计
    /// 
    /// 在每次任务切换时更新该值
    time: usize,
}


/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
// 跟踪系统调用实现（支持三种操作类型）
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    match trace_request {
        // 操作类型0：读取跟踪数据
        // id参数作为内存地址指针，返回该地址的字节数据
        // 安全警告：未验证指针有效性，存在潜在风险
        0 => {
            let data_ptr = id as *const u8;
            unsafe { *data_ptr as isize } // 直接解引用原始指针
        }
        
        // 操作类型1：写入跟踪数据
        // id参数作为内存地址指针，将data的低8位写入该地址
        // 安全警告：未验证指针有效性，可能引发内存错误
        1 => {
            let data_ptr = id as *mut u8;
            unsafe { *data_ptr = data as u8 }; // 写入单字节数据
            0 // 返回成功
        }
        
        // 操作类型2：获取系统调用统计
        // id参数作为系统调用ID，返回对应调用的次数统计
        2 => {
            let task = TASK_MANAGER.current_task(); // 获取当前任务控制块
            let info = task.info(); // 获取任务信息结构体
            info.syscall_times[id] as isize // 返回指定系统调用的调用次数
        }
        
        // 处理不支持的请求类型
        _ => {
            trace!("Unsupported trace request: {}", trace_request);
            -1 // 返回错误码
        }
    }
}
