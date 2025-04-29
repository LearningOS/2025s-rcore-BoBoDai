### 1 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

测试1报错
```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
```
行为
```
用户程序（application）在执行过程中发生了页错误（Page Fault）
程序试图访问地址 0x0，也就是空指针或者无效内存（Null Pointer Dereference）
触发错误的是位于地址 0x804003a4 的指令
内核检测到这个错误后，终止了应用程序
```
测试2报错
```
[kernel] IllegalInstruction in application, kernel killed it.
```
行为
```
在执行过程中遇到了一条非法指令。这通常意味着程序试图执行，需要确认CPU处于S模式才能使用sret
```
测试3同上

sbi版本：
```
RustSBI-QEMU Version 0.2.0-alpha.2
```
### 2 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:
#### 2.1 L40：刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景。
sp 代表的就是陷入时在内核栈上分配出的 TrapContext 起始地址。

__restore 的两种使用情景：
- trap_handler 正常返回用户态
    - 用户态执行 syscall → 进入 __alltraps
    - 保存现场，调用 trap_handler
    - trap_handler 正常处理完了
    - 于是直接跳到 __restore
    - __restore 恢复原上下文，然后执行 sret 回到用户态
- trap_handler 进程切换时需要手动调度
    - 用户态 trap → 进入 __alltraps
    - 保存原来任务的内核栈 TrapContext
    - trap_handler 检查发现：要进行进程切换
    - 当前任务的 TrapContext 保留不动
    - 找到下一个即将运行的任务的 TrapContext
    - 手动修改 sp → 指向新的任务的 TrapContext起始地址
    - 然后 trap_handler 返回到 __restore
    - __restore 从新的 TrapContext 中恢复寄存器
    - sret 返回到另一个进程的用户态
#### 2.2 L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

- ld t0, 32*8(sp)
    - 从 (sp + 32*8) 的位置加载数据到 t0
    - 32*8 = 256，也就是说 sp 上偏移 256 字节的地方存的是 陷入时保存下来的 sstatus
    - sstatus：保存了陷入前的用户态权限、关中断状态等
- ld t1, 33*8(sp)
    - 从 (sp + 33*8) 的位置加载数据到 t1
    - 33*8 = 264，存的是 陷入时保存下来的 sepc
    - sepc：用户态程序在陷入时暂停的位置，恢复后要从这里继续执行
- ld t2, 2*8(sp)
    - 从 (sp + 2*8) 加载到 t2
    - 2*8 = 16，这个地方存的是原先用户态的 sp
- csrw sstatus, t0
    - 把 t0 的值写回 sstatus
    - 恢复陷入前的特权级状态、中断使能状态等
- csrw sepc, t1
    - 把 t1 的值写回 sepc。
    - 以后执行 sret 指令时，处理器会跳转到这个 sepc 指向的指令继续执行用户态代码。
- csrw sscratch, t2
    - 把 t2 的值写回 sscratch。
    - 恢复用户栈指针。
#### 2.3 L50-L56：为何跳过了 x2 和 x4？
- x2是sp，如果中途恢复了sp那么后续所有寄存器就都乱了，所以要最后处理sp
- x4是tp，tp 一般是存储TLS的基址，通常不会动 tp
#### 2.4 L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
- sscratch 和 sp 中的值得到了切换，实际上就是用户态和内核态的切换
#### 2.5 __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
- sret、 sret 是 RISC-V 规定的返回用户态或低特权态的特权指令
    - sret具体动作
        - 读取 sstatus 寄存器：
        - 里面有个重要的字段叫 SPP
            - SPP = 0 ➔ 表示 trap 前是用户态（U-mode）
            - SPP = 1 ➔ 表示 trap 前是S态（Supervisor mode）
        - 根据 SPP 决定跳转到哪种特权模式：
        - 如果 SPP=0，那么 sret 执行后 CPU 会切换到用户态（U-mode）！！
        - 跳转到 sepc 指定的地址：
        - sepc 保存着trap发生时的指令地址，trap前用户程序执行到哪，恢复到哪。
        - 恢复中断使能（SIE）等状态：
        - 从 sstatus 的 SIE/U/SPIE 等字段恢复中断状态。

#### 2.6 L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
还是交换 sp 和 sscratch 同 2.4
#### 2.7 从 U 态进入 S 态是哪一条指令发生的？
用户调用syscall就会陷入S态度，同时中断和异常也都会陷入S态