# raft
Raft 协议的 Rust 实现。
- [x] 领导人选举
- [ ] 日志复制
- [ ] 集群成员变化
- [ ] 日志压缩
- [ ] 客户端交互

Raft 协议
- 中文：[maemual/raft-zh_cn](https://github.com/maemual/raft-zh_cn/blob/master/raft-zh_cn.md)、doc/raft-zh_cn.pdf
- 英文：doc/raft.pdf
- https://raft.github.io/

## 开始


## 参考
- [wenweihu86/raft-java](https://github.com/wenweihu86/raft-java)
- [logcabin/logcabin](https://github.com/logcabin/logcabin)
- [baidu/braft](https://github.com/baidu/braft)
- [hezhihua/horse-raft](https://github.com/hezhihua/horse-raft)
- [b站 MIT 6.824课程](https://www.bilibili.com/video/BV1R7411t71W)

## 问题
**1.如果发生网络分区，另一个区会产生一个新的leader，那么客户端提交到旧leader的请求为什么不会导致错误？**

另一个区产生新的leader，说明此区的server数量占多数，另一个区的数量占少数，因此旧leader在附加日志时，得不到多数server响应，因此此次日志不会被提交，也不会被执行。

**2.如果leader接收客户端的请求量很大，而某些follower无法及时执行日志，导致日志堆积，最终follower可能因内存不足而crash，这种情况如何优化？**

**3.如果leader网络出现单向故障，比如leader可以发出请求但无法接收请求，这样导致leader可以往followers发送心跳，但leader无法收到客户端请求，整个集群不可用，而且leader往follower发送的心跳会压制follower选举出新的leader，这种情况如何解决？**

**4.假如某个节点网络不可用，此节点不断执行选举-选举超时，term会增加到很大，会有什么影响？如何优化？**