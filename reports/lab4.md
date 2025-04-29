# 功能实现
```angular2html
拿到用户传过来的fd，利用多态调用实现的方法，将目标数据存入Inode结构体去取用。增加链接时将新文件挂载到老文件的inode_id上即可
```
# 简答作业

 在我们的easy-fs中，root inode起着什么作用？如果root inode中的内容损坏了，会发生什么？

```angular2html
root inode表示根目录 "/"，坏了就无法索引其他文件，
挂载可能失败或返回错误，
尽管数据块和其他 inode 可能仍然存在，但没有 root inode 的索引信息，恢复文件结构非常困难，可能需要磁盘分析工具做“扫盘”式恢复，
如果误识别一个损坏的 inode 为根 inode，可能会访问错误的数据区域，甚至覆盖其他数据。
```


举出使用 pipe 的一个实际应用的例子。 tips:想想你平时咋使用 linux terminal 的？

```angular2html
ps aux | grep "python"
```

如何使用 cat 和 wc 完成一个文件的行数统计？

```angular2html
cat filename | wc -l
```

如果需要在多个进程间互相通信，则需要为每一对进程建立一个管道，非常繁琐，请设计一个更易用的多进程通信机制。

```angular2html
可以用消息队列，进行多播和单播
```
