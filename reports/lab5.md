
# 功能实现
```
    利用银行家算法维护多个状态，增加线程时和创建资源时对数组扩列。
```
# 简答作业

#### 1 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

```
每个线程对应的TCB、被占用的内存、栈和调度信息 都要被回收
```

#### 2 对比以下两种 Mutex.unlock 的实现，二者有什么区别？这些区别可能会导致什么问题？
```
1impl Mutex for Mutex1 {
2    fn unlock(&self) {
3        let mut mutex_inner = self.inner.exclusive_access();
4        assert!(mutex_inner.locked);
5        mutex_inner.locked = false;
6        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
7            add_task(waking_task);
8        }
9    }
10}
11
12impl Mutex for Mutex2 {
13    fn unlock(&self) {
14        let mut mutex_inner = self.inner.exclusive_access();
15        assert!(mutex_inner.locked);
16        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
17            add_task(waking_task);
18        } else {
19            mutex_inner.locked = false;
20        }
21    }
22}
```

```
第一个提前解锁后，锁有可能被其他线程获得，从而无效唤醒，第二个多了一个判断的性能开销更大
```
