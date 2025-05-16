# 企鹅怎么叫？

![咕咕嘎嘎](./images/gugugaga.jpg)

---

利用监听 LinuxQQ 发送的通知，实现群消息的转发。

包含一个转发消息到 [danmaku-server](https://github.com/PKUOriginalFire/danmaku-server) 的示例实现。

## 使用方式

使用 docker compose 启动后，容器会在 14500 端口开启 xpra 远程桌面。用 Chrome 浏览器访问此地址，可以在浏览器内操作 LinuxQQ。

登录后，在设置内打开消息通知和自动登录，然后关闭聊天窗口（为了能收到通知）。

之后，收到的群消息就会被转发到弹幕服务，以群聊名称为房间号。
