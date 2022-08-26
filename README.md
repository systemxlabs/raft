# raft
Raft 协议的 Rust 实现。（欢迎交流~ wechat: wx597422850）
- [x] 领导人选举
- [x] 日志复制
- [ ] 集群成员变化
- [ ] 日志压缩
- [ ] 客户端交互
- [ ] 通过MIT6.824 lab2 raft测试

Raft 协议
- 中文：[maemual/raft-zh_cn](https://github.com/maemual/raft-zh_cn/blob/master/raft-zh_cn.md)、doc/raft-zh_cn.pdf、[OneSizeFitsQuorum/raft-thesis-zh_cn](https://github.com/OneSizeFitsQuorum/raft-thesis-zh_cn/blob/master/raft-thesis-zh_cn.md)
- 英文：doc/raft.pdf
- https://raft.github.io/

## 开始


## 参考
- [logcabin/logcabin](https://github.com/logcabin/logcabin)
- [wenweihu86/raft-java](https://github.com/wenweihu86/raft-java)
- [baidu/braft](https://github.com/baidu/braft)
- [b站 MIT 6.824课程](https://www.bilibili.com/video/BV1R7411t71W)
- [如何的才能更好地学习 MIT6.824 分布式系统课程？ - 知乎](https://www.zhihu.com/question/29597104)

## 问题
**1.如果发生网络分区，另一个区会产生一个新的leader，那么客户端提交到旧leader的请求为什么不会导致错误？**

另一个区产生新的leader，说明此区的server数量占多数，那么旧leader所在区的数量占少数，因此旧leader在附加日志时，得不到多数server响应，日志不会被提交，也不会被执行。

**2.如果leader接收客户端的请求量很大，而某些follower无法及时执行日志，导致日志堆积，最终follower可能因内存不足而crash，这种情况如何优化？**

**3.如果leader网络出现单向故障，比如leader可以发出请求但无法接收请求，这样导致leader可以往followers发送心跳，但leader无法收到客户端请求，整个集群不可用，而且leader往follower发送的心跳会压制follower选举出新的leader，这种情况如何解决？**

**4.假如某个节点网络不可用（跟其他节点网络隔离），此节点不断执行选举-选举超时，term会增加到很大，会有什么影响？如何优化？**

**5.心跳间隔、超时选举等时间设置最佳实践？**

**6.日志何时算作提交了？**

在leader将创建的日志条目复制到大多数的服务器上的时候，日志条目就算作提交（即使leadercommit没有增加或者传播到其他followers）。在后续选举时，新leader一定包含这条已提交的日志。

**7.follower收到candidate的RequestVote RPC，是否需要重置选举计时器？**

**8.为什么选举出leader后，leader要立刻append一条Noop日志条目？**

- 由于leader只能提交当前任期产生的日志（前面任期日志通过日志匹配特性间接提交）。假如新leader当选后，长时间没有新日志产生，会造成前面任期日志一直得到不提交。因此在当选leader后立刻添加一个Noop日志，可以防止这种情况发生。
- 集群成员变更采用单步变更，如果leader在没有append任何非配置类型日志条目，就直接append配置类型日志条目，会出现覆盖已提交日志条目bug。具体见 https://blog.openacid.com/distributed/raft-bug/ 。raft作者给的修正方法就是：新leader必须提交一条自己的term的日志, 才允许接变更日志。

**9.leader在复制到多数节点后，在回复客户端前宕机了，此时客户端收到超时进行重试，如何保证重试幂等性以防止同一数据复制多次？**

**10.multi-raft是如何实现的？**

**11.业界成熟raft实现（如etcd、tikv等）中有哪些原raft论文之外的优化方式？**